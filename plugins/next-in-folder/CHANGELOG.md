# Changelog of Next in Folder

All notable changes of this plugin. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
the plugin's versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.3.0] - 2026-11-19

### Changed
- The neighbor list is a navigator of the whole folder now (RustCut 0.0.117): a search with wildcards like Dolphin
  (or a regular expression), pages of files with their number and size, and the keys to move through them.

### Added
- "Files per page" for the neighbor list (15 by default, 5 to 50).
- Needs RustCut's plugin API 1.5 (RustCut 0.0.117 or newer).

## [1.2.0] - 2026-11-18

### Added
- Progress in the menu bar (on by default): a flat bar between the buttons, or alone, shows where the open video
  stands in its folder (10 of 100 fills a tenth); the number shortens to what fits, the tooltip has the exact
  position, the percent, the order and the folder. RustCut keeps it up to date when a video opens and when the folder
  changes.
- Neighbor list (on by default): a right click on the buttons or the progress lists the five videos before and after
  the open one; a click opens one (held while sections are not saved). With buttons and progress off it has a button of
  its own.
- Needs RustCut's plugin API 1.4 (RustCut 0.0.116 or newer).

## [1.1.0] - 2026-11-18

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
