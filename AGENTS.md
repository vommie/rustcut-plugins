# AGENTS.md

Rules for anyone, human or coding agent, who changes the RustCut plugins. Keep changes small and in line with these rules.

## Stack

- Rust only, compiled to WebAssembly components (`wasm32-wasip2`) for RustCut's plugin sandbox (no files, no network,
  time and memory limits). No other languages or runtimes.
- The plugin API comes from RustCut's repository (`rustcut-plugin-api`, and `rustcut-plugin-host` for the tests),
  pinned to a tag `plugin-api-vX.Y.Z` in `Cargo.toml`. A new API version is made in RustCut first; a WIT minor version
  only adds.

## Licenses

- Everything here is MIT; each plugin carries its `LICENSE`. Never copy code or assets from other projects; use a
  dependency or write it yourself, and check every new dependency's license first (`cargo deny`, part of
  `cargo xtask check`).

## Plugins

- One plugin per folder below `plugins/`, made with `cargo xtask new <id> --name "<Name>"`; folder, crate and
  `plugin.toml` id are the same. Every plugin builds on its own (`cargo xtask build <id>`).
- Versions per plugin: `Cargo.toml` and `plugin.toml` match; every user-visible change goes into the plugin's
  `CHANGELOG.md` under `## [Unreleased]`. A release is the tag `<id>-v<version>` made by `cargo xtask release <id>`.
- Edge cases belong to every plugin: settings that refer to things that may vanish (tags, target folders, folders on
  disk), broken input (regular expressions), duplicates and conflicts get a `check` finding with an explaining text;
  `handle` never fails on them.
- Every `plugin.toml` setting has a `help` text with an example.

## Code and checks

- US English everywhere: code, comments, messages, docs, commits.
- No `unwrap`, `expect`, `panic!` or fallible indexing outside tests (the lints enforce it). Pure logic in plain
  functions with native tests, plus at least one test in `tests/host.rs` that runs the built plugin in RustCut's host.
- `cargo xtask check` passes before every commit. `master` is always green; work on short branches with Conventional
  Commits (`feat(<plugin-id>): ...`) and merge them with `--no-ff`. Never force-push `master`.
