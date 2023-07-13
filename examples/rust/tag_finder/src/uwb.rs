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

// DWM3000
// Uses SPIM3 in hardware
//   - SPICLK  P1.15
//   - SPIMISO P1.14
//   - SPIMOSI P1.13
//   - SPICSn  P1.12
//   - WAKEUP  P1.11
//   - IRQ     P1.10
//   - RSTn    P1.08
//   - GPIO0   P1.07
//   - GPIO1   P1.06
//   - RXLED   P1.05
//   - TXLED   P1.04
//   - GPIO4   P1.03
//   - SPIPOL  P1.02
//   - SPIPHA  P1.01
//   - EXTON   P0.03
//   - GPIO7   P0.04
//   - TCXO_EN P0.26

pub struct Dwm3000 {

}

impl Dwm3000 {
    fn wakeup() {
        // WAKEUP pin high
        clock::sleep(Duration::from_millis(1));
        // WAKEUP pin low
        // EXTON pin high
        clock::sleep(Duration::from_millis(1));
    }
}

