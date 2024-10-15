// SPDX-License-Identifier: MIT OR Apache-2.0

use std::os::raw::c_uint;

///Rust QoS type.
///
// --
// Note that this isn't bridged to the C type, you must call [Self::as_raw()] instead.
pub enum QoS {
    UserInteractive,
    UserInitiated,
    Default,
    Utility,
    Background,
    Unspecified
}

impl QoS {
    pub(crate) fn as_raw(&self) -> c_uint {
        match self {
            QoS::UserInteractive => {0x21}
            QoS::UserInitiated => {0x19}
            QoS::Default => {0x15}
            QoS::Utility => {0x11}
            QoS::Background => {0x09}
            QoS::Unspecified => {0x00}
        }
    }
}

impl From<priority::Priority> for QoS {
    fn from(priority: priority::Priority) -> Self {
        match priority {
            priority::Priority::UserInteractive => QoS::UserInteractive,
            priority::Priority::UserInitiated => QoS::UserInitiated,
            priority::Priority::Utility => QoS::Utility,
            priority::Priority::Background => QoS::Background,
            _ => QoS::Default,
        }
    }
}