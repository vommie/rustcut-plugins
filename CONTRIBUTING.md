# Contributing

Thank you for helping. The rules that keep the plugins safe and easy to maintain:

- **One plugin per folder** below `plugins/`, named after its id (lower case letters, digits and dashes). The crate,
  the folder and `plugin.toml`'s `id` have the same name; `cargo xtask check` enforces it.
- **Versions:** bump the plugin's version in `Cargo.toml` and `plugin.toml` together; describe every user-visible
  change in its `CHANGELOG.md` under `## [Unreleased]`.
- **Code:** no `unwrap`, `expect`, `panic!` or indexing that can fail outside tests (the lints in `Cargo.toml` enforce
  it); keep the logic in plain functions and test them natively, plus at least one test in `tests/host.rs` that runs
  the built plugin in RustCut's host.
- **Settings:** every field in `plugin.toml` has a `help` text that explains it with an example.
- **Language:** US English in code, comments, messages, docs and commit messages.
- **Commits:** [Conventional Commits](https://www.conventionalcommits.org) with the plugin's id as the scope
  (`fix(auto-rate): ...`); `chore:`, `docs:`, `ci:` for the repository itself.
- **Before a pull request:** `cargo xtask check` must pass.
