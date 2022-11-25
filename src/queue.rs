/*! <dispatch/queue.h> */
use std::ffi::c_void;
use crate::qos::QoS;
use std::os::raw::c_uint;
use std::mem::MaybeUninit;
use std::pin::Pin;


#[repr(C)]
#[derive(Debug)]
/** Unmanaged queue type, performs no memory management.

Often used as the currency type for dispatch queues in case we don't care about memory management

 */
pub struct Unmanaged(c_void);
blocksr::once_noescape!(pub DispatchSyncBlock() -> ());

impl Unmanaged {
    ///dispatch_sync, block version.  You pass an instance of `DispatchSyncBlock` in here.
    ///
    ///This is the fastest variant and can work without heap allocation, for an example of creating noescape blocks on the stack, see [[blocksr::once_noescape]] documentation
    pub fn sync<F>(&self, block: &DispatchSyncBlock<F>) {
        unsafe {
            dispatch_sync(self, block as *const _ as *const c_void);
        }
    }
    ///dispatch_sync, closure version, passing through a returned value.
    ///
    /// A common pattern for this variant is 'getting some value out of AppKit' which can only be accessed on the main thread.
    /// Some objc developers frown on this pattern due to the possibility of deadlocks, so be on the lookout for deadlocks
    /// when you use the pattern.
    ///
    /// If you don't need a return value, the underlying [Unmanaged::sync] method is faster.
    pub fn sync_ret<F,R>(&self, f: F) -> R where F: FnOnce() -> R + Send, R: Send {
        let mut block_value = MaybeUninit::uninit();
        let mut return_value = MaybeUninit::uninit();
        let block_value = unsafe{ Pin::new_unchecked(&mut block_value) };
        let block_value = unsafe{ DispatchSyncBlock::new(block_value, || {
            return_value.write(f());
        })};
        self.sync(&block_value);
        unsafe{ return_value.assume_init() }
    }

    pub fn async_f(&self, context: *const c_void, work: extern "C" fn (*const c_void)) {
        unsafe {
            dispatch_async_f(self, context, work);
        }
    }
}
extern "C" {
    fn dispatch_get_global_queue(identifier: c_uint, flags: *const c_void) -> *const Unmanaged;
    static _dispatch_main_q: Unmanaged;
    ///block parameter is actually &DispatchSyncBlock
    fn dispatch_sync(queue: &Unmanaged, block: *const c_void);
    fn dispatch_async_f(queue: &Unmanaged, context: *const c_void, work: extern "C" fn (*const c_void));
}

///Like Swift `DispatchQueue.global(qos:)` or `dispatch_get_global_queue`
///
/// <https://developer.apple.com/documentation/dispatch/dispatchqueue/2300077-global>
// --
// > Although dispatch queues are reference-counted objects, you do not need to retain and release the global concurrent queues.
// <https://developer.apple.com/library/archive/documentation/General/Conceptual/ConcurrencyProgrammingGuide/OperationQueues/OperationQueues.html>
pub fn global(qos: QoS) -> Option<&'static Unmanaged> {
    let ptr = unsafe{ dispatch_get_global_queue(qos.as_raw(), std::ptr::null()) };
    if ptr.is_null() {
        None
    }
    else {
        Some(unsafe{&*ptr})
    }
}

///Like Swift `DispatchQueue.main` or `dispatch_get_main_queue()`
///
/// <https://developer.apple.com/documentation/dispatch/dispatchqueue/1781006-main>
pub fn main() -> &'static Unmanaged {
    unsafe { &_dispatch_main_q }
}


#[test] fn get_queue() {
    let _queue = unsafe{ dispatch_get_global_queue(QoS::UserInitiated.as_raw(), std::ptr::null()) };
    println!("{:?}",_queue);
    let _queue2 = global(QoS::Default);
    println!("{:?}",_queue2);
}

