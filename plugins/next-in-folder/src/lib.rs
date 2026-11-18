//! Next in Folder: opens the next or previous video of the open video's folder by a key, a File menu entry or a menu
//! bar button, in the order Dolphin shows that folder (RustCut asks Dolphin; names A to Z without it).
//!
//! RustCut steps through the folder (`step_folder`, plugin API 1.3: only the landing file reaches the plugin, however
//! large the folder) and opens it (`open_video`); this plugin decides the settings and what to tell.

mod walk;

use std::collections::BTreeSet;
use std::sync::Mutex;

use rustcut_plugin_api::{
    Action, Commands, Context, Event, FileManagerMode, FolderOrder, FolderRequest, FolderStep, FolderView, HiddenFiles,
    Issue, Level, Notice, OrderSource, Place, Plugin, export_plugin_with_commands,
};
use serde::Deserialize;
use walk::Direction;

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
    #[serde(flatten)]
    view: ViewSwitches,
}

/// What RustCut shows of the folder in the menu bar (API 1.4).
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(default)]
struct ViewSwitches {
    progress: bool,
    neighbors: bool,
}

impl Default for ViewSwitches {
    fn default() -> Self {
        Self {
            progress: true,
            neighbors: true,
        }
    }
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
            view: ViewSwitches::default(),
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
        // RustCut keeps the progress and the neighbor list up to date itself (API 1.4).
        let switches = parsed.view;
        let view = (switches.progress || switches.neighbors).then(|| FolderView {
            request: request(&parsed),
            progress: switches.progress,
            neighbors: switches.neighbors,
        });
        rustcut_plugin_api::set_folder_view(view);
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
        let folder = context
            .video
            .as_deref()
            .and_then(|v| v.rsplit_once('/'))
            .map_or("", |(dir, _)| dir)
            .to_owned();
        let forward = direction == Direction::Next;
        let step = rustcut_plugin_api::step_folder(
            request(&settings),
            forward,
            settings.skip_with_job,
            settings.at_end == "wrap",
        )?;
        Ok(after_step(&settings, direction, &step, &folder))
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

/// Opens the file the step landed on and says what the settings ask for. `folder`: the open video's folder (an
/// unknown order is told once per folder).
fn after_step(settings: &Settings, direction: Direction, step: &FolderStep, folder: &str) -> Vec<Action> {
    let mut actions = Vec::new();
    let order = &step.order;
    if order.source == OrderSource::NamesUnsupported && first_time(folder) {
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
    let Some(file) = &step.file else {
        let title = match direction {
            Direction::Next => "Last video in this folder",
            Direction::Previous => "First video in this folder",
        };
        let detail = if step.total == 0 {
            String::new()
        } else {
            format!("{} videos, {}.", step.total, order_words(order))
        };
        actions.extend(notice(&settings.msg_end, title, &detail));
        return actions;
    };
    rustcut_plugin_api::open_video(&file.path);
    let position = format!("{} of {} · {}", step.position, step.total, order_words(order));
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
fn order_words(order: &FolderOrder) -> String {
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

/// Whether the unknown order of `folder` is told for the first time.
fn first_time(folder: &str) -> bool {
    TOLD.lock().is_ok_and(|mut told| told.insert(folder.to_owned()))
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
