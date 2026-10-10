use crate::context::RawContext;
use crate::error::{ErrorInfo, HostError};
use crate::value::{HostValue, PluginValue};
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
    pub(crate) fn name(&self) -> String {
        (self.vtable.name)(self.instance).as_str().to_owned()
    }
    pub(crate) fn validate_config(&self,
                                  context: &RawContext,
                                  config: &HostValue
    ) -> Result<(), HostError> {
        let ffi_cnf = config.to_ffi();
        let res: Result<(), ErrorInfo> = (self.vtable.validate_config)(self.instance, context, &ffi_cnf as *const PluginValue).into();
        res.map_err(|e| e.to_host())
    }
    pub(crate) fn set_acme_challenge(&self, context: &RawContext, config: &HostValue) -> Result<HostValue, HostError> {
        let ffi_cnf = config.to_ffi();
        let res: Result<PluginValue, ErrorInfo> = (self.vtable.set_acme_challenge)(self.instance, context, &ffi_cnf as *const PluginValue).into();
        Self::extract_execution_result(res)
    }
    pub(crate) fn unset_acme_challenge(&self, context: &RawContext, config: &HostValue) -> Result<HostValue, HostError> {
        let ffi_cnf = config.to_ffi();
        let res: Result<PluginValue, ErrorInfo> = (self.vtable.unset_acme_challenge)(self.instance, context, &ffi_cnf as *const PluginValue).into();
        Self::extract_execution_result(res)
    }

    fn extract_execution_result(res: Result<PluginValue, ErrorInfo>) -> Result<HostValue, HostError> {
        match res {
            Ok(v) => {
                let host_value = v.to_host();
                drop(v);
                Ok(host_value)
            }
            Err(e) => {
                let host_err = e.to_host();
                drop(e);
                Err(host_err)
            }
        }
    }
}

impl Drop for PluginHandle {
    fn drop(&mut self) {
        (self.vtable.drop)(self.instance);
    }
}

unsafe impl Send for PluginHandle {}