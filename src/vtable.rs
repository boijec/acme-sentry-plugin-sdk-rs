use crate::context::RawContext;
use crate::error::ErrorInfo;
use crate::ffi::{FfiResult, FfiStr, FfiString};
use crate::value::Value;

#[repr(C)]
pub struct PluginVTable {
    pub name: extern "C" fn(*const ()) -> FfiString,
    pub validate: extern "C" fn(*const (), *const RawContext, *const Value) -> FfiResult<(), ErrorInfo>,
    pub execute: extern "C" fn(*const (), *const RawContext, *const Value) -> FfiResult<Value, ErrorInfo>,
    pub drop: extern "C" fn(*mut ()),
}

#[repr(C)]
pub struct HostVTable {
    pub log: extern "C" fn(*mut (), FfiStr),
    pub report_progress: extern "C" fn(*mut (), f32, FfiStr),
    pub should_cancel: extern "C" fn(*mut ()) -> bool,
}