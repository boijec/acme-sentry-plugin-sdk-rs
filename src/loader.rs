use crate::context::RawContext;
use crate::dl::Library;
use crate::error::HostError;
use crate::handles::PluginHandle;
use crate::value::HostValue;
use std::mem::ManuallyDrop;
use std::path::Path;

pub struct LoadedPlugin {
    name: String,
    handle: ManuallyDrop<PluginHandle>,
    lib: ManuallyDrop<Library>,
    policy: UnloadPolicy,
}

impl LoadedPlugin {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn validate_config(&self, ctx: &RawContext, config: &HostValue) -> Result<(), HostError> {
        self.handle.validate_config(ctx, config)
    }
    pub fn set_acme_challenge(
        &self,
        ctx: &RawContext,
        config: &HostValue,
    ) -> Result<HostValue, HostError> {
        self.handle.set_acme_challenge(ctx, config)
    }
    pub fn unset_acme_challenge(
        &self,
        ctx: &RawContext,
        config: &HostValue,
    ) -> Result<HostValue, HostError> {
        self.handle.unset_acme_challenge(ctx, config)
    }
}

impl Drop for LoadedPlugin {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.handle);
            match self.policy {
                UnloadPolicy::KeepLoaded => {} // leak
                UnloadPolicy::Unload => ManuallyDrop::drop(&mut self.lib),
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnloadPolicy {
    KeepLoaded,
    Unload,
}

pub fn load_plugin(path: &Path, unload_policy: UnloadPolicy) -> Result<LoadedPlugin, String> {
    let lib = Library::open(path)?;
    let fingerprint: extern "C" fn() -> u64 = lib.symbol(b"_plugin_layout")?;
    let plugin_print = fingerprint();
    let host_print = crate::plugin_fingerprint();

    if plugin_print != host_print {
        return Err(format!(
            "ABI mismatch for {:?}: plugin fingerprint {plugin_print:#x}, host {host_print:#x} \
             (built against a different plugin_abi or compiler?)",
            path
        ));
    }
    let ctor: extern "C" fn() -> PluginHandle = lib.symbol(b"_plugin_entrypoint")?;
    let handle = ctor();
    let name = handle.name();

    Ok(LoadedPlugin {
        name,
        handle: ManuallyDrop::new(handle),
        lib: ManuallyDrop::new(lib),
        policy: unload_policy,
    })
}
