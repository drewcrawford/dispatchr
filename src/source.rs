use std::ops::Deref;
use std::os::raw::c_void;
use libc::uintptr_t;
use crate::data::dispatch_release;
use crate::time::Time;

#[repr(transparent)]
pub struct Unmanaged(c_void);
impl Unmanaged {
    /**
    Calls dispatch_source_create.*/
    pub fn create(tipe: dispatch_source_type_t, handle: uintptr_t, mask: uintptr_t, queue: &crate::queue::Unmanaged) -> *mut Unmanaged {
        unsafe{dispatch_source_create(tipe, handle, mask, queue)}
    }
    pub fn set_event_handler_f(self: &Unmanaged, handler: extern "C" fn(*mut c_void)) {
        unsafe {
            dispatch_source_set_event_handler_f(self, handler)
        }
    }
    pub fn set_timer(self: &Unmanaged, time: Time, interval: u64, leeway: u64) {
        unsafe {
            dispatch_source_set_timer(self, time, interval, leeway)
        }
    }
    pub fn resume(&self) {
        unsafe {
            dispatch_resume(self as *const _ as *const c_void)
        }
    }
    pub fn cancel(&self) {
        unsafe {
            dispatch_source_cancel(self)
        }
    }
    pub fn suspend(&self) {
        unsafe {
            dispatch_suspend(self as *const _ as *const c_void);
        }
    }
}

/**
Drop-managed dispatch_source
*/
#[repr(transparent)]
pub struct Managed(*mut Unmanaged);
impl Drop for Managed {
    fn drop(&mut self) {
        unsafe {
            dispatch_release(self.0 as *mut c_void)
        }
    }
}
impl Managed {
    /**
    Calls dispatch_source_create.*/
    pub fn create(tipe: dispatch_source_type_t, handle: uintptr_t, mask: uintptr_t, queue: &crate::queue::Unmanaged) -> Self {
        Self(Unmanaged::create(tipe, handle, mask, queue))
    }
}
impl Deref for Managed {
    type Target = Unmanaged;

    fn deref(&self) -> &Self::Target {
        unsafe{&*self.0}
    }
}

unsafe impl Send for Managed {}
unsafe impl Sync for Managed {}

#[allow(non_camel_case_types)]
#[repr(transparent)]
pub struct dispatch_source_type_t(*const c_void);

extern "C" {
    static _dispatch_source_type_timer: c_void;
    fn dispatch_source_create(tipe: dispatch_source_type_t, handle: uintptr_t, mask: uintptr_t, queue: *const crate::queue::Unmanaged) -> *mut Unmanaged;
    fn dispatch_source_set_event_handler_f(source: *const Unmanaged, handler: extern "C" fn(*mut c_void));
    fn dispatch_source_set_timer(source: *const Unmanaged, time: Time, interval: u64, leeway: u64);
    fn dispatch_resume(object: *const c_void);
    fn dispatch_source_cancel(source: *const Unmanaged);
    fn dispatch_suspend(object: *const c_void);
}

impl dispatch_source_type_t {
    pub fn timer() -> dispatch_source_type_t {
        unsafe {
            dispatch_source_type_t(&_dispatch_source_type_timer)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::os::raw::c_void;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{SystemTime};
    use crate::QoS;
    use crate::source::{dispatch_source_type_t, Managed};
    use crate::time::Time;

    #[test] fn timer() {
        let queue = crate::queue::global(QoS::Default).unwrap();
        let f = Managed::create(dispatch_source_type_t::timer(), 0, 0, &queue);
        static ARRIVED: AtomicBool = AtomicBool::new(false);
        extern "C" fn handler(_arg: *mut c_void) {
            ARRIVED.store(true, Ordering::Relaxed);
        }
        f.set_event_handler_f(handler);
        f.set_timer(Time::NOW, 1,1_000_000);
        f.resume();
        let started = SystemTime::now();
        while ARRIVED.load(Ordering::Relaxed) == false {
            let elapsed = started.elapsed().unwrap();
            if elapsed.as_secs() > 1 {
                panic!("Never arrived!")
            }
        }
    }
}