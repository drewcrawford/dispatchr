//! https://clang.llvm.org/docs/Block-ABI-Apple.html#high-level

use std::ffi::c_void;
use std::os::raw::{c_int};
use crate::data::Unmanaged;
use std::marker::PhantomPinned;

use blocksr::once_escaping;
once_escaping!(pub(crate) ReadEscapingBlock(data: &Unmanaged, error: c_int) -> ());

once_escaping!(pub(crate) WriteEscapingBlock(data: Option<&Unmanaged>, error: c_int) -> ());

//all arguments to this one passed in via closure
once_escaping!(pub(crate) DropBlock() -> ());

///For reasons that are a mystery to me, Pin only works for !Unpin types
#[repr(transparent)]
#[allow(dead_code)]
struct ActuallyPinned<T> {
    t: T,
    pin: PhantomPinned // !Unpin
}

///A block that will drop the receiver.  This can be used to transfer
/// ownership of the receiver into dispatch.
///
/// # Safety
/// You must verify that
//  * Block will execute exactly once:
//      * If ObjC executes the block several times, it's UB
//      * If ObjC executes the block less than once, it is not UB, but it will leak.
pub(crate) unsafe fn drop_block<T: Send>(t: T) -> DropBlock {
    DropBlock::new(move || {
        std::mem::drop(t)
    })
}

