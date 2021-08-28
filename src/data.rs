use std::ffi::c_void;

#[repr(C)]
#[derive(Debug)]
pub struct Unmanaged(c_void);
impl DispatchData for &Unmanaged {
    fn as_unmanaged(&self) -> &Unmanaged {
        self
    }
}
const DISPATCH_DESTRUCTOR_DEFAULT: *const c_void = std::ptr::null();
impl Unmanaged {
    pub fn concat(&self, unmanaged: &Unmanaged) -> Managed {
        let o = unsafe{ dispatch_data_create_concat(self, unmanaged) };
        Managed(o as *const Unmanaged)
    }
    pub fn new() -> *const Unmanaged {
        unsafe {
            dispatch_data_create(std::ptr::null(), 0, std::ptr::null(), DISPATCH_DESTRUCTOR_DEFAULT)
        }
    }
    pub fn len(&self) -> usize {
        unsafe {
            dispatch_data_get_size(self)
        }
    }
}

pub trait DispatchData {
    fn as_unmanaged(&self) -> &Unmanaged;
}

#[derive(Debug)]
pub struct Managed(*const Unmanaged);
impl Managed {
    pub fn retain(unmanaged: *const Unmanaged) -> Self {
        unsafe{ dispatch_retain(unmanaged as *const _) }
        Managed(unmanaged)
    }
}
//ok to send this one since it has unlimited lifetime
unsafe impl Send for Managed {}
impl DispatchData for Managed {
    fn as_unmanaged(&self) -> &Unmanaged {
        //safe because this is valid for the lifetime of self
        unsafe{ &*(self.0) }
    }
}
impl Drop for Managed {
    fn drop(&mut self) {
        unsafe{
            dispatch_release(self.0 as *const _ as *const c_void)
        }
    }
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
    ///Creates a new managed object with a contiguous buffer.
    ///
    /// The implementation calls `dispatch_data_create_map`.
    pub fn new<D: DispatchData>(d: D) -> Self {
        let mut buffer = std::ptr::null();
        let mut size = 0;
        let object = unsafe{ dispatch_data_create_map(d.as_unmanaged(), &mut buffer, &mut size)};
        Contiguous {
            buffer,
            size,
            object: unsafe{&*object}
        }
    }
}


extern "C" {
    fn dispatch_data_create_map(data: *const Unmanaged, buffer_ptr: *mut *const c_void,
    size_ptr: *mut usize) -> *const Unmanaged;
    fn dispatch_data_create_concat(data1: *const Unmanaged, data2: *const Unmanaged) -> *const c_void;
    pub fn dispatch_release(object: *const c_void);

    pub fn dispatch_retain(object:  *const c_void);
    fn dispatch_data_create(buffer: *const c_void, size: usize, queue: *const super::queue::Unmanaged, destructor: *const c_void) -> *const Unmanaged;
    fn dispatch_data_get_size(buffer: *const Unmanaged) -> usize;
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