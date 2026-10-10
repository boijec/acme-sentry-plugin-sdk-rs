use crate::context::RawContext;
use crate::error::ErrorInfo;
use crate::ffi::{FfiResult, FfiStr, FfiString};
use crate::value::PluginValue;

#[repr(C)]
pub struct PluginVTable {
    pub name: extern "C" fn(*const ()) -> FfiString,
    pub validate_config: extern "C" fn(*const (), *const RawContext, *const PluginValue) -> FfiResult<(), ErrorInfo>,
    pub set_acme_challenge: extern "C" fn(*const (), *const RawContext, *const PluginValue) -> FfiResult<PluginValue, ErrorInfo>,
    pub unset_acme_challenge: extern "C" fn(*const (), *const RawContext, *const PluginValue) -> FfiResult<PluginValue, ErrorInfo>,
    pub drop: extern "C" fn(*mut ()),
}

#[repr(C)]
pub struct HostVTable {
    pub log: extern "C" fn(*mut (), FfiStr),
    pub should_cancel: extern "C" fn(*mut ()) -> bool,
}