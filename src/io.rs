//! <dispatch/io.h>

use std::os::raw::{c_char, c_int, c_ulong};
use crate::queue::{Unmanaged as UnmanagedQueue};
use std::os::unix::io::IntoRawFd;
use std::ffi::{c_void, CStr};
use std::ops::Deref;
use std::ptr::NonNull;
use libc::{mode_t, off_t, size_t};
use crate::data::{Unmanaged, DispatchData, dispatch_release};
use crate::block_impl::{WriteEscapingBlock};

///dispatch type for file descriptor
#[repr(transparent)]
#[allow(non_camel_case_types)]
#[derive(Clone,Copy)]
pub struct dispatch_fd_t(c_int);
impl dispatch_fd_t {
    pub fn new<F: IntoRawFd>(f: F) -> dispatch_fd_t {
        dispatch_fd_t(f.into_raw_fd())
    }
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct dispatch_io_type_t(pub c_ulong);
impl dispatch_io_type_t {
    pub const STREAM: dispatch_io_type_t = dispatch_io_type_t(0);
    pub const RANDOM: dispatch_io_type_t = dispatch_io_type_t(1);
}

#[repr(C)]
pub struct UnmanagedIO(c_void);
unsafe impl Send for UnmanagedIO {}
unsafe impl Sync for UnmanagedIO {}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct dispatch_io_close_flags_t(pub c_ulong);

impl  Default for dispatch_io_close_flags_t {
    fn default() -> Self {
        Self(0)
    }
}
impl dispatch_io_close_flags_t {
    pub const STOP: dispatch_io_close_flags_t = dispatch_io_close_flags_t(0x1);
}


extern "C" {
    fn dispatch_read(fd: dispatch_fd_t, length: usize, queue: *const UnmanagedQueue,
                         handler: *mut c_void);
    fn dispatch_write(fd: dispatch_fd_t, data: *const Unmanaged, queue: *const UnmanagedQueue, handler: *mut c_void);
    fn dispatch_io_create(tipe: dispatch_io_type_t, fd: dispatch_fd_t, queue: *const UnmanagedQueue, cleanup_handler: *mut c_void) -> *mut UnmanagedIO;
    fn dispatch_io_create_with_path(tipe: dispatch_io_type_t, path: *const c_char, oflag: c_int, mode_t: mode_t, queue: *const UnmanagedQueue,cleanup_handler: *mut c_void) -> *mut UnmanagedIO;
    fn dispatch_io_read(channel: *const UnmanagedIO, offset: off_t, length: size_t, queue: *const UnmanagedQueue, handler: *const c_void);
    fn dispatch_io_close(channel: *const UnmanagedIO, flags: dispatch_io_close_flags_t);
}

///Calls `dispatch_read` with the specified completion handler.  You can use a `blocksr::continuation` to wrap this in an async method if desired.
pub fn read_completion<F>(fd: dispatch_fd_t, length: usize, queue: &UnmanagedQueue, handler: F) where F: FnOnce(*const Unmanaged, c_int) + Send + 'static {
    unsafe{
        use crate::block_impl::ReadEscapingBlock;
        let mut block = ReadEscapingBlock::new(handler);
        dispatch_read(fd, length, queue, &mut block as *mut _ as *mut c_void)
    }
}

///Calls `dispatch_write` with the specified completion handler.  You can use a `blocksr::continuation` to wrap this in an async method if desired.
pub fn write_completion<F,D: DispatchData>(fd: dispatch_fd_t, data: &D, queue: &UnmanagedQueue, handler: F) where F: FnOnce(Option<&Unmanaged>, c_int) + Send + 'static {
    unsafe {
        let mut block = WriteEscapingBlock::new(handler);
        let actual_data = data.as_unmanaged();
        dispatch_write(fd, actual_data, queue, &mut block as *mut _ as *mut c_void)
    }
}

impl UnmanagedIO {
    pub fn new(tipe: dispatch_io_type_t, fd: dispatch_fd_t, queue: &UnmanagedQueue) -> *mut Self {
        unsafe {
            dispatch_io_create(tipe, fd, queue, std::ptr::null_mut())
        }
    }
    ///Calls `dispatch_io_create_with_path`.
    ///
    /// Since optional closures are tough in rust, just omit them for now...
    pub fn new_with_path(tipe: dispatch_io_type_t,path: &CStr,  oflag: c_int,  mode_t: mode_t, queue: &UnmanagedQueue) -> *mut Self {
        unsafe {
            dispatch_io_create_with_path(tipe, path.as_ptr(), oflag, mode_t, queue, std::ptr::null_mut() )
        }
    }
    ///Calls `dispatch_io_read`.
    pub fn read<H: FnMut(&mut E, bool, *const Unmanaged, c_int) + Send + 'static,E>(&self, offset: off_t, length: size_t, queue: *const UnmanagedQueue, handler: H,initial_environment: E) {
        unsafe {
            blocksr::many_escaping_nonreentrant!(DataHandler (environment: &mut E, done: bool, data: *const Unmanaged, error: c_int) -> ());

            let mut block = DataHandler::new(initial_environment, handler);
            dispatch_io_read(self, offset, length, queue, &mut block as *mut _ as *mut c_void);
        }
    }
    pub fn close(&self, flags: dispatch_io_close_flags_t) {
        unsafe{dispatch_io_close(self, flags)}
    }
}

/**
Lifetime-managed dispatch channel.

 This type will be automatically closed AND released upon drop.

Therefore, there is no need to call .close().
 */
pub struct IO(NonNull<UnmanagedIO>);
unsafe impl Send for IO {}
unsafe impl Sync for IO {}
impl IO {
    ///Calls `dispatch_io_create_with_path`.
    ///
    /// Since optional closures are tough in rust, just omit them for now...
    pub fn new_with_path(tipe: dispatch_io_type_t,path: &CStr,  oflag: c_int,  mode_t: mode_t, queue: &UnmanagedQueue) -> Option<Self> {
        let ptr = UnmanagedIO::new_with_path(tipe, path,oflag,mode_t,queue);
        unsafe {
            if ptr.is_null() {
                None
            }
            else {
                Some(Self(NonNull::new_unchecked(ptr)))
            }
        }
    }
    ///Calls `dispatch_io_create`.
    /// Since optional closures are tough in rust, just omit them for now...
    pub fn new(tipe: dispatch_io_type_t, fd: dispatch_fd_t, queue: &UnmanagedQueue) -> Option<Self> {
        let ptr = UnmanagedIO::new(tipe, fd, queue);
        unsafe {
            if ptr.is_null() {
                None
            }
            else {
                Some(Self(NonNull::new_unchecked(ptr)))
            }
        }
    }
}

impl Deref for IO {
    type Target = UnmanagedIO;

    fn deref(&self) -> &Self::Target {
        unsafe{self.0.as_ref()}
    }
}

impl Drop for IO {
    fn drop(&mut self) {
        self.close(dispatch_io_close_flags_t::STOP);
        unsafe{
            dispatch_release(self.0.as_ptr() as *const c_void)
        }
    }
}



#[test] fn read_t() {
    use std::os::unix::io::IntoRawFd;
    use crate::qos::QoS;
    let path = std::path::Path::new("src/io.rs");
    let file = std::fs::File::open(path).unwrap();
    use crate::data::{Contiguous};
    let fd = dispatch_fd_t(file.into_raw_fd());
    let (sender,receiver) = std::sync::mpsc::channel::<Result<Contiguous,()>>();
    read_completion(fd,20,crate::queue::global(QoS::UserInitiated).unwrap(), move |data,err| {
        if err != 0 {
            sender.send(Err(())).unwrap();
        }
        else {
            let as_contig = Contiguous::new(unsafe{&*data});
            sender.send(Ok(as_contig)).unwrap();
        }
    });
    let item = receiver.recv().unwrap().expect("Not error");
    assert_eq!(item.as_slice(), [47, 47, 33, 32, 60, 100, 105, 115, 112, 97, 116, 99, 104, 47, 105, 111, 46, 104, 62, 10]);
}

#[test] fn write_t() {
    use std::os::unix::io::IntoRawFd;
    use crate::qos::QoS;
    use crate::queue::global;
    use crate::external_data::{ExternalMemory, HasMemory};
    let path = std::path::Path::new("/tmp/dispatchr_write_t.txt");
    let file = std::fs::File::create(path).unwrap();
    let fd = dispatch_fd_t(file.into_raw_fd());
    let (sender,receiver) = std::sync::mpsc::channel::<Result<(),()>>();
    struct StaticMemory;
    impl HasMemory for StaticMemory {
        fn as_slice(&self) -> &[u8] {
            "hello from the test".as_bytes()
        }
    }
    let queue = global(QoS::UserInitiated).unwrap();
    let data = ExternalMemory::new(StaticMemory, queue);
    write_completion(fd, &data, queue,move |a,b| {
        if b == 0 {
            sender.send(Ok(())).unwrap()
        }
        else {
            sender.send(Err(())).unwrap()
        }
        println!("hello from write {:?} {:?}",a,b);
    });
    receiver.recv().unwrap().expect("Not error");

    let result = std::fs::read(path).unwrap();
    assert!(result.as_slice() == "hello from the test".as_bytes());
}

#[test] fn create_io() {
    let path = std::path::Path::new("src/io.rs").canonicalize().unwrap();
    use std::os::unix::ffi::OsStrExt;
    use std::ffi::CString;
    use crate::qos::QoS;
    let c_path = CString::new(path.as_os_str().as_bytes()).unwrap();
    let queue = super::queue::global(QoS::Default).unwrap();
    let f = UnmanagedIO::new_with_path(dispatch_io_type_t::STREAM, &c_path, 0, 0, queue);
    assert!(!f.is_null());
    let as_ref = unsafe{&*f};
    as_ref.close(dispatch_io_close_flags_t::default());
    unsafe{
        dispatch_release(as_ref as *const _ as *const c_void);
    }

    let p = IO::new_with_path(dispatch_io_type_t::STREAM, &c_path, 0, 0, queue);
    assert!(p.is_some());

    //wait for 'done'
    use std::sync::mpsc::sync_channel;
    let (sender,receiver) = sync_channel(0);
    let channel = p.unwrap();
    channel.read(0, 100, queue,  |environment, done,data,err| {
        println!("hi");
        if done {
            assert_eq!(err,0);
            assert!(unsafe{&*data}.len()>=100);
            environment.send(()).unwrap();
        }
    },sender);

    receiver.recv_timeout(std::time::Duration::new(10,0)).unwrap();
}

#[test] fn assert_send() {
    fn assert_send<T: Send>() {}
    assert_send::<IO>();
}