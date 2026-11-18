# Changelog of Next in Folder

All notable changes of this plugin. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
the plugin's versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- RustCut takes the step through the folder (plugin API 1.3 `step-folder`): only the file it lands on reaches the
  plugin, so folders of hundreds of thousands of files cost no more time or memory than small ones. Needs RustCut
  0.0.115 or newer.

## [1.0.0] - 2026-11-17

### Added
- The plugin: Next File (Ctrl+PageDown) and Previous File (Ctrl+PageUp) open the next or previous video of the open
  video's folder, by key, File menu entry or menu bar button.
- Dolphin's order: when the folder is open in a Dolphin tab (or whenever Dolphin runs, or always from Dolphin's stored
  order), the files go in the order Dolphin shows them: name, size, date or type, either direction; else names A to Z.
- Files that already have a job are passed over (on by default); hidden files like Dolphin; stop or start over at the
  end of the folder.
- Notifications: the opened file and its position in a bubble at the button; the end of the folder, skipped files and
  an order RustCut cannot rebuild as nothing, a toast or a banner.
- Needs RustCut's plugin API 1.2.
