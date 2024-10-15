// SPDX-License-Identifier: MIT OR Apache-2.0

use std::os::raw::{c_ulonglong};

extern "C" {
    fn dispatch_time(when: Time, delta: i64) -> Time;
}

///Transparent newtype for dispatch_time
///
/// Layout-compatible
#[repr(transparent)]
#[derive(Copy,Clone,Debug)]
pub struct Time(pub c_ulonglong);
impl Time {
    pub const NOW: Time = Time(0);
    pub const FOREVER: Time = Time(!0);

    pub fn new_after(self, delta: i64) -> Self {
        unsafe { dispatch_time(self, delta)}
    }

}