//! `cargo xtask check`: what every plugin must have, then formatting, clippy and the tests.

use std::path::{Path, PathBuf};

use rustcut_plugin_host::manifest::Manifest;

use crate::commands::run;

/// The plugin API (`MAJOR.MINOR`) a crate in `dir` builds against: the tag of its own `rustcut-plugin-api` dependency,
/// else the workspace's (`plugin-api-v1.2.0` -> `1.2`).
pub fn kit_api(root: &Path, dir: &Path) -> Result<String, String> {
    let read = |path: PathBuf| -> Result<toml::Table, String> {
        std::fs::read_to_string(&path)
            .map_err(|err| format!("{}: {err}", path.display()))
            .and_then(|text| {
                text.parse()
                    .map_err(|err: toml::de::Error| format!("{}: {err}", path.display()))
            })
    };
    let tag_of = |table: Option<&toml::Value>| {
        table
            .and_then(|d| d.get("rustcut-plugin-api"))
            .and_then(|d| d.get("tag"))
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
    };
    let own = dir.join("Cargo.toml");
    let own = if own.is_file() {
        tag_of(read(own)?.get("dependencies"))
    } else {
        None
    };
    let workspace = read(root.join("Cargo.toml"))?;
    let tag = own
        .or_else(|| tag_of(workspace.get("workspace").and_then(|w| w.get("dependencies"))))
        .ok_or("no rustcut-plugin-api tag in the plugin's or the workspace's Cargo.toml")?;
    let version = tag
        .strip_prefix("plugin-api-v")
        .ok_or_else(|| format!("{tag} is no plugin-api-vX.Y.Z tag"))?;
    let version = semver::Version::parse(version).map_err(|err| format!("{tag}: {err}"))?;
    Ok(format!("{}.{}", version.major, version.minor))
}

/// Every plugin folder: (id, folder), sorted.
pub fn plugins(root: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    let entries = std::fs::read_dir(root.join("plugins")).map_err(|err| format!("plugins/: {err}"))?;
    let mut found: Vec<(String, PathBuf)> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.join("plugin.toml").is_file())
        .map(|path| {
            (
                path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                path,
            )
        })
        .collect();
    found.sort();
    Ok(found)
}

pub fn plugin_dir(root: &Path, id: &str) -> Result<(String, PathBuf), String> {
    let dir = root.join("plugins").join(id);
    if dir.join("plugin.toml").is_file() {
        Ok((id.to_owned(), dir))
    } else {
        Err(format!("there is no plugins/{id}/plugin.toml"))
    }
}

/// The rules of one plugin folder.
pub fn plugin(id: &str, dir: &Path) -> Result<(), String> {
    let manifest = Manifest::read(dir).map_err(|err| format!("plugins/{id}: {err}"))?;
    if manifest.id != id {
        return Err(format!(
            "plugins/{id}: the folder's name must be the plugin's id \"{}\"",
            manifest.id
        ));
    }
    let cargo: toml::Table = std::fs::read_to_string(dir.join("Cargo.toml"))
        .map_err(|err| err.to_string())
        .and_then(|text| text.parse().map_err(|err: toml::de::Error| err.to_string()))?;
    let package = cargo.get("package").and_then(toml::Value::as_table);
    let name = package.and_then(|p| p.get("name")).and_then(toml::Value::as_str);
    let version = package.and_then(|p| p.get("version")).and_then(toml::Value::as_str);
    if name != Some(id) {
        return Err(format!("plugins/{id}/Cargo.toml: the package must be called \"{id}\""));
    }
    if version != Some(manifest.version.to_string().as_str()) {
        return Err(format!(
            "plugins/{id}: Cargo.toml and plugin.toml must have the same version ({})",
            manifest.version
        ));
    }
    let expected = PathBuf::from(format!(
        "../../target/wasm32-wasip2/release/{}.wasm",
        id.replace('-', "_")
    ));
    if manifest.wasm != expected {
        return Err(format!(
            "plugins/{id}/plugin.toml: wasm must be \"{}\"",
            expected.display()
        ));
    }
    let built = kit_api(&dir.join("../.."), dir)?;
    if manifest.api != built {
        return Err(format!(
            "plugins/{id}: plugin.toml says api \"{}\", but the plugin builds against plugin-api {built} (its Cargo.toml or the \
             workspace's rustcut-plugin-api tag): a build imports exactly that API and loads in no older RustCut",
            manifest.api
        ));
    }
    if !manifest.api_compatible() {
        return Err(format!(
            "plugins/{id}: api {} is newer than the plugin host this repository builds against",
            manifest.api
        ));
    }
    for file in ["README.md", "CHANGELOG.md", "LICENSE"] {
        if !dir.join(file).is_file() {
            return Err(format!("plugins/{id}: {file} is missing"));
        }
    }
    let changelog = std::fs::read_to_string(dir.join("CHANGELOG.md")).map_err(|err| err.to_string())?;
    let version = manifest.version.to_string();
    if !changelog.contains("## [Unreleased]") && !changelog.contains(&format!("## [{version}]")) {
        return Err(format!(
            "plugins/{id}/CHANGELOG.md documents neither [Unreleased] nor [{version}]"
        ));
    }
    Ok(())
}

pub fn check(root: &Path, fast: bool) -> Result<(), String> {
    for (id, dir) in plugins(root)? {
        plugin(&id, &dir)?;
        println!("plugins/{id}: ok");
    }
    let args = |list: &[&str]| list.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    run(root, "cargo", &args(&["fmt", "--all", "--check"]))?;
    if fast {
        return Ok(());
    }
    run(
        root,
        "cargo",
        &args(&["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"]),
    )?;
    run(
        root,
        "cargo",
        &args(&[
            "clippy",
            "--target",
            "wasm32-wasip2",
            "--release",
            "--",
            "-D",
            "warnings",
        ]),
    )?;
    run(root, "cargo", &args(&["test", "--workspace"]))?;
    if std::process::Command::new("cargo")
        .args(["deny", "--version"])
        .output()
        .is_ok_and(|out| out.status.success())
    {
        run(
            root,
            "cargo",
            &args(&["deny", "--all-features", "check", "licenses", "sources", "bans"]),
        )
    } else {
        Err("cargo-deny is missing (sudo pacman -S cargo-deny): the licenses and sources are not checked".to_owned())
    }
}
