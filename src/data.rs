use std::ffi::c_void;

#[repr(C)]
#[derive(Debug)]
/**Unmanaged dispatch_data type.  This is only used in its pointer form (e.g. `&Unmanaged`).

Compare with [Managed].
*/
pub struct Unmanaged(c_void);
impl DispatchData for &Unmanaged {
    fn as_unmanaged(&self) -> &Unmanaged {
        self
    }
}
//apparently this value is defined in the header
const DISPATCH_DESTRUCTOR_DEFAULT: *const c_void = std::ptr::null();
impl Unmanaged {
    ///Concatenates two datas together
    pub fn concat(&self, unmanaged: &Unmanaged) -> Managed {
        let o = unsafe{ dispatch_data_create_concat(self, unmanaged) };
        Managed(o as *const Unmanaged)
    }
    ///Creates a new, empty, unmanaged data
    pub fn new() -> *const Unmanaged {
        unsafe {
            dispatch_data_create(std::ptr::null(), 0, std::ptr::null(), DISPATCH_DESTRUCTOR_DEFAULT)
        }
    }
    ///Returns the length of the data.
    pub fn len(&self) -> usize {
        unsafe {
            dispatch_data_get_size(self)
        }
    }
}

///A trait that unifies various other types of `dispatch_data_t`.
pub trait DispatchData {
    ///Returns an unmanaged instance of the data.
    fn as_unmanaged(&self) -> &Unmanaged;
}

///A managed version of `dispatch_data_t`, this will be `dispatch_release`d when it is dropped.
///
/// This is primarily used in its owned flavor (`Managed`) as that is how [Drop] knows to drop it.
///
/// Compare with [Unmanaged].
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


/**Contiguous flavor of dispatch_data_t.

In general, `dispatch_data_t` are not defined to be contiguous (they may hold multiple underlying buffers).
This allows them to better sit into fragmented memory, for applications that don't require contiguousness.

Rust programs often want a slice however (which is guaranteed to be congtiguous) so you can create this type
to copy the data into a contiguous region (to the extent it isn't already) and get your slice.

This type is memory-managed, see [Managed].
*/
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
    ///Returns the inner slice of the contiguous data
    pub fn as_slice(&self) -> &[u8] {
        unsafe{ std::slice::from_raw_parts(self.buffer as *const u8, self.size) }
    }
}

impl Drop for Contiguous {
    fn drop(&mut self) {
        unsafe{ dispatch_release(self.object as *const _ as *const c_void) }
    }
}