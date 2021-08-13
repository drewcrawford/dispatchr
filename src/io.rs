//! <dispatch/io.h>

use std::os::raw::c_int;
use crate::queue::{Unmanaged as UnmanagedQueue};
use std::os::unix::io::IntoRawFd;
use std::ffi::c_void;
use crate::data::{Unmanaged, DispatchData};
use crate::block_impl::{dispatch_read_block, dispatch_write_block};


#[repr(transparent)]
#[allow(non_camel_case_types)]
#[derive(Clone,Copy)]
pub struct dispatch_fd_t(c_int);
impl dispatch_fd_t {
    pub fn new<F: IntoRawFd>(f: F) -> dispatch_fd_t {
        dispatch_fd_t(f.into_raw_fd())
    }
}

extern "C" {
    fn dispatch_read(fd: dispatch_fd_t, length: usize, queue: *const UnmanagedQueue,
                         handler: *mut c_void);
    fn dispatch_write(fd: dispatch_fd_t, data: *const Unmanaged, queue: *const UnmanagedQueue, handler: *mut c_void);
}

pub fn read<F>(fd: dispatch_fd_t, length: usize, queue: &UnmanagedQueue, handler: F) where F: FnOnce(&Unmanaged, c_int) + Send + 'static {
    let mut block = dispatch_read_block(handler);
    unsafe{ dispatch_read(fd, length, queue, &mut block as *mut _ as *mut c_void) }
    std::mem::forget(block);
}
pub fn write<F,D: DispatchData>(fd: dispatch_fd_t, data: &D, queue: &UnmanagedQueue, handler: F) where F: FnOnce(Option<&Unmanaged>, c_int) + Send + 'static {
    let mut block = dispatch_write_block(handler);
    let actual_data = data.as_unmanaged();
    unsafe { dispatch_write(fd, actual_data, queue, &mut block as *mut _ as *mut c_void)}
    std::mem::forget(block);
}



#[test] fn read_t() {
    use std::os::unix::io::IntoRawFd;
    use crate::qos::QoS;
    let path = std::path::Path::new("src/io.rs");
    let file = std::fs::File::open(path).unwrap();
    use crate::data::{Contiguous};
    let fd = dispatch_fd_t(file.into_raw_fd());
    let (sender,receiver) = std::sync::mpsc::channel::<Result<Contiguous,()>>();
    read(fd,20,crate::queue::global(QoS::UserInitiated).unwrap(), move |data,err| {
        println!("read_begin");
        if err != 0 {
            sender.send(Err(())).unwrap();
        }
        else {
            let as_contig = data.as_contiguous();
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
    write(fd, &data, queue,move |a,b| {
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

