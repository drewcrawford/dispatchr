/*! <dispatch/queue.h> */
use std::ffi::c_void;
use crate::qos::QoS;
use std::os::raw::c_uint;


#[repr(C)]
#[derive(Debug)]
/*
> Although dispatch queues are reference-counted objects, you do not need to retain and release the global concurrent queues.
https://developer.apple.com/library/archive/documentation/General/Conceptual/ConcurrencyProgrammingGuide/OperationQueues/OperationQueues.html
 */
pub struct Unmanaged(c_void);


extern "C" {
    fn dispatch_get_global_queue(identifier: c_uint, flags: *const c_void) -> *const Unmanaged;
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


#[test] fn get_queue() {
    let _queue = unsafe{ dispatch_get_global_queue(QoS::UserInitiated.as_raw(), std::ptr::null()) };
    println!("{:?}",_queue);
    let _queue2 = global(QoS::Default);
    println!("{:?}",_queue2);
}