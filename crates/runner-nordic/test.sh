#!/bin/sh
# Copyright 2022 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

set -ex

if [ ! -e ../../target/applet.wasm ]; then
  mkdir -p ../../target
  touch ../../target/applet.wasm
fi
cargo check --target=thumbv7em-none-eabi --features=debug,nrf52840
cargo check --target=thumbv7em-none-eabi --features=debug,nrf52833
DEFMT_LOG=trace cargo check --target=thumbv7em-none-eabi --features=debug,nrf52840
cargo check --target=thumbv7em-none-eabi --features=debug,nrf52833
cargo check --target=thumbv7em-none-eabi --features=release,nrf52840
cargo check --target=thumbv7em-none-eabi --features=debug,nrf52833
cargo fmt -- --check
cargo clippy --target=thumbv7em-none-eabi --features=debug,nrf52840 -- --deny=warnings
cargo check --target=thumbv7em-none-eabi --features=debug,nrf52833
