use std::path::Path;
use crate::dl::Library;
use crate::handles::PluginHandle;

pub struct LoadedPlugin {
    pub handle: PluginHandle,
    _lib: Library,
}

impl LoadedPlugin {
    pub fn name(&self) -> String {
        self.handle.name()
    }
}

pub fn load_plugin(path: &Path) -> Result<LoadedPlugin, String> {
    let lib = Library::open(path)?;
    let ctor: extern "C" fn() -> PluginHandle = lib.symbol(b"_plugin_create")?;
    let handle = ctor();
    Ok(LoadedPlugin { handle, _lib: lib })
}