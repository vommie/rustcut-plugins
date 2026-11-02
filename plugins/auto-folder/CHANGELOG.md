# Changelog of Auto-Folder

All notable changes of this plugin. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
the plugin's versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Rules from a source folder to a target folder of Settings > Target Folders, with subfolders on or off and regular
  expressions; the most specific rule wins (a deeper folder over a shallower one, a folder over a regular expression,
  the upper rule among equals).
- A message per rule (none, toast, banner) and a master switch for all of them.
- When the rules apply: when a new video is opened (default), or also when a queued job is loaded.
- Without a match: nothing happens (default), or a default target folder is used.
- Marks in Settings for broken regular expressions, rules without a source or target, duplicates and rules a more
  specific one overrides.
- Rules can be reordered by dragging their grip or with their move buttons (the upper rule wins among equals).
