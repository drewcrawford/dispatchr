//! https://clang.llvm.org/docs/Block-ABI-Apple.html#high-level

use std::ffi::c_void;
use std::os::raw::{c_int, c_long};

#[repr(C)]
#[derive(Debug)]
#[doc(hidden)]
pub struct __block_descriptor_1 {
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
#[derive(Debug,Clone)]
#[allow(non_camel_case_types)]
#[doc(hidden)]
pub struct __block_literal<R> {
    pub isa: *mut c_void,
    pub flags: c_int,
    pub reserved: c_int,
    //first arg to this fn ptr is &block_literal_1
    pub invoke: *mut c_void,
    pub descriptor: *mut __block_descriptor_1,
    //variables
    pub rust_fn: *const c_void,
    //store the context fn inline
    pub rust_context: R
}

#[repr(transparent)]
///Block type
pub struct ReadEscapingBlock<T>(pub T);
impl<T> ReadEscapingBlock<T> {
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

/**Defines a block suitable for dispatch_read.
```
# use dispatchr::dispatch_read_block;
use dispatchr::data::Unmanaged;
use std::os::raw::{c_int, c_long};

fn read_fn(context:bool, u: Unmanaged, err: c_int) {
    println!("reading {:?}",context);
}
let b = dispatch_read_block!(read_fn, true,bool);
//pass to dispatch_read
```
*/
#[macro_export]
macro_rules! dispatch_read_block {
    ($f: expr, $r:expr, $R: ty) => {
        {
            use $crate::block_impl::{__block_descriptor_1,_NSConcreteStackBlock,__BLOCK_HAS_STRET,__block_literal,ReadEscapingBlock};
            use std::ffi::c_void;
            extern "C" fn invoke_thunk(block: *mut __block_literal<$R>, data: Unmanaged, error: c_int) {
                let f: fn($R, Unmanaged, c_int) = unsafe{std::mem::transmute((*block).rust_fn)};
                let mut temp = std::mem::MaybeUninit::uninit();
                unsafe{ std::mem::swap(&mut (*block).rust_context, std::mem::transmute(temp.as_mut_ptr())) };
                f(unsafe{ temp.assume_init()},data,error);
            }
            static mut DESCRIPTOR: __block_descriptor_1 = __block_descriptor_1 {
                reserved: 0, //unsafe{std::mem::MaybeUninit::uninit().assume_init()} is unstable as const fn
                size: std::mem::size_of::<__block_literal<$R>>() as i64
            };
            let block = __block_literal {
                isa: unsafe{ _NSConcreteStackBlock},
                flags: __BLOCK_HAS_STRET,
                reserved: 0,
                invoke: invoke_thunk as *mut c_void ,
                descriptor: unsafe{ &mut DESCRIPTOR},
                rust_context: $r,
                rust_fn: $f as fn($R, Unmanaged, c_int) as *const c_void
            };
            ReadEscapingBlock(block)
        }

    }
}


