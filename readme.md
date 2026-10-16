# ACME Sentry Plugin SDK
A small dependency-free Rust SDK for developing plugins and export them to shared libraries (`.so`) at runtime.

Plugin should be implemented in ordinary safe (and blocking) Rust against the `Plugin` trait. ACME Sentry will discover the shared libraries and load them as needed.  
ACME Sentry then receives data back as plain Rust data. The `unsafe` code and the C-ABI plumbing live in this SDK. This implementation allows for a few things:
- *std olny*: No crates.io dependencies, it builds fully offline. Dynamic loading is provided with a wrapper around `dlopen`/`dlsym`/`dlclose`.
- *Typed data in both directions*: A tagged union `PluginValue` carries bool, numbers, strings, lists & maps across the ABI boundary.
- *Structure errors ans panic containment*: A failing plugin returns `Err(code, message)`. A panicking plugin returns an error instead of a `SIGBUS`
- *Host callbacks*: Plugins can utilize logging in the ACME Sentry by calling `log(&str)` and cancellation polling via `should_cancel()` during execution.
- *Fail-closed loading*: Incompatible or damaged plugin files are rejected with a fingerprint error before any plugin code is run.

> **This is not a sandbox.** A native plugin runs inside the ACME Sentry process with the same privileges.
It protects against honest mistakes, not malicious plugins. See also [when not to use this](#when-not-to-use-this).

## Quick start
 
### A plugin
 
```toml
# Cargo.toml
[lib]
crate-type = ["cdylib"]
 
[dependencies]
plugin_sdk = { git  = "https://github.com/boijec/acme-sentry-plugin-sdk-rs" }
```
 
```rust
use plugin_sdk::{export_plugin, Context, Plugin, PluginError, Value};
 
#[derive(Default)]
struct Downloader;
 
impl Plugin for Downloader {
    fn name(&self) -> &str {
        "downloader"
    }
 
    // Check the basic configs that you need
    fn validate_config(&self, _ctx: &Context, config: &Value) -> Result<(), PluginError> {
        config
            .get("url")
            .and_then(Value::as_str)
            .map(|_| ())
            .ok_or_else(|| PluginError::invalid_config("missing required field: url"))
    }
 
    fn set_acme_challenge(&self, ctx: &Context, config: &Value) -> Result<Value, PluginError> {
        let url = config.get("url").and_then(Value::as_str).unwrap();
        ctx.log(&format!("setting acme challenge for {url}"));
 
        for i in 0..10 {
            if ctx.should_cancel() {
                return Err(PluginError::cancelled());
            }
            // ... real work ...
        }
        Ok(Value::map(vec![("bytes", Value::I64(1_048_576))]))
    }
    fn unset_acme_challenge(&self, ctx: &Context, config: &Value) -> Result<Value, PluginError> {
        let url = config.get("url").and_then(Value::as_str).unwrap();
        ctx.log(&format!("unsetting acme challenge for {url}"));
 
        for i in 0..10 {
            if ctx.should_cancel() {
                return Err(PluginError::cancelled());
            }
            // ... real work ...
        }
        Ok(Value::map(vec![("bytes", Value::I64(1_048_576))]))
    }
}
 
export_to_so!(Downloader);
```

No `unsafe`, no `extern "C"`: the macro generates the entry points, panic handling and vtable.

ACME Sentry will never see the `PluginValue` or `ErrorInfo` types since these are immediately deep copied (then dropped) into their own value objects (`HostValue` & `HostError`).
Both are safe to store and send across thread after plugin is unloaded.

## Concepts
 
### Data: `PluginValue` (plugin side) and `HostValue` (host side)
 
| Variant | Meaning |
|---|---|
| `Unit` | no value |
| `Bool(bool)` | |
| `I64(i64)` | |
| `F64(f64)` | `NaN`, infinities and `-0.0` survive the boundary |
| `Str(..)` | UTF-8, any length, embedded NULs preserved |
| `List(..)` | ordered values |
| `Map(..)` | ordered key/value pairs (duplicate keys allowed; `get` returns the first) |
 
Both types have `as_str`, `as_i64`, `as_bool`, `as_f64`, `as_list` and `get(key)` accessors that return `Option`.

### Plugin lifecycle
 
1. `load_plugin(path, policy)` opens the file, checks it, creates one plugin instance and reads its name.
2. `validate_config(ctx, config)` checks a config without side effects.
3. `set_acme_challenge(ctx, config)` set acme challenge token as a dns record in the plugin specified domain provider.
4. `unset_acme_challenge(ctx, config)` un sets the dns record in the plugin specified domain provider..
5. Dropping the `LoadedPlugin` destroys the instance, then optionally unloads the library.

### Errors
 
A plugin returns `PluginError::new(code, message)` (helpers: `invalid_config` = 400, `cancelled` = 499, `internal` = 500).
The host receives `HostError { code, message }`.
If a plugin panics, the host receives code `-1` and the message `"plugin panicked in execute"` (or `validate`), and the instance remains usable.

### The context
 
`Context` is how a running plugin talks back to the host: `log` and `should_cancel`. Cancellation is cooperative: the plugin has to poll it.

## How the boundary stays safe
 
| Hazard | What this crate does |
|---|---|
| Plugin built against a different ABI | Every plugin exports a layout fingerprint (hash of the size/alignment of every boundary type). It is checked before `_plugin_entrypoint` is ever called. |
| Truncated or half-copied `.so` | glibc's `dlopen` can SIGBUS the whole process on one. An ELF check verifies every loadable segment fits in the file before `dlopen` runs. |
| Use after `dlclose` | Plugin-allocated results are deep-copied into host-owned `HostValue`/`HostError` and the originals freed *inside the call*, so nothing with a destructor pointing into a library escapes. The handle is private and always dropped before the library. |
| Panic across `extern "C"` | Every exported entry point wraps the plugin call in `catch_unwind`. |
| Cross-allocator frees | Every owned buffer carries its own destructor, so memory is freed by the code that allocated it. |
| Wrong-typed data | Tagged unions with a pinned wire layout; a test fails if any discriminant or size changes. |

## Requirements and caveats
 
- **Same compiler, same version of this crate.** Host and plugins must be built with the same `rustc` and the same `acme_sentry_plugin_sdk_rs`. The fingerprint catches layout drift, not every codegen difference. Pin the toolchain with `rust-toolchain.toml`.
- **Do not set `panic = "abort"`** in any profile. Panic containment relies on unwinding.
- **Platform.** Developed and tested on Linux (x86_64). The Windows loader code compiles in the source but is untested; macOS is untested.
- **One caller at a time.** `LoadedPlugin` is `Send` but not `Sync`. Share one behind a `Mutex`, or give each thread its own.
- **A plugin's panic message** is printed to stderr by the plugin's own panic hook; the host only receives the generic error.

### Deploying plugin files
 
Never overwrite a loaded `.so` in place. `cp new.so plugins/x.so` truncates and rewrites the same file, and a running host will crash with SIGBUS on its next call into it.
Write to a temporary name and `rename` it over the old file (atomic), or `unlink` first. Have discovery ignore names that don't end in the final extension.

## When not to use this
 
Use something else if any of these apply:
- **Challenge type differential**: ACME Sentry currently supports DNS challenges only, if you intend on developing a plugin for a TLS-01 challenge, wait for support in ACME Sentry.

## WIP (ideas)
 
- Forward plugin panic messages through the host callback instead of the plugin's stderr.
- Optionally copy each plugin to a private temp file before loading, to remove the overwrite-in-place hazard entirely.
- A plugin config schema.
- A `describe()` function so ACME Sentry can discover a potential config schema.
