// Copyright 2023 Google LLC
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


// Apple AirTag ADV packet
// From: https://www.petsymposium.org/2021/files/papers/issue3/popets-2021-0045.pdf
// (d0, p0) = priv/pub keypair on P-224 curve
// SK0 = 32B symmetric key
// KDF = ANSI X.963 KDF w/ SHA256
// G = Generator on NIST P-224 curve

// SKi = KDF(SKi-1, "update", 32)
// (Ui, Vi) = KDF(SKi, "diversify", 72)
// di = (d0 * Ui) + Vi
// pi = di * G
// This key rotation happens every 15 minutes

// Advertisement packets are sent every 2 seconds
// 6B [BDADDR]
//      (pi[0] | (0b11 << 6)) || pi[1..5]
// 1B Payload length in bytes (30)
// 1B Advertisement type (0xff = Manufacturer specific)
// 2B Company ID (0x004C for Apple)
// 1B Offline-Finding type (0x12)
// 1B Offline-Finding data length in bytes (25)
// 1B Status (e.g. battery level)
// 22B pi[6..27]
// 1B pi[0] >> 6
// 1B Hint

#![no_std]
wasefire::applet!();

fn main() {
    // We define a radio handler printing the new state.
    let handler = || {
        debug!("BLE packet received.");
        let mut buf: [u8; 256] = [0; 256];
        let res = radio::read(&mut buf);
        debug!("radio::read() -> {:?}", res);
    };

    // We start listening for state changes with the handler.
    let _listener = radio::Listener::new(handler);

    // We indefinitely wait for callbacks.
    scheduling::wait_indefinitely();
}
