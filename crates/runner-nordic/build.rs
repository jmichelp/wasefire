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

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    #[cfg(feature = "nrf52833")]
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("nrf52833_memory.x"))
        .unwrap();
    #[cfg(feature = "nrf52840")]
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("nrf52840_memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    // TODO(jmichel): Should we feature-gate these too?
    println!("cargo:rerun-if-changed=nrf52833_memory.x");
    println!("cargo:rerun-if-changed=nrf52840_memory.x");
}
