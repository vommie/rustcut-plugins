//! Auto-Folder: sets the target folder of Job Finalization from the folder a video comes from.
//!
//! The rules are a table in the plugin's Settings page (plugin.toml); `rules.rs` decides which one wins.

mod rules;

use std::path::Path;
use std::sync::Mutex;

use rustcut_plugin_api::{
    Action, Context, Event, Issue, Level, Notice, OpenedBy, Place, Plugin, SetTarget, Target, export_plugin,
};

pub use rules::{Matcher, Message, NoMatch, Rule, Settings, Trigger, best, findings};

/// The settings and the compiled rules, from the last `configure`.
struct State {
    settings: Settings,
    matchers: Vec<Matcher>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

struct AutoFolder;

impl Plugin for AutoFolder {
    fn configure(settings: String) -> Result<(), String> {
        let settings: Settings = rustcut_plugin_api::parse_settings(&settings)?;
        let matchers = settings.rules.iter().map(Matcher::new).collect();
        let count = settings.rules.len();
        *STATE
            .lock()
            .map_err(|_poisoned| "the plugin's state is broken".to_owned())? = Some(State { settings, matchers });
        rustcut_plugin_api::log(Level::Debug, &format!("{count} rules"));
        Ok(())
    }

    fn handle(event: Event, context: Context) -> Result<Vec<Action>, String> {
        let guard = STATE
            .lock()
            .map_err(|_poisoned| "the plugin's state is broken".to_owned())?;
        let Some(state) = guard.as_ref() else {
            return Ok(Vec::new());
        };
        Ok(decide(state, &event, &context, rustcut_plugin_api::target_folder()))
    }

    fn check(settings: String) -> Vec<Issue> {
        match rustcut_plugin_api::parse_settings::<Settings>(&settings) {
            Ok(settings) => findings(&settings.rules)
                .into_iter()
                .map(|finding| Issue {
                    field: "rules".into(),
                    row: u32::try_from(finding.row).ok(),
                    level: if finding.error { Level::Error } else { Level::Warning },
                    message: finding.text,
                })
                .collect(),
            Err(err) => vec![Issue {
                field: "rules".into(),
                row: None,
                level: Level::Error,
                message: err,
            }],
        }
    }
}

export_plugin!(AutoFolder);

/// What to do for an event: the winning rule's target (or the default target) and its message; nothing when that
/// folder is the target already (`current`, the id RustCut reports), so the message does not repeat.
fn decide(state: &State, event: &Event, context: &Context, current: Option<u32>) -> Vec<Action> {
    let settings = &state.settings;
    let applies = match event {
        Event::VideoOpened(OpenedBy::User) => true,
        Event::VideoOpened(OpenedBy::Job) => settings.trigger == Trigger::AlsoJobs,
        Event::TagAssigned(_) | Event::TagRemoved(_) => false,
    };
    let Some(video) = context.video.as_deref().filter(|_| applies) else {
        return Vec::new();
    };
    let Some(folder) = Path::new(video).parent() else {
        return Vec::new();
    };
    let (target, message, reason) = match best(&state.matchers, folder).and_then(|i| settings.rules.get(i)) {
        Some(rule) => (rule.target, rule.message, format!("rule {}", rule.source.trim())),
        None if settings.when_no_match == NoMatch::Default => (
            settings.default_target,
            settings.default_message,
            "no rule matched".to_owned(),
        ),
        None => return Vec::new(),
    };
    let Some(target) = target else { return Vec::new() };
    if current == Some(target) {
        rustcut_plugin_api::log(
            Level::Debug,
            &format!("the target is already folder {target} ({reason})"),
        );
        return Vec::new();
    }
    let mut actions = vec![Action::SetTarget(SetTarget {
        target: Target::Folder(target),
        reason: reason.clone(),
    })];
    let place = match message {
        Message::None => None,
        Message::Toast => Some(Place::Toast),
        Message::Banner => Some(Place::Banner),
    };
    if let Some(place) = place.filter(|_| settings.show_messages) {
        actions.push(Action::Notify(Notice::new(
            place,
            Level::Info,
            "Target folder set",
            reason,
        )));
    }
    actions
}
