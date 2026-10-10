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
pub use value::PluginValue;
pub use vtable::PluginVTable;

pub trait Plugin: Default {
    fn name(&self) -> &str;
    fn validate_config(&self, ctx: &Context, config: &PluginValue) -> Result<(), PluginError>;
    fn set_acme_challenge(&self, ctx: &Context, config: &PluginValue) -> Result<PluginValue, PluginError>;
    fn unset_acme_challenge(&self, ctx: &Context, config: &PluginValue) -> Result<PluginValue, PluginError>;
}

#[macro_export]
macro_rules! export_to_so {
    ($ty:ty) => {
      const _: () = {
          use $crate::context::{Context, RawContext};
          use $crate::error::ErrorInfo;
          use $crate::value::PluginValue;
          use $crate::ffi::{FfiString, FfiResult};
          use $crate::vtable::PluginVTable;
          use $crate::handles::PluginHandle;

          extern "C" fn name_impl(this: *const ()) -> FfiString {
              let plugin = unsafe {
                  &*(this as *const $ty)
              };
              FfiString::from_string($crate::Plugin::name(plugin).to_owned())
          }
          extern "C" fn validate_config_impl(
              this: *const (),
              raw_ctx: *const RawContext,
              config: *const PluginValue,
          ) -> FfiResult<(), ErrorInfo> {
              let plugin = unsafe {
                  &*(this as *const $ty)
              };
              let ctx = Context::__from_raw(unsafe { &*raw_ctx });
              let config = unsafe { &*config };
              let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                  $crate::Plugin::validate_config(plugin, &ctx, config)
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
          extern "C" fn set_acme_challenge_impl(
              this: *const (),
              raw_ctx: *const RawContext,
              config: *const PluginValue,
          ) -> FfiResult<PluginValue, ErrorInfo> {
              let plugin = unsafe {
                  &*(this as *const $ty)
              };
              let ctx = Context::__from_raw(unsafe { &*raw_ctx });
              let config = unsafe { &*config };
              let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                  $crate::Plugin::set_acme_challenge(plugin, &ctx, config)
              }));
              match result {
                  Ok(Ok(v)) => FfiResult::Ok(v),
                  Ok(Err(e)) => FfiResult::Err(e.into()),
                  Err(_) => FfiResult::Err(ErrorInfo {
                      code: -1,
                      message: FfiString::from_string("plugin panicked during set of acme challenge".into())
                  })
              }
          }
          extern "C" fn unset_acme_challenge_impl(
              this: *const (),
              raw_ctx: *const RawContext,
              config: *const PluginValue,
          ) -> FfiResult<PluginValue, ErrorInfo> {
              let plugin = unsafe {
                  &*(this as *const $ty)
              };
              let ctx = Context::__from_raw(unsafe { &*raw_ctx });
              let config = unsafe { &*config };
              let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                  $crate::Plugin::unset_acme_challenge(plugin, &ctx, config)
              }));
              match result {
                  Ok(Ok(v)) => FfiResult::Ok(v),
                  Ok(Err(e)) => FfiResult::Err(e.into()),
                  Err(_) => FfiResult::Err(ErrorInfo {
                      code: -1,
                      message: FfiString::from_string("plugin panicked during unset of acme challenge".into())
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
              validate_config: validate_config_impl,
              set_acme_challenge: set_acme_challenge_impl,
              unset_acme_challenge: unset_acme_challenge_impl,
              drop: drop_impl,
          };

          #[unsafe(no_mangle)]
          pub extern "C" fn _plugin_entrypoint() -> PluginHandle {
              let on_heap = Box::new(<$ty as Default>::default());
              PluginHandle::__new(Box::into_raw(on_heap) as *mut (), &VTABLE)
          }

          #[unsafe(no_mangle)]
          pub extern "C" fn _plugin_layout() -> u64 {
              $crate::plugin_fingerprint()
          }
      };
    };
}

pub const ABI_VERSION: u32 = 1;

// Hash of the sizes/alignments of every type that crosses the boundary.
pub fn plugin_fingerprint() -> u64 {
    use crate::context::{RawContext};
    use crate::error::ErrorInfo;
    use crate::ffi::{FfiString, FfiStr, FfiVec, FfiResult};
    use crate::value::{KVPair, PluginValue};
    use crate::handles::{PluginHandle};
    use crate::vtable::{PluginVTable, HostVTable};
    use std::mem::{align_of, size_of};

    let parts: [u64; 17] = [
        ABI_VERSION as u64,
        size_of::<FfiStr>() as u64,
        size_of::<FfiString>() as u64,
        size_of::<PluginValue>() as u64,
        align_of::<PluginValue>() as u64,
        size_of::<KVPair>() as u64,
        size_of::<FfiVec<PluginValue>>() as u64,
        size_of::<ErrorInfo>() as u64,
        size_of::<FfiResult<(), ErrorInfo>>() as u64,
        size_of::<FfiResult<PluginValue, ErrorInfo>>() as u64,
        size_of::<RawContext>() as u64,
        size_of::<HostVTable>() as u64,
        size_of::<PluginVTable>() as u64,
        size_of::<PluginHandle>() as u64,
        align_of::<PluginHandle>() as u64,
        size_of::<usize>() as u64,
        size_of::<bool>() as u64,
    ];

    // FNV-1a over the parts, curtesy of Claude
    parts.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, p| {
        (h ^ p).wrapping_mul(0x0000_0100_0000_01b3)
    })
}