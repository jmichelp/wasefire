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

//! Demonstrates simple LED usage.
//!
//! The board must possess at least one LED and one timer. The applet blinks the first LED with a
//! period of 1 second and 50% duty cycle.

#![no_std]
wasefire::applet!();

use core::time::Duration;

fn main() {
    // Make sure there is at least one LED.
    let num_leds = led::count();
    assert!(num_leds > 0, "Board has no LEDs.");

    // Initialize the first action.
    let mut led_index = 0;
    let mut led_status = led::On;

    // Loop indefinitely.
    loop {
        // Execute the action.
        led::set(led_index, led_status);

        // Prepare the next action.
        led_status = !led_status;
        if matches!(led_status, led::On) {
            led_index = (led_index + 1) % num_leds;
        }

        // Wait before executing the next action.
        clock::sleep(Duration::from_millis(500));
    }
}
