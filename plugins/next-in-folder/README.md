# Next in Folder

A RustCut plugin that opens the next or previous video of the open video's folder, in the order Dolphin shows that
folder.

Example: `/home/me/File D.mp4` is open, and Dolphin shows `/home/me` sorted by date, newest first. Ctrl+PageDown opens
the video Dolphin shows below `File D.mp4`. When Dolphin does not show the folder, the files go by name, A to Z.

## Settings

Settings > Plugins > Next in Folder:

| Setting | What it does |
|---|---|
| Next file, Previous file | The keys (Ctrl+PageDown, Ctrl+PageUp). RustCut refuses a key that one of its actions or another plugin has. |
| Buttons in the menu bar | Next and previous left of the messages bell (on). The File menu has both entries either way. |
| Skip files that have a job | A file that is the source of a queued job (any state) is passed over (on). |
| Hidden files | Like Dolphin (default), Skip or Include. |
| At the end of the folder | Stop (default) or Start over. |
| Use Dolphin's order, When | On; only when the folder is open in a Dolphin tab (default), whenever Dolphin runs, or always (Dolphin's stored order, Dolphin is never started). |
| Notifications | The opened file and its position (a bubble at the button), the end of the folder, skipped files, an order RustCut cannot rebuild (rating, tags, length: names A to Z instead). |

With sections that are not saved as a job, the key, the button and the menu entry must be held for the hold time of
Settings > General, since opening another video discards them. Only videos RustCut opens count. Needs RustCut's
plugin API 1.2.

## How Dolphin is asked

Over the session D-Bus: every Dolphin window answers whether it shows the folder in any tab
(`org.kde.dolphin.MainWindow.isUrlOpen`), and `org.freedesktop.FileManager1.SortOrderForUrl` gives the folder's sort.
Folders first, hidden files and "hidden files last" come from Dolphin's stored view properties. Dolphin's name order
(numbers by value, umlauts, case) is rebuilt in RustCut.

## Build

```bash
cargo xtask build next-in-folder
cargo xtask package next-in-folder
```

## License

MIT, see [LICENSE](LICENSE).
