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

use alloc::boxed::Box;

use wasefire_applet_api::radio as api;

/// Provides callback support for button events.
pub trait Handler: 'static {
    /// Called when a radio packet is received.
    fn event(&self);
}

impl<F: Fn() + 'static> Handler for F {
    fn event(&self) {
        self()
    }
}

/// Provides listening support for radio events.
#[must_use]
pub struct Listener<H: Handler> {
    handler: *mut H,
}

impl<H: Handler> Listener<H> {
    /// Starts listening for radio events.
    ///
    /// The `button` argument is the index of the button to listen events for. It must be less than
    /// [count()]. The `handler` argument is the callback to be called on events. Note that it may
    /// be an `Fn(state: State)` closure, see [Handler::event()] for callback documentation.
    ///
    /// The listener stops listening when dropped.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// Listener::new(index, |state| debug!("Button has been {state:?}"))
    /// ```
    pub fn new(handler: H) -> Self {
        let handler_func = Self::call;
        let handler = Box::into_raw(Box::new(handler));
        let handler_data = handler as *mut u8;
        unsafe { api::register(api::register::Params { handler_func, handler_data }) };
        Listener { handler }
    }

    /// Stops listening.
    ///
    /// This is equivalent to calling `core::mem::drop()`.
    pub fn stop(self) {
        core::mem::drop(self);
    }

    /// Drops the listener but continues listening.
    ///
    /// This is equivalent to calling `core::mem::forget()`. This can be useful if the listener is
    /// created deeply in the stack but the callback must continue processing events until the
    /// applet exits or traps.
    pub fn leak(self) {
        core::mem::forget(self);
    }

    extern "C" fn call(data: *mut u8) {
        let handler = unsafe { &mut *(data as *mut H) };
        handler.event();
    }
}

impl<H: Handler> Drop for Listener<H> {
    fn drop(&mut self) {
        unsafe { api::unregister() };
        unsafe { Box::from_raw(self.handler) };
    }
}
