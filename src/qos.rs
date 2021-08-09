use std::os::raw::c_uint;

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