use crate::ffi::FfiStr;
use crate::vtable::HostVTable;

#[repr(C)]
pub struct RawContext {
    pub host_state: *mut (),
    pub vtable: &'static HostVTable,
}

pub struct Context<'a> {
    raw: &'a RawContext,
}

impl<'a> Context<'a> {
    #[doc(hidden)]
    pub fn __from_raw(raw: &'a RawContext) -> Self {
        Context { raw }
    }
    pub fn log(&self, msg: &str) {
        (self.raw.vtable.log)(self.raw.host_state, FfiStr::from_str(msg))
    }
    pub fn should_cancel(&self) -> bool {
        (self.raw.vtable.should_cancel)(self.raw.host_state)
    }
}
