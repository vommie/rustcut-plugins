//! Next in Folder built for wasm32-wasip2 and run in RustCut's real plugin host, with a fake folder.

#![allow(clippy::unwrap_used, clippy::indexing_slicing, reason = "test code")]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use plugin_testkit::{
    Action, Context, FileManagerMode, FolderFile, FolderListing, FolderOrder, FolderRequest, HiddenFiles, Navigator,
    OrderSource, Place,
};
use serde_json::json;

/// `/v/a.mp4` .. `/v/d.mp4` in Dolphin's order "modified, newest first", the open video is `b.mp4`; `c.mp4` has a job.
struct Folder {
    source: OrderSource,
    role: &'static str,
    asked: Mutex<Vec<FolderRequest>>,
}

impl Folder {
    fn new(source: OrderSource, role: &'static str) -> Arc<Self> {
        Arc::new(Self {
            source,
            role,
            asked: Mutex::new(Vec::new()),
        })
    }
}

impl Navigator for Folder {
    fn list_folder(&self, video: &Path, request: FolderRequest) -> Result<FolderListing, String> {
        self.asked.lock().unwrap().push(request);
        assert_eq!(video, Path::new("/v/b.mp4"));
        let files = ["a.mp4", "b.mp4", "c.mp4", "d.mp4"]
            .iter()
            .map(|name| FolderFile {
                path: PathBuf::from("/v").join(name),
                name: (*name).into(),
                has_job: *name == "c.mp4",
            })
            .collect();
        let order = FolderOrder {
            source: self.source,
            role: self.role.into(),
            descending: true,
            detail: String::new(),
        };
        Ok(FolderListing {
            files,
            current: Some(1),
            order,
        })
    }
}

fn open() -> Context {
    Context {
        video: Some("/v/b.mp4".into()),
        ..Context::default()
    }
}

#[test]
fn next_passes_files_with_a_job_and_says_where_it_landed() {
    let mut plugin = plugin_testkit::plugin!(json!({}));
    let folder = Folder::new(OrderSource::FileManager, "modificationtime");
    let outcome = plugin.run_command("next-file", &open(), Some(folder.clone())).unwrap();
    assert_eq!(outcome.open.as_deref(), Some(Path::new("/v/d.mp4")));
    assert_eq!(outcome.bubbles, ["d.mp4 · 4 of 4 · Dolphin: Modified, newest first"]);
    assert!(
        matches!(&outcome.actions[..], [Action::Notify(n)] if n.place == Place::Toast && n.title == "Skipped files with a job"),
        "{:?}",
        outcome.actions
    );
    let asked = folder.asked.lock().unwrap()[0];
    assert_eq!(asked.file_manager, FileManagerMode::OpenTab);
    assert_eq!(asked.hidden, HiddenFiles::LikeFileManager);
    assert!(asked.videos_only);
}

#[test]
fn previous_and_the_end_of_the_folder() {
    let mut plugin = plugin_testkit::plugin!(json!({ "msg_skipped": "none" }));
    let folder = Folder::new(OrderSource::NamesNotOpen, "text");
    let outcome = plugin
        .run_command("previous-file", &open(), Some(folder.clone()))
        .unwrap();
    assert_eq!(outcome.open.as_deref(), Some(Path::new("/v/a.mp4")));
    assert_eq!(outcome.bubbles, ["a.mp4 · 1 of 4 · names A to Z"]);

    plugin
        .configure(&json!({ "skip_with_job": false, "msg_end": "banner" }))
        .unwrap();
    let outcome = plugin.run_command("next-file", &open(), Some(folder)).unwrap();
    assert_eq!(outcome.open.as_deref(), Some(Path::new("/v/c.mp4")));
    assert!(outcome.actions.is_empty());
}

#[test]
fn an_order_rustcut_cannot_rebuild_is_told_once_and_settings_reach_the_request() {
    let mut plugin = plugin_testkit::plugin!(json!({
        "dolphin_when": "always", "hidden": "skip", "at_end": "wrap", "msg_opened": "none", "msg_position": "toast"
    }));
    let folder = Folder::new(OrderSource::NamesUnsupported, "rating");
    let outcome = plugin.run_command("next-file", &open(), Some(folder.clone())).unwrap();
    assert!(outcome.bubbles.is_empty());
    let titles: Vec<&str> = outcome
        .actions
        .iter()
        .filter_map(|a| {
            if let Action::Notify(n) = a {
                Some(n.title.as_str())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        titles,
        ["Dolphin's order is unknown", "Skipped files with a job", "d.mp4"]
    );
    let again = plugin.run_command("next-file", &open(), Some(folder.clone())).unwrap();
    assert!(
        !again
            .actions
            .iter()
            .any(|a| matches!(a, Action::Notify(n) if n.title == "Dolphin's order is unknown"))
    );
    let asked = folder.asked.lock().unwrap()[0];
    assert_eq!(
        (asked.file_manager, asked.hidden),
        (FileManagerMode::Always, HiddenFiles::Skip)
    );

    plugin.configure(&json!({ "use_dolphin": false })).unwrap();
    plugin.run_command("next-file", &open(), Some(folder.clone())).unwrap();
    assert_eq!(folder.asked.lock().unwrap()[2].file_manager, FileManagerMode::Off);
    assert!(plugin.run_command("other", &open(), Some(folder)).is_err());
}

/// A folder of `n` videos, the open one in the middle, prepared before the call.
struct Large(FolderListing);

impl Large {
    fn new(n: usize) -> Arc<Self> {
        let files = (0..n)
            .map(|i| {
                let name = format!("Holiday 2026 [GoPro] - clip {i:06} take {}.mp4", i % 7);
                FolderFile {
                    path: PathBuf::from("/media/videos/[test]/test").join(&name),
                    name,
                    has_job: i % 3 == 0,
                }
            })
            .collect();
        let order = FolderOrder {
            source: OrderSource::FileManager,
            role: "text".into(),
            descending: false,
            detail: String::new(),
        };
        Arc::new(Self(FolderListing {
            files,
            current: u32::try_from(n / 2).ok(),
            order,
        }))
    }
}

impl Navigator for Large {
    fn list_folder(&self, _video: &Path, _request: FolderRequest) -> Result<FolderListing, String> {
        Ok(self.0.clone())
    }
}

#[test]
fn a_folder_of_500_000_files_is_quick_and_fits_the_memory_limit() {
    let mut plugin = plugin_testkit::plugin!(json!({}));
    for n in [1_000, 100_000, 500_000] {
        let folder = Large::new(n);
        let context = Context {
            video: Some("/media/videos/[test]/test/x.mp4".into()),
            ..Context::default()
        };
        let started = std::time::Instant::now();
        let outcome = plugin.run_command("next-file", &context, Some(folder)).unwrap();
        let took = started.elapsed();
        println!("{n} files: {took:?}");
        assert!(outcome.open.is_some());
        assert!(took < std::time::Duration::from_millis(900), "{n} files took {took:?}");
    }
}
