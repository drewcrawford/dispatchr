//! https://clang.llvm.org/docs/Block-ABI-Apple.html#high-level

use std::ffi::c_void;
use std::os::raw::{c_int, c_long};
use crate::data::Unmanaged;

#[repr(C)]
#[derive(Debug)]
struct block_descriptor_1 {
    pub reserved: c_long,
    pub size: c_long,
    /*
     optional helper functions
        void (*copy_helper)(void *dst, void *src);     // IFF (1<<25)
        void (*dispose_helper)(void *src);             // IFF (1<<25)
        required ABI.2010.3.16
        const char *signature;                         // IFF (1<<30)
     */
}
#[repr(C)]
#[derive(Debug)]
pub(crate) struct block_literal_1 {
    isa: *mut c_void,
    flags: c_int,
    reserved: c_int,
    //first arg to this fn ptr is &block_literal_1
    invoke: *const c_void,
    descriptor: *mut block_descriptor_1,
    /*
    At some length, I looked into whether it makes sense to store types inline here, with some kind of macro system.

    The answer seems to be no, both involve timings of around 65us for dispatch_read.

    This might be a result specific to escaping blocks, as there's some possiblity non-escaping blocks would avoid
    a Box worth of overhead.
     */
    rust_context: *mut c_void
}
static mut BLOCK_DESCRIPTOR_1: block_descriptor_1 = block_descriptor_1 {
    reserved: 0, //unsafe{std::mem::MaybeUninit::uninit().assume_init()} is unstable as const fn
    size: std::mem::size_of::<block_literal_1>() as i64
};


///Block type
#[repr(transparent)]
pub struct ReadEscapingBlock(block_literal_1);
impl ReadEscapingBlock {
    pub unsafe fn as_raw(&self) -> *const c_void {
        std::mem::transmute(self)
    }
}

extern {
    #[doc(hidden)]
    pub static _NSConcreteStackBlock: *mut c_void;
}


// const BLOCK_IS_GLOBAL: c_int = 1<<28;
#[doc(hidden)]
pub const __BLOCK_HAS_STRET: c_int = 1<<29;

pub(crate) fn dispatch_read_block<F>(f: F) -> ReadEscapingBlock where F: FnOnce(Unmanaged, c_int) {
    extern "C" fn invoke_thunk<R>(block: *mut block_literal_1, data: Unmanaged, error: c_int) where R: FnOnce(Unmanaged, c_int) {
        let typed_ptr: *mut R = unsafe{ (*block).rust_context as *mut R};
        let rust_fn = unsafe{ Box::from_raw(typed_ptr)};
        rust_fn(data,error);
    }
    let boxed = Box::new(f);
    let thunk_fn: *const c_void = invoke_thunk::<F> as *const c_void;
    let block = block_literal_1 {
        isa: unsafe{ _NSConcreteStackBlock},
        flags: __BLOCK_HAS_STRET,
        reserved: unsafe{ std::mem::MaybeUninit::uninit().assume_init()},
        invoke: thunk_fn ,
        descriptor: unsafe{ &mut BLOCK_DESCRIPTOR_1},
        rust_context: Box::into_raw(boxed) as *mut c_void,
    };
    ReadEscapingBlock(block)
}



