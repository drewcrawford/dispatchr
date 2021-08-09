//! <dispatch/io.h>

use std::os::raw::c_int;
use crate::queue::Unmanaged as UnmanagedQueue;
use std::os::unix::io::IntoRawFd;
use std::ffi::c_void;
use crate::data::Unmanaged;
use crate::block_impl::dispatch_read_block;


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
    #[allow(unused)]
    pub fn dispatch_read(fd: dispatch_fd_t, length: usize, queue: UnmanagedQueue,
                         handler: *mut c_void);
}

pub fn read<F>(fd: dispatch_fd_t, length: usize, queue: UnmanagedQueue, handler: F) where F: FnOnce(Unmanaged, c_int) {
    let mut block = dispatch_read_block(handler);
    unsafe{ dispatch_read(fd, length, queue, &mut block as *mut _ as *mut c_void) }
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
    read(fd,20,crate::queue::global(QoS::UserInitiated), |data,err| {
        println!("read_begin");
        if err != 0 {
            sender.send(Err(())).unwrap();
        }
        else {
            let as_contig = data.into_contiguous();
            sender.send(Ok(as_contig)).unwrap();
        }
    });
    let item = receiver.recv().unwrap().expect("Not error");
    assert_eq!(item.as_slice(), [47, 47, 33, 32, 60, 100, 105, 115, 112, 97, 116, 99, 104, 47, 105, 111, 46, 104, 62, 10]);
}

