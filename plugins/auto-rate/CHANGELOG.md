# Changelog of Auto-Rate

All notable changes of this plugin. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
the plugin's versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2026-11-09

### Added
- "Show mark" (Mark card, on by default): off sets the rating without the Auto-Rate icon at the rating bar. Settings of
  1.0 keep the icon.

### Changed
- Needs RustCut's plugin API 1.1.

## [1.0.0] - 2026-11-02

### Added

- Rules from a tag to a rating: assigning the tag in the Tag & Rate panel sets the rating; removing the tag keeps it.
- Per rule "Only if unrated" (on by default: a rating you gave stays) and a message (none, toast, banner), with a
  master switch for all messages.
- Marks in Settings for rules without a tag or rating and for duplicate tags.
- Rules can be reordered by dragging their grip or with their move buttons (the upper rule wins for a tag twice).
