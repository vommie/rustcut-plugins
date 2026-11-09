//! Auto-Folder built for wasm32-wasip2 and run in RustCut's real plugin host.

#![allow(clippy::unwrap_used, clippy::indexing_slicing, reason = "test code")]

use plugin_testkit::{Action, Context, Event, Level, OpenedBy, Place, Target};
use serde_json::json;

fn opened(path: &str, by: OpenedBy) -> (Event, Context) {
    (
        Event::VideoOpened(by),
        Context {
            video: Some(path.into()),
            ..Context::default()
        },
    )
}

fn settings() -> serde_json::Value {
    json!({
        "rules": [
            { "source": "/videos", "target": 1, "message": "none" },
            { "source": "/videos/holiday", "target": 2, "message": "toast" },
            { "source": "^/mnt/usb[0-9]+/", "regex": true, "target": 3, "message": "banner" },
        ],
    })
}

#[test]
fn the_most_specific_rule_sets_the_target_with_its_message() {
    let mut plugin = plugin_testkit::plugin!(settings());
    let (event, context) = opened("/videos/holiday/2026/beach.mp4", OpenedBy::User);
    let actions = plugin.handle(&event, &context).unwrap();
    assert!(
        matches!(&actions[0], Action::SetTarget { target: Target::Folder(2), reason, mark: true } if reason == "rule /videos/holiday")
    );
    assert!(matches!(&actions[1], Action::Notify(n) if n.place == Place::Toast && n.level == Level::Info));

    let (event, context) = opened("/mnt/usb2/clips/a.mp4", OpenedBy::User);
    let actions = plugin.handle(&event, &context).unwrap();
    assert!(matches!(
        &actions[0],
        Action::SetTarget {
            target: Target::Folder(3),
            ..
        }
    ));
    assert!(matches!(&actions[1], Action::Notify(n) if n.place == Place::Banner));
}

#[test]
fn loaded_jobs_count_only_when_asked_and_messages_have_a_master_switch() {
    let mut plugin = plugin_testkit::plugin!(settings());
    let (event, context) = opened("/videos/holiday/a.mp4", OpenedBy::Job);
    assert!(plugin.handle(&event, &context).unwrap().is_empty());

    let mut with_jobs = settings();
    with_jobs["trigger"] = json!("also-jobs");
    with_jobs["show_messages"] = json!(false);
    plugin.configure(&with_jobs).unwrap();
    let actions = plugin.handle(&event, &context).unwrap();
    assert_eq!(
        actions.len(),
        1,
        "no message while the master switch is off: {actions:?}"
    );
}

#[test]
fn without_a_match_nothing_happens_unless_a_default_target_is_set() {
    let mut plugin = plugin_testkit::plugin!(settings());
    let (event, context) = opened("/elsewhere/a.mp4", OpenedBy::User);
    assert!(plugin.handle(&event, &context).unwrap().is_empty());
    let mut defaulting = settings();
    defaulting["when_no_match"] = json!("default");
    defaulting["default_target"] = json!(9);
    plugin.configure(&defaulting).unwrap();
    let actions = plugin.handle(&event, &context).unwrap();
    assert!(
        matches!(
            &actions[..],
            [Action::SetTarget {
                target: Target::Folder(9),
                ..
            }]
        ),
        "{actions:?}"
    );
}

#[test]
fn tags_and_a_missing_video_change_nothing() {
    let mut plugin = plugin_testkit::plugin!(settings());
    let tag = plugin_testkit::Tag {
        id: 1,
        name: "a".into(),
        path: vec!["a".into()],
    };
    let context = Context {
        video: Some("/videos/a.mp4".into()),
        ..Context::default()
    };
    assert!(plugin.handle(&Event::TagAssigned(tag), &context).unwrap().is_empty());
    assert!(
        plugin
            .handle(&Event::VideoOpened(OpenedBy::User), &Context::default())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn check_marks_the_rows_with_problems() {
    let mut plugin = plugin_testkit::plugin!(json!({}));
    let issues = plugin
        .check(&json!({ "rules": [{ "source": "(", "regex": true, "target": 1 }, { "source": "/a" }] }))
        .unwrap();
    assert!(
        issues.iter().any(|i| i.row == Some(0) && i.level == Level::Error),
        "{issues:?}"
    );
    assert!(
        issues
            .iter()
            .any(|i| i.row == Some(1) && i.message.contains("no target")),
        "{issues:?}"
    );
}

#[test]
fn the_target_already_set_stays_without_a_message() {
    let mut plugin = plugin_testkit::plugin!(settings());
    let (event, mut context) = opened("/videos/holiday/beach.mp4", OpenedBy::User);
    context.target_folder = Some(2);
    assert!(
        plugin.handle(&event, &context).unwrap().is_empty(),
        "folder 2 is the target already"
    );

    context.target_folder = Some(1);
    let actions = plugin.handle(&event, &context).unwrap();
    assert!(
        matches!(
            &actions[0],
            Action::SetTarget {
                target: Target::Folder(2),
                ..
            }
        ),
        "another folder is replaced as before: {actions:?}"
    );
    assert!(matches!(&actions[1], Action::Notify(n) if n.place == Place::Toast));

    // A loaded job's own folder is no folder of the list: the rule applies as before.
    let mut with_jobs = settings();
    with_jobs["trigger"] = json!("also-jobs");
    plugin.configure(&with_jobs).unwrap();
    let (event, mut context) = opened("/videos/holiday/beach.mp4", OpenedBy::Job);
    context.target_folder = None;
    assert_eq!(plugin.handle(&event, &context).unwrap().len(), 2);

    // The default target counts the same way.
    let mut default = settings();
    default["when_no_match"] = json!("default");
    default["default_target"] = json!(5);
    default["default_message"] = json!("toast");
    plugin.configure(&default).unwrap();
    let (event, mut context) = opened("/elsewhere/a.mp4", OpenedBy::User);
    context.target_folder = Some(5);
    assert!(plugin.handle(&event, &context).unwrap().is_empty());
    context.target_folder = Some(0);
    assert_eq!(plugin.handle(&event, &context).unwrap().len(), 2);
}
