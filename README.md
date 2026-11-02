# RustCut plugins

Plugins for [RustCut](https://github.com/vommie/rustcut), the video cutter: each one a small WebAssembly component that
reacts to what happens in RustCut and runs in RustCut's sandbox (no files, no network, a time and a memory limit).

| Plugin | What it does |
|---|---|
| [Auto-Folder](plugins/auto-folder) | Sets the target folder from the folder the video comes from. |
| [Auto-Rate](plugins/auto-rate) | Sets a rating when you assign a tag. |

## Install a plugin

Download `<plugin>-<version>.rcplugin` from the [releases](https://github.com/vommie/rustcut-plugins/releases) (or build
it, below), then in RustCut: Settings > Plugins > Install, or drop the file onto that page. The plugin's own settings
appear as a page below Plugins. Every release also has a `.sha256` file to compare with `sha256sum -c`.

## Build

Needs Rust (stable, through rustup) with the WebAssembly target; `rust-toolchain.toml` asks rustup for it:

```bash
rustup target add wasm32-wasip2
```

Then, from the repository's root:

| Command | What it does |
|---|---|
| `cargo xtask build auto-folder` | Builds `target/wasm32-wasip2/release/auto_folder.wasm`. |
| `cargo xtask package auto-folder` | Builds and packs `dist/auto-folder-<version>.rcplugin` and its `.sha256`. |
| `cargo xtask check` | Checks every plugin (manifest, versions, changelog, files), formatting, clippy (native and wasm), the tests and the dependencies' licenses (`cargo deny`). |
| `cargo xtask new auto-name --name "Auto-Name"` | Creates `plugins/auto-name` from the template, ready to build and test. |
| `cargo xtask release auto-folder` | Prepares a release: check, package, then prints the tag and GitHub release commands (`--publish` runs them). |

While developing, add the plugin's folder (`plugins/auto-folder`) in Settings > Plugins > Add development folder:
RustCut loads it straight from the build and reloads it whenever `cargo xtask build` writes a new one.

## Layout

```text
plugins/<id>/            one plugin: its own crate, version, CHANGELOG.md, README.md and LICENSE
  plugin.toml            who it is, which plugin API it needs, its settings (RustCut draws the page)
  src/lib.rs             the plugin; pure logic in its own modules, tested natively
  tests/host.rs          the built plugin in RustCut's real plugin host (crates/plugin-testkit)
crates/plugin-testkit/   builds a plugin for wasm32-wasip2 and runs it the way RustCut does
templates/plugin/        what `cargo xtask new` copies
xtask/                   the repository's tools
dist/                    built plugin files (not in git)
```

## Versions and releases

- Every plugin has its own [Semantic Version](https://semver.org) in `Cargo.toml` and `plugin.toml` (the same, checked)
  and its own `CHANGELOG.md` ([Keep a Changelog](https://keepachangelog.com/en/1.1.0/)): changes go under
  `## [Unreleased]`; a release renames it to `## [x.y.z] - YYYY-MM-DD`.
- A release is the tag `<id>-v<version>` (for example `auto-folder-v1.1.0`) with a GitHub release holding the plugin
  file and its checksum, made by `cargo xtask release <id> --publish`.
- The plugin API comes from RustCut's repository as `rustcut-plugin-api`, pinned to a tag `plugin-api-vX.Y.Z`
  (`Cargo.toml`, `[workspace.dependencies]`). A plugin declares the API it was built against in `plugin.toml`
  (`api = "1.0"`); RustCut runs every plugin of the same major version up to its own minor version.
- Commits follow [Conventional Commits](https://www.conventionalcommits.org) with the plugin's id as the scope:
  `feat(auto-folder): patterns with **`.

## Writing a plugin

`cargo xtask new` gives a plugin that builds, loads and passes its test. RustCut's `docs/plugins.md` describes the
events, actions, settings fields, limits and errors; the contract itself is
[`rustcut-plugin.wit`](https://github.com/vommie/rustcut/blob/master/crates/rustcut-plugin-api/wit/rustcut-plugin.wit).
See [CONTRIBUTING.md](CONTRIBUTING.md) for the rules of this repository.

## License

MIT, see [LICENSE](LICENSE). Each plugin carries the license in its folder and in its plugin file.
