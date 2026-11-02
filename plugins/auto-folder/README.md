# Auto-Folder

A RustCut plugin that sets the target folder of Job Finalization from the folder a video comes from.

Example: a rule with the source `~/Videos/Holiday` and the target folder "Holiday 2026". Every video you open from
`~/Videos/Holiday` or a folder below it gets "Holiday 2026" as its target. If you choose another folder yourself, your
choice stays until the next video.

## Settings

Settings > Plugins > Auto-Folder:

| Setting | What it does |
|---|---|
| Rules | Source folder, Subfolders, Regex, Target folder, Message (none, toast, banner) per rule. |
| Apply the rules when | A new video is opened (default), or also when a queued job is loaded. |
| When no rule matches | Do nothing (default), or use a default target with its own message. |
| Show messages | The master switch of all messages; the Message column stays as it is. |

Which rule wins: the deepest matching folder; a folder rule beats a regular expression; among equals the upper rule.
The Settings page marks rules that never apply, broken regular expressions and rules a more specific one overrides;
RustCut itself marks target folders that no longer exist and source folders that are missing on disk.

## Build

```bash
cargo xtask build auto-folder
cargo xtask package auto-folder
```

The first command builds `target/wasm32-wasip2/release/auto_folder.wasm` (add `plugins/auto-folder` as a development
folder in RustCut to run it from there), the second `dist/auto-folder-<version>.rcplugin` for Settings > Plugins >
Install.

## License

MIT, see [LICENSE](LICENSE).
