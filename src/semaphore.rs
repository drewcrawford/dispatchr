use std::ffi::c_void;
use std::ops::Deref;
use crate::data::{dispatch_release, dispatch_retain};
use crate::time::Time;

extern "C" {
    fn dispatch_semaphore_create(value: isize) -> *mut Unmanaged;
    fn dispatch_semaphore_wait(sema:*const Unmanaged, timeout: Time) -> isize;
    fn dispatch_semaphore_signal(sema: *const Unmanaged) -> isize;
}
///An unmanaged GCD semaphore.  Generally, you work with pointers of this type and/or ManagedSemaphore.
#[derive(Debug)]
#[repr(transparent)]
pub struct Unmanaged(c_void);

impl Unmanaged {
    ///Creates an unmanaged DispatchSemaphore.
    pub fn new(value: isize) -> *mut Unmanaged {
        unsafe { dispatch_semaphore_create(value)}
    }
    ///# Safety
    ///
    /// Undefined behavior to use the argument after calling this function.
    unsafe fn release(&mut self) {
        //'safe' because
        //1.  We are guaranteed to have an exclusive reference by Rust semantics
        //2.  Reference assumed to be valid by Rust semantics
        //3.  Caller guarantees we only do this once.
        dispatch_release(self as *mut Unmanaged as *const Unmanaged as *const c_void)
    }
    ///Calls dispatch_semaphore_wait.
    pub fn wait(&self, time: Time) -> isize {
        unsafe {
            dispatch_semaphore_wait(self, time)
        }
    }
    ///Calls dispatch_semaphore_signal.
    pub fn signal(&self) -> isize {
        unsafe {
            dispatch_semaphore_signal(self)
        }
    }
}

///Memory-managed wrapper for GCD semaphore.
#[derive(Debug)]
pub struct Managed(*mut Unmanaged);
impl Managed {
    pub fn new(value: isize) -> Self {
        Self(Unmanaged::new(value))
    }
}
impl Drop for Managed {
    fn drop(&mut self) {
        unsafe{ (&mut *(self.0)).release()};
    }
}
impl Deref for Managed {
    type Target = Unmanaged;

    fn deref(&self) -> &Self::Target {
        unsafe{&*self.0}
    }
}
impl Clone for Managed {
    fn clone(&self) -> Self {
        unsafe{dispatch_retain(self.0 as *const c_void)};
        Managed {
            0: self.0
        }
    }
}
unsafe impl Sync for Managed {}
unsafe impl Send for Managed {}

#[test] fn test_allocation() {
    let f = Unmanaged::new(0);
    unsafe {
        //give f an unbounded lifetime
        let f: &mut Unmanaged = &mut *f;
        f.release()
    }
}
#[test] fn test_wait() {
    unsafe {
        let f = Managed::new(0);
        //we're going to cheat and pretend f is static
        let static_f: &'static Managed = std::mem::transmute(&f);
        //deal with unbounded lifetimes in here
        std::thread::spawn(move || {
            static_f.signal();
        });
        f.wait(Time::FOREVER);
        //try cloning
        let _ = f.clone();

    }
}
