use crate::data::{Unmanaged, DispatchData};
use crate::queue::Unmanaged as UnmanagedQueue;
use crate::block_impl::{drop_block};
use std::ffi::c_void;
use crate::data::dispatch_release;

pub trait HasMemory {
    fn as_slice(&self) -> &[u8];
}

impl HasMemory for String {
    fn as_slice(&self) -> &[u8] {
        self.as_bytes()
    }
}



///Wraps a dispatch data that points to external memory.
///
/// This implementation will drop both the dispatch_data_t and its external memory.
#[derive(Debug)]
pub struct ExternalMemory {
    object: *const Unmanaged,
}

extern "C" {
    fn dispatch_data_create(buffer: *const c_void, size: usize,
                            queue: *const UnmanagedQueue, destructor: *const c_void) -> *const Unmanaged;
}
impl ExternalMemory {
    pub fn new<T: HasMemory + Send + 'static>(memory: T, destructor_queue: &UnmanagedQueue) -> Self {
        let slice_ptr = memory.as_slice().as_ptr();
        let slice_len = memory.as_slice().len();
        let block = unsafe{ drop_block(memory) };
        let object = unsafe{ dispatch_data_create(slice_ptr as *const c_void,slice_len,destructor_queue, &block as *const _ as *const c_void )};
        Self {
            object
        }
    }
}
impl Drop for ExternalMemory {
    fn drop(&mut self) {
        unsafe{ dispatch_release(self.object as *const c_void) };
    }
}
impl DispatchData for ExternalMemory {
    fn as_unmanaged(&self) -> &Unmanaged {
        unsafe{&*self.object}
    }
}

#[test] fn external_memory() {
    struct TestOwner(std::sync::mpsc::Sender<()>, Box<[u8; 3]>);
    let (sender,receiver) = std::sync::mpsc::channel();

    impl HasMemory for TestOwner {
        fn as_slice(&self) -> &[u8] {
            self.1.as_ref()
        }
    }
    impl Drop for TestOwner {
        fn drop(&mut self) {
            self.0.send(()).unwrap();
        }
    }
    let memory = TestOwner(sender, Box::new([1,2,3]));
    let data = ExternalMemory::new(memory, crate::queue::global(crate::QoS::UserInitiated).unwrap());
    assert!(receiver.recv_timeout(std::time::Duration::from_millis(100)).is_err());
    println!("data {:?}",data);
    //when this is dropped, we should get the data
    drop(data);
    assert!(receiver.recv_timeout(std::time::Duration::from_millis(100)).is_ok());
}