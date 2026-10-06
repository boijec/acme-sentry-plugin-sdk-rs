use crate::context::RawContext;
use crate::error::ErrorInfo;
use crate::value::Value;
use crate::vtable::PluginVTable;

#[repr(C)]
pub struct PluginHandle {
    instance: *mut (),
    vtable: &'static PluginVTable,
}

impl PluginHandle {
    #[doc(hidden)]
    pub fn __new(instance: *mut (), vtable: &'static PluginVTable) -> Self {
        PluginHandle { instance, vtable }
    }
    pub fn name(&self) -> String {
        (self.vtable.name)(self.instance).as_str().to_owned()
    }
    pub fn validate(&self, context: &RawContext, config: &Value) -> Result<(), ErrorInfo> {
        (self.vtable.validate)(self.instance, context, config as *const Value).into()
    }
    pub fn execute(&self, context: &RawContext, config: &Value) -> Result<Value, ErrorInfo> {
        (self.vtable.execute)(self.instance, context, config as *const Value).into()
    }
}

impl Drop for PluginHandle {
    fn drop(&mut self) {
        (self.vtable.drop)(self.instance);
    }
}