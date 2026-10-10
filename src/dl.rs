use std::ffi::{c_char, c_int, c_void, CString, CStr};
use std::mem::MaybeUninit;
use std::path::Path;
use std::ptr::write;

unsafe extern "C" {
    unsafe fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    unsafe fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    unsafe fn dlclose(handle: *mut c_void) -> c_int;
    unsafe fn dlerror() -> *mut c_char;
}

const RTLD_NOW: c_int = 2;
const RTLD_LOCAL: c_int = 0;

pub struct Library {
    handle: *mut c_void,
}

unsafe impl Send for Library {}

impl Library {
    pub fn open(path: &Path) -> Result<Self, String> {
        let path_str = path.to_str().ok_or("path does not follow valid encoding")?;
        let c_path = CString::new(path_str).map_err(|_| "path contains null byte")?;

        unsafe {
            dlerror();
            let handle = dlopen(c_path.as_ptr(), RTLD_NOW | RTLD_LOCAL);
            if handle.is_null() {
                return Err(last_error());
            }
            Ok(Library { handle })
        }
    }
    pub fn symbol<T: Copy>(&self, name: &[u8]) -> Result<T, String> {
        if size_of::<T>() != size_of::<*mut c_void>() {
            return Err("symbol::<T> must be called with a function pointer sized type".to_owned());
        }
        let name = CString::new(name).map_err(|_| "symbol name contains null byte")?;

        unsafe {
            dlerror();
            let sym = dlsym(self.handle, name.as_ptr());
            if sym.is_null() {
                let err = last_error();
                if !err.is_empty() {
                    return Err(err);
                }
                return Err(format!("symbol not found {:?}", CStr::from_ptr(name.as_ptr())));
            }
            let mut out = MaybeUninit::<T>::uninit();
            write(out.as_mut_ptr() as *mut *mut c_void, sym);
            Ok(out.assume_init())
        }
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe {
            dlclose(self.handle);
        }
    }
}

fn last_error() -> String {
    unsafe {
        let error = dlerror();
        if error.is_null() {
            String::new()
        } else {
            CStr::from_ptr(error).to_string_lossy().into_owned()
        }
    }
}