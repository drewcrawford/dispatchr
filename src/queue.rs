/*! <dispatch/queue.h> */
use std::ffi::c_void;
use crate::qos::QoS;
use std::os::raw::c_uint;
use std::mem::MaybeUninit;
use std::pin::Pin;


#[repr(C)]
#[derive(Debug)]
/*
> Although dispatch queues are reference-counted objects, you do not need to retain and release the global concurrent queues.
https://developer.apple.com/library/archive/documentation/General/Conceptual/ConcurrencyProgrammingGuide/OperationQueues/OperationQueues.html
 */
pub struct Unmanaged(c_void);
blocksr::once_noescape!(pub DispatchSyncBlock() -> ());

impl Unmanaged {
    ///dispatch_sync
    pub fn sync<F>(&self, block: &DispatchSyncBlock<F>) {
        unsafe {
            dispatch_sync(self, block as *const _ as *const c_void);
        }
    }
    ///dispatch_sync, closure version, providing a returned value.
    ///
    /// If you don't need a return value, the underlying [Unmanaged::sync] method may be faster.
    pub fn sync_ret<F,R>(&self, f: F) -> R where F: FnOnce() -> R + Send, R: Send {
        let mut block_value = MaybeUninit::uninit();
        let mut return_value = MaybeUninit::uninit();
        let block_value = unsafe{ Pin::new_unchecked(&mut block_value) };
        let block_value = unsafe{ DispatchSyncBlock::new(block_value, || {
           *return_value.as_mut_ptr() = f();
        })};
        self.sync(&block_value);
        unsafe{ return_value.assume_init() }
    }
}
extern "C" {
    fn dispatch_get_global_queue(identifier: c_uint, flags: *const c_void) -> *const Unmanaged;
    static _dispatch_main_q: Unmanaged;
    ///block parameter is actually &DispatchSyncBlock
    fn dispatch_sync(queue: &Unmanaged, block: *const c_void);
}

//Nice Rust functions.  These map the swift API DispatchQueue() type
pub fn global(qos: QoS) -> Option<&'static Unmanaged> {
    let ptr = unsafe{ dispatch_get_global_queue(qos.as_raw(), std::ptr::null()) };
    if ptr.is_null() {
        None
    }
    else {
        Some(unsafe{&*ptr})
    }
}

pub fn main() -> &'static Unmanaged {
    unsafe { &_dispatch_main_q }
}


#[test] fn get_queue() {
    let _queue = unsafe{ dispatch_get_global_queue(QoS::UserInitiated.as_raw(), std::ptr::null()) };
    println!("{:?}",_queue);
    let _queue2 = global(QoS::Default);
    println!("{:?}",_queue2);
}

