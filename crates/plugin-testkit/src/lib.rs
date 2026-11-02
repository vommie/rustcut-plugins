//! Runs a plugin the way RustCut runs it: built for `wasm32-wasip2`, loaded by RustCut's plugin host (the same
//! sandbox, limits and setting values), configured with its `plugin.toml` defaults plus the test's values.
//!
//! ```ignore
//! let mut plugin = plugin_testkit::plugin!(serde_json::json!({ "rules": [] }));
//! let actions = plugin.handle(&event, &context).unwrap();
//! ```

#![allow(
    clippy::panic,
    reason = "a test helper: a failed step must fail the test with its reason"
)]

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use rustcut_plugin_host::manifest::Manifest;
pub use rustcut_plugin_host::runtime::{CallError, Limits};
use rustcut_plugin_host::runtime::{Instance, Runtime};
pub use rustcut_plugin_host::types::{Action, Context, Event, Issue, Level, Notice, OpenedBy, Place, Tag, Target};
use rustcut_plugin_host::values;
use serde_json::{Map, Value};

/// Loads the calling plugin crate with `settings` (a JSON object; missing fields take their defaults).
#[macro_export]
macro_rules! plugin {
    ($settings:expr) => {
        $crate::load(
            ::std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
            env!("CARGO_PKG_NAME"),
            $settings,
        )
    };
}

/// A plugin running in RustCut's host.
pub struct Plugin {
    manifest: Manifest,
    instance: Instance,
}

/// Builds the crate `package` in `dir` (once per test run) and starts it with `settings`.
///
/// # Panics
/// When the plugin does not build, load or accept the settings: the test fails with the reason.
pub fn load(dir: &Path, package: &str, settings: Value) -> Plugin {
    let wasm = build(dir, package);
    let manifest = Manifest::read(dir).unwrap_or_else(|err| panic!("plugin.toml: {err}"));
    let mut loading = manifest.clone();
    loading.wasm = wasm;
    let loaded = runtime()
        .load(dir, loading)
        .unwrap_or_else(|err| panic!("the plugin does not load: {err}"));
    let mut instance = Instance::new(&loaded, Limits::default()).unwrap_or_else(|err| panic!("{err}"));
    let values = effective(&manifest, &settings);
    instance
        .configure(&Value::Object(values))
        .unwrap_or_else(|err| panic!("configure refused: {err}"));
    Plugin { manifest, instance }
}

impl Plugin {
    /// Hands the plugin an event, as RustCut would.
    ///
    /// # Errors
    /// The plugin's error, crash or timeout.
    pub fn handle(&mut self, event: &Event, context: &Context) -> Result<Vec<Action>, CallError> {
        self.instance.handle(event, context)
    }

    /// Checks settings, as the plugin's Settings page does while the user edits them.
    ///
    /// # Errors
    /// The plugin's crash or timeout.
    pub fn check(&mut self, settings: &Value) -> Result<Vec<Issue>, CallError> {
        let values = effective(&self.manifest, settings);
        self.instance.check(&Value::Object(values))
    }

    /// Hands the plugin new settings.
    ///
    /// # Errors
    /// The plugin refused them.
    pub fn configure(&mut self, settings: &Value) -> Result<(), CallError> {
        let values = effective(&self.manifest, settings);
        self.instance.configure(&Value::Object(values))
    }

    /// The lines the plugin wrote since the last call.
    pub fn log(&mut self) -> Vec<String> {
        self.instance.take_log().into_iter().map(|line| line.text).collect()
    }
}

fn effective(manifest: &Manifest, settings: &Value) -> Map<String, Value> {
    let stored = settings.as_object().cloned().unwrap_or_default();
    values::effective(manifest, &stored)
}

fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new(None).unwrap_or_else(|err| panic!("{err}")))
}

/// `cargo build --release --target wasm32-wasip2 -p <package>` into `target/testkit` (a target folder of its own:
/// the running `cargo test` holds the lock of the usual one).
fn build(dir: &Path, package: &str) -> PathBuf {
    static BUILT: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let root = dir.ancestors().nth(2).unwrap_or(dir);
    let target = root.join("target").join("testkit");
    let wasm = target
        .join("wasm32-wasip2/release")
        .join(format!("{}.wasm", package.replace('-', "_")));
    let mut built = BUILT.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if !built.iter().any(|p| p == package) {
        let status = std::process::Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
            .args([
                "build",
                "--release",
                "--target",
                "wasm32-wasip2",
                "--package",
                package,
                "--target-dir",
            ])
            .arg(&target)
            .current_dir(root)
            .status()
            .unwrap_or_else(|err| panic!("cargo does not run: {err}"));
        assert!(
            status.success(),
            "{package} must build for wasm32-wasip2 (rustup target add wasm32-wasip2)"
        );
        built.push(package.to_owned());
    }
    wasm
}
