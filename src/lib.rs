pub mod context;
pub mod vtable;
pub mod handles;
pub mod error;
pub mod ffi;
pub mod value;
pub mod loader;
pub mod dl;

pub use context::Context;
pub use error::PluginError;
pub use value::Value;
pub use vtable::PluginVTable;

pub trait Plugin: Default {
    fn name(&self) -> &str;
    fn validate(&self, ctx: &Context, config: &Value) -> Result<(), PluginError>;
    fn execute(&self, ctx: &Context, config: &Value) -> Result<Value, PluginError>;
}

#[macro_export]
macro_rules! export_to_so {
    ($ty:ty) => {
      const _: () = {
          use $crate::context::{Context, RawContext}
          use $crate::error::ErrorInfo;
          use $crate::value::Value;
          use $crate::ffi::{FfiString, FfiResult};
          use $crate::vtable::PluginVTable;
          use $crate::handles::PluginHandle;

          extern "C" fn name_impl(this: *const ()) -> FfiString {
              let plugin = unsafe { &*(this as *const $ty) }
              FfiString::from_string($crate::Plugin::name(plugin).to_owned())
          }
          extern "C" fn validate_impl(
              this: *const (),
              raw_ctx: *const RawContext,
              config: *const Value
          ) -> FfiResult<(), ErrorInfo> {
              let plugin = unsafe { &*(this as *const $ty) }
              let ctx = Context::__from_raw(unsafe { &*raw_ctx });
              let config = unsafe { &*config };
              let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                  $crate::Plugin::validate(plugin, &ctx, config)
              }));
              match result {
                  Ok(Ok(())) => FfiResult::Ok(()),
                  Ok(Err(e)) => FfiResult::Err(e.into()),
                  Err(_) => FfiResult::Err(ErrorInfo {
                      code: -1,
                      message: FfiString::from_string("plugin panicked during validation".into())
                  })
              }
          }
          extern "C" fn execute_impl(
              this: *const (),
              raw_ctx: *const RawContext,
              config: *const Value
          ) -> FfiResult<Value, ErrorInfo> {
              let plugin = unsafe { &*(this as *const $ty) }
              let ctx = Context::__from_raw(unsafe { &*raw_ctx });
              let config = unsafe { &*config };
              let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                  $crate::Plugin::execute(plugin, &ctx, config)
              }));
              match result {
                  Ok(Ok(())) => FfiResult::Ok(()),
                  Ok(Err(e)) => FfiResult::Err(e.into()),
                  Err(_) => FfiResult::Err(ErrorInfo {
                      code: -1,
                      message: FfiString::from_string("plugin panicked during execution".into())
                  })
              }
          }
          extern "C" fn drop_impl(this: *mut ()) {
              unsafe {
                  drop(Box::from_raw(this as *mut $ty))
              }
          }

          static VTABLE: PluginVTable = PluginVTable {
              name: name_impl,
              validate: validate_impl,
              execute: execute_impl,
              drop: drop_impl,
          }

          #[unsafe(no_mangle)]
          pub extern "C" fn _plugin_create() -> PluginHandle {
              let on_heap = Box::new(<$ty as Default>::default());
              PluginHandle::__new(Box::into_raw(on_heap) as *mut (), &VTABLE)
          }
      };
    };
}