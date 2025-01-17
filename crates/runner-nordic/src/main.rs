// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![no_std]
#![no_main]
#![feature(try_blocks)]

extern crate alloc;

mod allocator;
mod storage;
#[cfg(feature = "debug")]
mod systick;
mod tasks;

use alloc::collections::VecDeque;
use core::cell::RefCell;
use core::mem::MaybeUninit;

use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use critical_section::Mutex;
#[cfg(feature = "debug")]
use defmt_rtt as _;
#[cfg(feature = "nrf52833")]
use nrf52833_hal as nrf5x_hal;
#[cfg(feature = "nrf52840")]
use nrf52840_hal as nrf5x_hal;
use nrf5x_hal::ccm::{Ccm, DataRate};
use nrf5x_hal::clocks::{self, ExternalOscillator, Internal, LfOscStopped};
use nrf5x_hal::gpio;
use nrf5x_hal::gpio::{Level, Output, Pin, PushPull};
use nrf5x_hal::gpiote::Gpiote;
use nrf5x_hal::pac::{interrupt, Interrupt, TIMER0};
use nrf5x_hal::prelude::InputPin;
use nrf5x_hal::rng::Rng;
use nrf5x_hal::usbd::{UsbPeripheral, Usbd};
#[cfg(feature = "release")]
use panic_abort as _;
#[cfg(feature = "debug")]
use panic_probe as _;
use rubble::beacon::{BeaconScanner, ScanCallback};
use rubble::bytes::{ByteWriter, ToBytes};
use rubble::link::ad_structure::AdStructure;
use rubble::link::filter::AllowAll;
use rubble::link::{DeviceAddress, Metadata, MIN_PDU_BUF};
use rubble::time::Timer;
use rubble_nrf5x::radio::{BleRadio, PacketBuffer};
use rubble_nrf5x::timer::BleTimer;
use storage::Storage;
use tasks::button::{channel, Button};
use tasks::clock::Timers;
use tasks::usb::Usb;
use tasks::{button, led, Events};
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::device::{UsbDevice, UsbDeviceBuilder, UsbVidPid};
use usbd_serial::{SerialPort, USB_CLASS_CDC};
use wasefire_board_api::usb::serial::Serial;
use wasefire_board_api::{Id, Support};
use wasefire_scheduler::Scheduler;
use {wasefire_board_api as board, wasefire_logger as logger};

#[cfg(feature = "debug")]
#[defmt::panic_handler]
fn panic() -> ! {
    panic_probe::hard_fault();
}

type Clocks = clocks::Clocks<ExternalOscillator, Internal, LfOscStopped>;

struct State {
    events: Events,
    buttons: [Button; <button::Impl as Support<usize>>::SUPPORT],
    gpiote: Gpiote,
    serial: Serial<'static, Usb>,
    timers: Timers,
    ccm: Ccm,
    leds: [Pin<Output<PushPull>>; <led::Impl as Support<usize>>::SUPPORT],
    ble_radio: BleRadio,
    ble_scanner: BeaconScanner<TrackerScanCallback, AllowAll>,
    ble_timer: BleTimer<TIMER0>,
    ble_packet_queue: VecDeque<BlePacket>,
    rng: Rng,
    storage: Option<Storage>,
    usb_dev: UsbDevice<'static, Usb>,
}

pub enum Board {}

static STATE: Mutex<RefCell<Option<State>>> = Mutex::new(RefCell::new(None));
static BLE_PACKET: Mutex<RefCell<Option<BlePacket>>> = Mutex::new(RefCell::new(None));

fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
    critical_section::with(|cs| f(STATE.borrow_ref_mut(cs).as_mut().unwrap()))
}

#[entry]
fn main() -> ! {
    static mut CLOCKS: MaybeUninit<Clocks> = MaybeUninit::uninit();
    static mut USB_BUS: MaybeUninit<UsbBusAllocator<Usb>> = MaybeUninit::uninit();
    // TX buffer is mandatory even when we only listen
    static mut BLE_TX: MaybeUninit<PacketBuffer> = MaybeUninit::uninit();
    static mut BLE_RX: MaybeUninit<PacketBuffer> = MaybeUninit::uninit();

    #[cfg(feature = "debug")]
    let c = nrf5x_hal::pac::CorePeripherals::take().unwrap();
    #[cfg(feature = "debug")]
    systick::init(c.SYST);
    allocator::init();
    logger::debug!("Runner starts.");
    let p = nrf5x_hal::pac::Peripherals::take().unwrap();
    let port0 = gpio::p0::Parts::new(p.P0);
    let buttons = [
        Button::new(port0.p0_11.into_pullup_input().degrade()),
        Button::new(port0.p0_12.into_pullup_input().degrade()),
        Button::new(port0.p0_24.into_pullup_input().degrade()),
        Button::new(port0.p0_25.into_pullup_input().degrade()),
    ];
    let leds = [
        port0.p0_13.into_push_pull_output(Level::High).degrade(),
        port0.p0_14.into_push_pull_output(Level::High).degrade(),
        port0.p0_15.into_push_pull_output(Level::High).degrade(),
        port0.p0_16.into_push_pull_output(Level::High).degrade(),
    ];
    let timers = Timers::new(p.TIMER1, p.TIMER2, p.TIMER3, p.TIMER4);
    let gpiote = Gpiote::new(p.GPIOTE);
    // We enable all USB interrupts except STARTED and EPDATA which are feedback loops.
    p.USBD.inten.write(|w| unsafe { w.bits(0x00fffffd) });
    let clocks = CLOCKS.write(clocks::Clocks::new(p.CLOCK).enable_ext_hfosc());
    let usb_bus = UsbBusAllocator::new(Usbd::new(UsbPeripheral::new(p.USBD, clocks)));
    let usb_bus = USB_BUS.write(usb_bus);
    let serial = Serial::new(SerialPort::new(usb_bus));
    let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .product("Serial port")
        .device_class(USB_CLASS_CDC)
        .build();

    // Setup BLE radio to scan for peripherals but let the applet start it
    let ble_radio = BleRadio::new(
        p.RADIO,
        &p.FICR,
        BLE_TX.write([0; MIN_PDU_BUF]),
        BLE_RX.write([0; MIN_PDU_BUF]),
    );
    let ble_timer = BleTimer::init(p.TIMER0);
    let ble_scanner = BeaconScanner::new(TrackerScanCallback);
    let ble_packet_queue = VecDeque::<BlePacket>::new();

    let rng = Rng::new(p.RNG);
    let ccm = Ccm::init(p.CCM, p.AAR, DataRate::_1Mbit);
    let storage = Some(Storage::new(p.NVMC));
    let events = Events::default();
    let state = State {
        events,
        buttons,
        gpiote,
        serial,
        timers,
        ccm,
        leds,
        ble_radio,
        ble_scanner,
        ble_timer,
        ble_packet_queue,
        rng,
        storage,
        usb_dev,
    };
    // We first set the board and then enable interrupts so that interrupts may assume the board is
    // always present.
    critical_section::with(|cs| STATE.replace(cs, Some(state)));
    for &interrupt in INTERRUPTS {
        unsafe { NVIC::unmask(interrupt) };
    }
    logger::debug!("Runner is initialized.");
    const WASM: &[u8] = include_bytes!("../../../target/applet.wasm");
    Scheduler::<Board>::run(WASM)
}

pub struct RadioMetadata {
    ticks: u32,
    freq: u16,
    rssi: i8,
    pdu_type: u8,
}

impl RadioMetadata {
    pub fn len(&self) -> usize {
        6
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}

impl From<Metadata> for RadioMetadata {
    fn from(value: Metadata) -> Self {
        RadioMetadata {
            ticks: value.timestamp.unwrap().ticks(),
            freq: match value.channel {
                ch @ 0 ..= 10 => 2404 + 2 * (ch as u16),
                ch @ 11 ..= 36 => 2404 + 2 * (ch as u16 + 1),
                37 => 2402,
                38 => 2426,
                39 => 2480,
                _ => 0,
            },
            rssi: value.rssi.unwrap(),
            pdu_type: u8::from(value.pdu_type.unwrap()),
        }
    }
}

pub struct BlePacket {
    addr: [u8; 6],
    metadata: RadioMetadata,
    data: alloc::vec::Vec<u8>,
}

impl BlePacket {
    pub fn len(&self) -> usize {
        self.addr.len() + self.metadata.len() + self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}

pub struct TrackerScanCallback;

impl ScanCallback for TrackerScanCallback {
    fn beacon<'a, I>(&mut self, addr: DeviceAddress, data: I, metadata: Metadata)
    where I: Iterator<Item = AdStructure<'a>> {
        let mut buf: [u8; MIN_PDU_BUF] = [0; MIN_PDU_BUF];
        let mut writer = ByteWriter::new(&mut buf);
        for p in data {
            assert!(p.to_bytes(&mut writer).is_ok());
        }
        let len = MIN_PDU_BUF - writer.space_left();
        let packet = BlePacket {
            addr: *addr.raw(),
            metadata: metadata.clone().into(),
            data: buf[.. len].to_vec(),
        };
        logger::trace!(
            "[{}] CH:{} Type:{}, RSSI:{}dBm BDADDR:{:x}, DATA:{:x}",
            packet.metadata.ticks,
            packet.metadata.freq,
            metadata.channel,
            packet.metadata.rssi,
            packet.addr,
            buf[.. len],
        );
        assert!(critical_section::with(|cs| BLE_PACKET.replace(cs, Some(packet))).is_none());
    }
}

macro_rules! interrupts {
    ($($name:ident = $func:ident($($arg:expr),*$(,)?)),*$(,)?) => {
        const INTERRUPTS: &[Interrupt] = &[$(Interrupt::$name),*];
        $(
            #[interrupt]
            fn $name() {
                $func($($arg),*);
            }
        )*
    };
}

interrupts! {
    GPIOTE = gpiote(),
    RADIO = radio(),
    TIMER0 = radio_timer(),
    TIMER1 = timer(0),
    TIMER2 = timer(1),
    TIMER3 = timer(2),
    TIMER4 = timer(3),
    USBD = usbd(),
}

fn gpiote() {
    with_state(|state| {
        for (i, button) in state.buttons.iter_mut().enumerate() {
            let id = Id::new(i).unwrap();
            if channel(&state.gpiote, id).is_event_triggered() {
                let pressed = button.pin.is_low().unwrap();
                state.events.push(board::button::Event { button: id, pressed }.into());
            }
        }
        state.gpiote.reset_events();
    });
}

fn radio() {
    with_state(|state| {
        if let Some(next_update) =
            state.ble_radio.recv_beacon_interrupt(state.ble_timer.now(), &mut state.ble_scanner)
        {
            state.ble_timer.configure_interrupt(next_update);
            critical_section::with(|cs| {
                if let Some(packet) = BLE_PACKET.take(cs) {
                    if state.ble_packet_queue.len() < 10 {
                        state.ble_packet_queue.push_back(packet);
                        state.events.push(board::radio::Event::Received.into());
                    } else {
                        logger::warn!("BLE Packet dropped");
                    }
                }
            });
        }
    })
}

fn radio_timer() {
    with_state(|state| {
        if !state.ble_timer.is_interrupt_pending() {
            return;
        }
        state.ble_timer.clear_interrupt();

        let cmd = state.ble_scanner.timer_update(state.ble_timer.now());
        state.ble_radio.configure_receiver(cmd.radio);
        state.ble_timer.configure_interrupt(cmd.next_update);
    });
}

fn timer(timer: usize) {
    let timer = Id::new(timer).unwrap();
    with_state(|state| {
        state.events.push(board::timer::Event { timer }.into());
        state.timers.tick(*timer);
    })
}

fn usbd() {
    #[cfg(feature = "debug")]
    {
        use core::sync::atomic::AtomicU32;
        use core::sync::atomic::Ordering::SeqCst;
        static COUNT: AtomicU32 = AtomicU32::new(0);
        static MASK: AtomicU32 = AtomicU32::new(0);
        let count = COUNT.fetch_add(1, SeqCst).wrapping_add(1);
        let mut mask = 0;
        for i in 0 ..= 24 {
            let x = (0x40027100 + 4 * i) as *const u32;
            let x = unsafe { core::ptr::read_volatile(x) };
            core::assert!(x <= 1);
            mask |= x << i;
        }
        mask |= MASK.fetch_or(mask, SeqCst);
        if count % 1000 == 0 {
            logger::trace!("Got {} USB interrupts matching {:08x}.", count, mask);
            MASK.store(0, SeqCst);
        }
    }
    with_state(|state| {
        let polled = state.usb_dev.poll(&mut [state.serial.port()]);
        state.serial.tick(polled, |event| state.events.push(event.into()));
    });
}
