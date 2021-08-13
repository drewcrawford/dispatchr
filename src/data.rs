use std::ffi::c_void;

#[repr(C)]
#[derive(Debug)]
pub struct Unmanaged(c_void);

pub trait DispatchData {
    fn as_unmanaged(&self) -> &Unmanaged;
}


#[derive(Debug)]
pub struct Contiguous {
    //owned type
    object: *const Unmanaged,
    buffer: *const c_void,
    size: usize
}
unsafe impl Send for Contiguous {}
impl Contiguous {
    ///Returns a reference to the inner dispatch object.
    pub fn as_dispatch_data(&self) -> &Unmanaged {
        //should be valid for self lifetime
        unsafe{ &*self.object }
    }
}


extern "C" {
    fn dispatch_data_create_map(data: *const Unmanaged, buffer_ptr: *mut *const c_void,
    size_ptr: *mut usize) -> *const Unmanaged;
    pub fn dispatch_release(object: *const c_void);

}

impl Unmanaged {
    ///Creates a new managed object with a contiguous buffer.
    ///
    /// The implementation calls `dispatch_data_create_map`.
    pub fn as_contiguous(&self) -> Contiguous {
        let mut buffer = std::ptr::null();
        let mut size = 0;
        let object = unsafe{ dispatch_data_create_map(self, &mut buffer, &mut size)};
        Contiguous {
            buffer,
            size,
            object: unsafe{&*object}
        }
    }
}

impl Contiguous {
    pub fn as_slice(&self) -> &[u8] {
        unsafe{ std::slice::from_raw_parts(self.buffer as *const u8, self.size) }
    }
}

impl Drop for Contiguous {
    fn drop(&mut self) {
        unsafe{ dispatch_release(self.object as *const _ as *const c_void) }
    }
}