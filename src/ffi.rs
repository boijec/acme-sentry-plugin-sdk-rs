use std::mem::ManuallyDrop;
use std::ptr::slice_from_raw_parts;
use std::slice::from_raw_parts;
use std::str::from_utf8_unchecked;

#[repr(C)]
pub struct FfiStr {
    ptr: *const u8,
    len: usize,
}

impl FfiStr {
    pub fn from_str(str: &str) -> Self {
        FfiStr { ptr: str.as_ptr(), len: str.len() }
    }
    #[allow(unused)]
    pub fn as_str(&self) -> &str {
        unsafe {
            from_utf8_unchecked(from_raw_parts(self.ptr, self.len))
        }
    }
}

#[repr(C)]
pub struct FfiString {
    ptr: *mut u8,
    len: usize,
    drop_fn: extern "C" fn(*mut u8, usize),
}

impl FfiString {
    pub fn from_string(str: String) -> Self {
        let on_heap: Box<str> = str.into_boxed_str();
        let len = on_heap.len();
        let ptr = Box::into_raw(on_heap) as *mut u8;

        extern "C" fn drop_impl(ptr: *mut u8, len: usize) {
            unsafe {
                let slice = slice_from_raw_parts(ptr, len);
                drop(Box::from_raw(slice as *mut str));
            }
        }
        FfiString {
            ptr,
            len,
            drop_fn: drop_impl,
        }
    }
    pub fn as_str(&self) -> &str {
        unsafe {
            from_utf8_unchecked(from_raw_parts(self.ptr, self.len))
        }
    }
}

impl Drop for FfiString {
    fn drop(&mut self) {
        (self.drop_fn)(self.ptr, self.len)
    }
}

unsafe impl Send for FfiString {}

#[repr(C)]
pub struct FfiVec<T> {
    ptr: *mut T,
    len: usize,
    drop_fn: extern "C" fn(*mut T, usize),
}

impl<T> FfiVec<T> {
    pub fn from_vec(mut v: Vec<T>) -> Self {
        v.shrink_to_fit();
        let mut v = ManuallyDrop::new(v);
        let ptr = v.as_mut_ptr();
        let len = v.len();
        extern "C" fn drop_impl<T>(ptr: *mut T, len: usize) {
            unsafe {
                drop(Vec::from_raw_parts(ptr, len, len))
            }
        }
        FfiVec {
            ptr,
            len,
            drop_fn: drop_impl,
        }
    }
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            from_raw_parts(self.ptr, self.len)
        }
    }
}

impl<T> Drop for FfiVec<T> {
    fn drop(&mut self) {
        (self.drop_fn)(self.ptr, self.len)
    }
}

unsafe impl<T: Send> Send for FfiVec<T> {}

#[repr(C, u8)]
pub enum FfiResult<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> From<Result<T, E>> for FfiResult<T, E> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(v) => FfiResult::Ok(v),
            Err(e) => FfiResult::Err(e),
        }
    }
}
impl<T, E> From<FfiResult<T, E>> for Result<T, E> {
    fn from(result: FfiResult<T, E>) -> Self {
        match result {
            FfiResult::Ok(v) => Ok(v),
            FfiResult::Err(e) => Err(e),
        }
    }
}