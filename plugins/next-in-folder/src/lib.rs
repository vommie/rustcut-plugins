//! Next in Folder: opens the next or previous video of the open video's folder by a key, a File menu entry or a menu
//! bar button, in the order Dolphin shows that folder (RustCut asks Dolphin; names A to Z without it).
//!
//! RustCut lists the folder (`list_folder`, plugin API 1.2) and opens the chosen file (`open_video`); this plugin
//! decides which one and what to tell.

mod walk;

use std::collections::BTreeSet;
use std::sync::Mutex;

use rustcut_plugin_api::{
    Action, Commands, Context, Event, FileManagerMode, FolderListing, FolderRequest, HiddenFiles, Issue, Level, Notice,
    OrderSource, Place, Plugin, export_plugin_with_commands,
};
use serde::Deserialize;
use walk::{Direction, File};

/// The settings of `plugin.toml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct Settings {
    skip_with_job: bool,
    hidden: String,
    at_end: String,
    use_dolphin: bool,
    dolphin_when: String,
    msg_opened: String,
    msg_position: String,
    msg_end: String,
    msg_skipped: String,
    msg_unsupported: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            skip_with_job: true,
            hidden: "like-dolphin".into(),
            at_end: "stop".into(),
            use_dolphin: true,
            dolphin_when: "open-tab".into(),
            msg_opened: "bubble".into(),
            msg_position: "with-opened".into(),
            msg_end: "toast".into(),
            msg_skipped: "toast".into(),
            msg_unsupported: "toast".into(),
        }
    }
}

static SETTINGS: Mutex<Option<Settings>> = Mutex::new(None);
/// The folders whose unknown Dolphin order was told already (once per folder while RustCut runs).
static TOLD: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

struct NextInFolder;

impl Plugin for NextInFolder {
    fn configure(settings: String) -> Result<(), String> {
        let parsed: Settings = rustcut_plugin_api::parse_settings(&settings)?;
        *SETTINGS
            .lock()
            .map_err(|err| format!("the settings are locked: {err}"))? = Some(parsed);
        Ok(())
    }

    fn handle(_event: Event, _context: Context) -> Result<Vec<Action>, String> {
        Ok(Vec::new())
    }

    fn check(_settings: String) -> Vec<Issue> {
        Vec::new()
    }
}

impl Commands for NextInFolder {
    fn run(command: String, context: Context) -> Result<Vec<Action>, String> {
        let direction = match command.as_str() {
            "next-file" => Direction::Next,
            "previous-file" => Direction::Previous,
            other => return Err(format!("unknown command {other}")),
        };
        let settings = SETTINGS
            .lock()
            .map_err(|err| format!("the settings are locked: {err}"))?
            .clone()
            .unwrap_or_default();
        if context.video.is_none() {
            return Ok(notice(&settings.msg_end, "No video is open", "").into_iter().collect());
        }
        let listing = rustcut_plugin_api::list_folder(request(&settings))?;
        Ok(walk_folder(&settings, direction, &listing))
    }
}

fn request(settings: &Settings) -> FolderRequest {
    let file_manager = if settings.use_dolphin {
        match settings.dolphin_when.as_str() {
            "running" => FileManagerMode::Running,
            "always" => FileManagerMode::Always,
            _ => FileManagerMode::OpenTab,
        }
    } else {
        FileManagerMode::Off
    };
    let hidden = match settings.hidden.as_str() {
        "skip" => HiddenFiles::Skip,
        "show" => HiddenFiles::Show,
        _ => HiddenFiles::LikeFileManager,
    };
    FolderRequest {
        file_manager,
        hidden,
        videos_only: true,
    }
}

/// Picks the file, opens it and says what the settings ask for.
fn walk_folder(settings: &Settings, direction: Direction, listing: &FolderListing) -> Vec<Action> {
    let files: Vec<File> = listing
        .files
        .iter()
        .map(|f| File {
            name: f.name.clone(),
            has_job: f.has_job,
        })
        .collect();
    let current = listing.current.and_then(|i| usize::try_from(i).ok());
    let step = walk::step(
        &files,
        current,
        direction,
        settings.skip_with_job,
        settings.at_end == "wrap",
    );
    let mut actions = Vec::new();
    let order = &listing.order;
    if order.source == OrderSource::NamesUnsupported && first_time(listing) {
        let detail = format!(
            "Dolphin sorts this folder by {}; Next in Folder goes by name, A to Z.",
            walk::role_label(&order.role)
        );
        actions.extend(notice(&settings.msg_unsupported, "Dolphin's order is unknown", &detail));
    }
    if order.source == OrderSource::NamesFailed {
        rustcut_plugin_api::log(Level::Warning, &format!("Dolphin could not be asked: {}", order.detail));
    }
    if step.skipped > 0 {
        let noun = if step.skipped == 1 { "file" } else { "files" };
        let detail = format!("{} {noun} with a job in the queue passed over.", step.skipped);
        actions.extend(notice(&settings.msg_skipped, "Skipped files with a job", &detail));
    }
    let Some(index) = step.target else {
        let title = match direction {
            Direction::Next => "Last video in this folder",
            Direction::Previous => "First video in this folder",
        };
        let detail = if files.is_empty() {
            String::new()
        } else {
            format!("{} videos, {}.", files.len(), order_words(listing))
        };
        actions.extend(notice(&settings.msg_end, title, &detail));
        return actions;
    };
    let Some(file) = listing.files.get(index) else {
        return actions;
    };
    rustcut_plugin_api::open_video(&file.path);
    let position = format!("{} of {} · {}", index + 1, files.len(), order_words(listing));
    let position = if step.wrapped {
        format!("{position} · started over")
    } else {
        position
    };
    match (settings.msg_opened.as_str(), settings.msg_position.as_str()) {
        ("bubble", "with-opened") => rustcut_plugin_api::bubble(&format!("{} · {position}", file.name)),
        ("bubble", _) => rustcut_plugin_api::bubble(&file.name),
        (_, "with-opened") => rustcut_plugin_api::bubble(&position),
        _ => {}
    }
    if matches!(settings.msg_position.as_str(), "toast" | "banner") {
        actions.extend(notice(&settings.msg_position, &file.name, &position));
    }
    actions
}

/// "Dolphin: Modified, newest first" or "names A to Z".
fn order_words(listing: &FolderListing) -> String {
    let order = &listing.order;
    if order.source == OrderSource::FileManager {
        format!(
            "Dolphin: {}, {}",
            walk::role_label(&order.role),
            walk::direction_label(&order.role, order.descending)
        )
    } else {
        "names A to Z".to_owned()
    }
}

/// Whether the unknown order of this listing's folder is told for the first time.
fn first_time(listing: &FolderListing) -> bool {
    let folder = listing
        .files
        .first()
        .and_then(|f| f.path.rsplit_once('/').map(|(dir, _)| dir.to_owned()))
        .unwrap_or_default();
    TOLD.lock().is_ok_and(|mut told| told.insert(folder))
}

/// A message at `place` ("none", "toast", "banner"); `None` for nothing.
fn notice(place: &str, title: &str, detail: &str) -> Option<Action> {
    let place = match place {
        "toast" => Place::Toast,
        "banner" => Place::Banner,
        _ => return None,
    };
    Some(Action::Notify(Notice::new(place, Level::Info, title, detail)))
}

export_plugin_with_commands!(NextInFolder);
