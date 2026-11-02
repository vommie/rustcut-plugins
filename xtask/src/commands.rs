//! new, build, package, release.

use std::path::{Path, PathBuf};
use std::process::Command;

use rustcut_plugin_host::manifest::{Manifest, valid_id};
use rustcut_plugin_host::package;
use sha2::{Digest, Sha256};

use crate::checks;

pub fn new(root: &Path, id: &str, name: &str) -> Result<(), String> {
    if !valid_id(id) {
        return Err(format!(
            "\"{id}\" is no plugin id: lower case letters, digits and dashes, starting with a letter"
        ));
    }
    let dir = root.join("plugins").join(id);
    if dir.exists() {
        return Err(format!("{} exists already", dir.display()));
    }
    let template = root.join("templates/plugin");
    copy_template(
        &template,
        &dir,
        &[("{{id}}", id), ("{{crate}}", &id.replace('-', "_")), ("{{name}}", name)],
    )?;
    println!("plugins/{id} is ready: cargo xtask build {id}, then add plugins/{id} as a development folder in RustCut");
    Ok(())
}

fn copy_template(from: &Path, to: &Path, replace: &[(&str, &str)]) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|err| format!("{}: {err}", to.display()))?;
    let entries = std::fs::read_dir(from).map_err(|err| format!("{}: {err}", from.display()))?;
    for entry in entries {
        let path = entry.map_err(|err| err.to_string())?.path();
        let Some(name) = path.file_name() else { continue };
        let target = to.join(name.to_string_lossy().trim_end_matches(".tmpl"));
        if path.is_dir() {
            copy_template(&path, &target, replace)?;
        } else {
            let mut text = std::fs::read_to_string(&path).map_err(|err| format!("{}: {err}", path.display()))?;
            for (key, value) in replace {
                text = text.replace(key, value);
            }
            std::fs::write(&target, text).map_err(|err| format!("{}: {err}", target.display()))?;
        }
    }
    Ok(())
}

pub fn build(root: &Path, id: Option<&str>) -> Result<(), String> {
    let ids = match id {
        Some(id) => vec![checks::plugin_dir(root, id)?.0],
        None => checks::plugins(root)?.into_iter().map(|(id, _)| id).collect(),
    };
    let mut args = vec![
        "build".to_owned(),
        "--release".to_owned(),
        "--target".to_owned(),
        "wasm32-wasip2".to_owned(),
    ];
    for id in &ids {
        args.extend(["--package".to_owned(), id.clone()]);
    }
    run(root, "cargo", &args)
}

/// Builds and packs the plugin; returns the plugin file.
pub fn package(root: &Path, id: &str) -> Result<PathBuf, String> {
    let (id, dir) = checks::plugin_dir(root, id)?;
    checks::plugin(&id, &dir)?;
    build(root, Some(&id))?;
    let manifest = Manifest::read(&dir).map_err(|err| err.to_string())?;
    let wasm = dir.join(&manifest.wasm);
    let dist = root.join("dist");
    std::fs::create_dir_all(&dist).map_err(|err| err.to_string())?;
    let file = dist.join(format!("{id}-{}.{}", manifest.version, package::EXTENSION));
    package::pack(&manifest, &wasm, &dir, &file).map_err(|err| err.to_string())?;
    // The file must read back like RustCut reads it.
    package::Package::read(&file).map_err(|err| format!("the packed file does not read back: {err}"))?;
    let bytes = std::fs::read(&file).map_err(|err| err.to_string())?;
    let sum = Sha256::digest(&bytes).iter().fold(String::new(), |mut hex, byte| {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
        hex
    });
    let name = file
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    std::fs::write(dist.join(format!("{name}.sha256")), format!("{sum}  {name}\n")).map_err(|err| err.to_string())?;
    Ok(file)
}

pub fn release(root: &Path, id: &str, publish: bool) -> Result<(), String> {
    let (id, dir) = checks::plugin_dir(root, id)?;
    let manifest = Manifest::read(&dir).map_err(|err| err.to_string())?;
    let version = manifest.version.to_string();
    let changelog = std::fs::read_to_string(dir.join("CHANGELOG.md")).map_err(|err| err.to_string())?;
    if !changelog.contains(&format!("## [{version}] - ")) {
        return Err(format!(
            "plugins/{id}/CHANGELOG.md has no \"## [{version}] - <date>\": rename [Unreleased] to the version first"
        ));
    }
    let status = output(root, "git", &["status", "--porcelain"])?;
    if !status.trim().is_empty() {
        return Err("the working tree is not clean: commit first".to_owned());
    }
    let tag = format!("{id}-v{version}");
    if !output(root, "git", &["tag", "--list", &tag])?.trim().is_empty() {
        return Err(format!("the tag {tag} exists already: bump the version"));
    }
    checks::check(root, false)?;
    let file = package(root, &id)?;
    let sum = PathBuf::from(format!("{}.sha256", file.display()));
    let notes = release_notes(&changelog, &version);
    let notes_file = root.join("dist").join(format!("{tag}.md"));
    std::fs::write(&notes_file, notes).map_err(|err| err.to_string())?;
    let title = format!("{} {version}", manifest.name);
    let steps: Vec<(&str, Vec<String>)> = vec![
        (
            "git",
            vec!["tag".into(), "-a".into(), tag.clone(), "-m".into(), title.clone()],
        ),
        ("git", vec!["push".into(), "origin".into(), tag.clone()]),
        (
            "gh",
            vec![
                "release".into(),
                "create".into(),
                tag,
                file.display().to_string(),
                sum.display().to_string(),
                "--title".into(),
                title,
                "--notes-file".into(),
                notes_file.display().to_string(),
            ],
        ),
    ];
    for (program, args) in &steps {
        if publish {
            run(root, program, args)?;
        } else {
            println!(
                "{program} {}",
                args.iter().map(|a| shell_word(a)).collect::<Vec<_>>().join(" ")
            );
        }
    }
    if !publish {
        println!("(nothing was published; run again with --publish to do it)");
    }
    Ok(())
}

/// The changelog's section of a version, without its heading.
fn release_notes(changelog: &str, version: &str) -> String {
    let head = format!("## [{version}]");
    let mut lines = changelog.lines().skip_while(|line| !line.starts_with(&head));
    lines.next();
    lines
        .take_while(|line| !line.starts_with("## ["))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
        + "\n"
}

fn shell_word(word: &str) -> String {
    if word.chars().all(|c| c.is_ascii_alphanumeric() || "-_./=:".contains(c)) {
        word.to_owned()
    } else {
        format!("'{}'", word.replace('\'', "'\\''"))
    }
}

pub fn run(root: &Path, program: &str, args: &[String]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .map_err(|err| format!("{program}: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} {} failed", args.join(" ")))
    }
}

fn output(root: &Path, program: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|err| format!("{program}: {err}"))?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_notes_are_the_version_section() {
        let log = "# Changelog\n\n## [Unreleased]\n\n## [1.1.0] - 2026-11-02\n\n### Added\n\n- Patterns.\n\n## [1.0.0] - 2026-11-01\n\n- First.\n";
        assert_eq!(release_notes(log, "1.1.0"), "### Added\n\n- Patterns.\n");
    }

    #[test]
    fn shell_words_are_quoted_only_when_needed() {
        assert_eq!(shell_word("auto-folder-v1.0.0"), "auto-folder-v1.0.0");
        assert_eq!(shell_word("Auto-Folder 1.0.0"), "'Auto-Folder 1.0.0'");
    }
}
