# Auto-Rate

A RustCut plugin that sets a rating when you assign a tag in the Tag & Rate panel.

Example: a rule with the tag `Treasure chest › Great` and 5 stars. When you assign that tag, the clip gets 5 stars,
unless it has a rating already ("Only if unrated", on by default). Removing the tag again keeps the rating.

## Settings

Settings > Plugins > Auto-Rate:

| Setting | What it does |
|---|---|
| Rules | Tag, Rating, Only if unrated, Message (none, toast, banner) per rule. |
| Show messages | The master switch of all messages; the Message column stays as it is. |

Only your own assignment of the tag counts, never a loaded job's tags. The Settings page marks rules without a tag or
rating and duplicate tags; RustCut itself marks tags that no longer exist in RustView's tag tree.

## Build

```bash
cargo xtask build auto-rate
cargo xtask package auto-rate
```

## License

MIT, see [LICENSE](LICENSE).
