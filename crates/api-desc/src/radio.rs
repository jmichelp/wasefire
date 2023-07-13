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

use crate::*;

pub(crate) fn new() -> Item {
    let docs = docs! {
        /// Radio operations.
    };
    let name = "radio".into();
    let items = vec![
        item! {
            /// Register a handler for radio events.
            fn register "rr" {
                /// Function called on radio events.
                ///
                /// The function takes its opaque `data` as argument.
                handler_func: fn { data: *mut u8 },

                /// The opaque data to use when calling the handler function.
                handler_data: *mut u8,
            } -> {}
        },
        item! {
            /// Unregister handlers for radio events.
            fn unregister "ru" {
            } -> {}
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
