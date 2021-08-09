use std::ffi::c_void;

#[repr(transparent)]
#[derive(Debug)]
pub struct Unmanaged(*mut c_void);

#[derive(Debug)]
pub struct Contiguous{
    object: Unmanaged,
    buffer: *const c_void,
    size: usize
}


extern "C" {
    fn dispatch_data_create_map(data: Unmanaged, buffer_ptr: *mut *const c_void,
    size_ptr: *mut usize) -> Unmanaged;
    fn dispatch_release(object: *mut c_void);
}

impl Unmanaged {
    ///Creates a new managed object with a contiguous buffer.
    ///
    /// The implementation calls `dispatch_data_create_map`.
    pub fn into_contiguous(self) -> Contiguous {
        let mut buffer = std::ptr::null();
        let mut size = 0;
        let object = unsafe{ dispatch_data_create_map(self, &mut buffer, &mut size)};
        Contiguous {
            buffer,
            size,
            object
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
        unsafe{ dispatch_release(self.object.0) }
    }
}