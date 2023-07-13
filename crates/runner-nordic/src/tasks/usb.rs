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

#[cfg(feature = "nrf52833")]
use nrf52833_hal as nrf5x_hal;
#[cfg(feature = "nrf52840")]
use nrf52840_hal as nrf5x_hal;
use nrf5x_hal::usbd::{UsbPeripheral, Usbd};
use wasefire_board_api::usb::serial::{HasSerial, Serial, WithSerial};
use wasefire_board_api::usb::Api;

use crate::with_state;

pub type Usb = Usbd<UsbPeripheral<'static>>;

pub enum Impl {}

impl Api for Impl {
    type Serial = WithSerial<Impl>;
}

impl HasSerial for Impl {
    type UsbBus = Usb;

    fn with_serial<R>(f: impl FnOnce(&mut Serial<Self::UsbBus>) -> R) -> R {
        with_state(|state| f(&mut state.serial))
    }
}
