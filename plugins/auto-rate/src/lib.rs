//! Auto-Rate: sets the rating in the Tag & Rate panel when the user assigns a tag that a rule names.
//!
//! Only the user's own assignment triggers it; removing the tag keeps the rating. `rules.rs` decides.

mod rules;

use std::sync::Mutex;

use rustcut_plugin_api::{Action, Context, Event, Issue, Level, Notice, Place, Plugin, SetRating, export_plugin};

pub use rules::{Message, Rule, Settings, findings, rating_for};

static SETTINGS: Mutex<Option<Settings>> = Mutex::new(None);

struct AutoRate;

impl Plugin for AutoRate {
    fn configure(settings: String) -> Result<(), String> {
        let settings: Settings = rustcut_plugin_api::parse_settings(&settings)?;
        rustcut_plugin_api::log(Level::Debug, &format!("{} rules", settings.rules.len()));
        *SETTINGS
            .lock()
            .map_err(|_poisoned| "the plugin's state is broken".to_owned())? = Some(settings);
        Ok(())
    }

    fn handle(event: Event, context: Context) -> Result<Vec<Action>, String> {
        let guard = SETTINGS
            .lock()
            .map_err(|_poisoned| "the plugin's state is broken".to_owned())?;
        let Some(settings) = guard.as_ref() else {
            return Ok(Vec::new());
        };
        let Event::TagAssigned(tag) = event else {
            return Ok(Vec::new());
        };
        let Some((stars, rule)) = rating_for(&settings.rules, tag.id, context.rating) else {
            return Ok(Vec::new());
        };
        let reason = format!("tag {}", tag.path_label());
        let mut actions = vec![Action::SetRating(SetRating {
            stars,
            reason: reason.clone(),
        })];
        let place = match rule.message {
            Message::None => None,
            Message::Toast => Some(Place::Toast),
            Message::Banner => Some(Place::Banner),
        };
        if let Some(place) = place.filter(|_| settings.show_messages) {
            let title = format!("Rating set to {stars} {}", if stars == 1 { "star" } else { "stars" });
            actions.push(Action::Notify(Notice::new(place, Level::Info, title, reason)));
        }
        Ok(actions)
    }

    fn check(settings: String) -> Vec<Issue> {
        match rustcut_plugin_api::parse_settings::<Settings>(&settings) {
            Ok(settings) => findings(&settings.rules)
                .into_iter()
                .map(|finding| Issue {
                    field: "rules".into(),
                    row: u32::try_from(finding.row).ok(),
                    level: Level::Warning,
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

export_plugin!(AutoRate);
