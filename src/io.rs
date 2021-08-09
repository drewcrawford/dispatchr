//! <dispatch/io.h>

use std::os::raw::c_int;
use crate::queue::UnmanagedQueue;
use std::os::unix::io::IntoRawFd;
use std::ffi::c_void;


#[repr(transparent)]
#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct dispatch_fd_t(c_int);
impl dispatch_fd_t {
    pub fn new<F: IntoRawFd>(f: F) -> dispatch_fd_t {
        dispatch_fd_t(f.into_raw_fd())
    }
}

extern "C" {
    #[allow(unused)]
    pub fn dispatch_read(fd: dispatch_fd_t, length: usize, queue: UnmanagedQueue,
    handler: *const c_void);
}



#[test] fn read_t() {
    use std::os::unix::io::IntoRawFd;
    use crate::qos::QoS;
    let path = std::path::Path::new("src/io.rs");
    let file = std::fs::File::open(path).unwrap();
    use crate::data::{Contiguous,Unmanaged};
    use std::sync::mpsc::Sender;
    let fd = dispatch_fd_t(file.into_raw_fd());
    let (sender,receiver) = std::sync::mpsc::channel::<Result<Contiguous,()>>();
    fn read_fn(context:Sender<Result<Contiguous, ()>>, u: Unmanaged, err: c_int) {
        println!("read_begin");
        if err != 0 {
            context.send(Err(())).unwrap();
        }
        else {
            let as_contig = u.into_contiguous();
            println!("as_contig {:?}",as_contig);
            context.send(Ok(as_contig)).unwrap();
        }
    }
    let block = crate::dispatch_read_block!(read_fn, sender, std::sync::mpsc::Sender<Result<Contiguous,()>> );
    unsafe { dispatch_read(fd,20,crate::queue::global(QoS::UserInitiated),block.as_raw())};

    // read(fd,20,crate::queue::global(QoS::UserInitiated), read_fn);
    let item = receiver.recv().unwrap().expect("Not error");
    assert_eq!(item.as_slice(), [47, 47, 33, 32, 60, 100, 105, 115, 112, 97, 116, 99, 104, 47, 105, 111, 46, 104, 62, 10]);
}

