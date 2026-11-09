//! Auto-Rate built for wasm32-wasip2 and run in RustCut's real plugin host.

#![allow(clippy::unwrap_used, clippy::indexing_slicing, reason = "test code")]

use plugin_testkit::{Action, Context, Event, Place, Tag};
use serde_json::json;

fn tag(id: u32) -> Tag {
    Tag {
        id,
        name: "Great".into(),
        path: vec!["Treasure chest".into(), "Great".into()],
    }
}

fn rated(rating: Option<u8>) -> Context {
    Context {
        video: Some("/v/a.mp4".into()),
        rating,
        ..Context::default()
    }
}

fn settings() -> serde_json::Value {
    json!({ "rules": [
        { "tag": 7, "stars": 5, "message": "toast" },
        { "tag": 8, "stars": 2, "only_if_none": false },
    ] })
}

#[test]
fn assigning_a_rule_tag_sets_its_rating() {
    let mut plugin = plugin_testkit::plugin!(settings());
    let actions = plugin.handle(&Event::TagAssigned(tag(7)), &rated(None)).unwrap();
    assert!(
        matches!(&actions[0], Action::SetRating { stars: 5, reason, mark: true } if reason == "tag Treasure chest › Great"),
        "{actions:?}"
    );
    assert!(matches!(&actions[1], Action::Notify(n) if n.place == Place::Toast && n.title == "Rating set to 5 stars"));
}

#[test]
fn an_existing_rating_stays_unless_the_rule_replaces_it() {
    let mut plugin = plugin_testkit::plugin!(settings());
    assert!(
        plugin
            .handle(&Event::TagAssigned(tag(7)), &rated(Some(3)))
            .unwrap()
            .is_empty()
    );
    let actions = plugin.handle(&Event::TagAssigned(tag(8)), &rated(Some(4))).unwrap();
    assert!(
        matches!(&actions[..], [Action::SetRating { stars: 2, .. }]),
        "{actions:?}"
    );
}

#[test]
fn removing_the_tag_keeps_the_rating_and_other_tags_do_nothing() {
    let mut plugin = plugin_testkit::plugin!(settings());
    assert!(
        plugin
            .handle(&Event::TagRemoved(tag(7)), &rated(Some(5)))
            .unwrap()
            .is_empty()
    );
    assert!(
        plugin
            .handle(&Event::TagAssigned(tag(99)), &rated(None))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn the_master_switch_hides_the_messages() {
    let mut quiet = settings();
    quiet["show_messages"] = json!(false);
    let mut plugin = plugin_testkit::plugin!(quiet);
    let actions = plugin.handle(&Event::TagAssigned(tag(7)), &rated(None)).unwrap();
    assert_eq!(actions.len(), 1);
}

#[test]
fn check_marks_duplicate_and_empty_rules() {
    let mut plugin = plugin_testkit::plugin!(json!({}));
    let issues = plugin
        .check(&json!({ "rules": [{ "tag": 7 }, { "tag": 7 }, {}] }))
        .unwrap();
    assert!(issues.iter().any(|i| i.row == Some(1)), "{issues:?}");
    assert!(issues.iter().any(|i| i.row == Some(2)), "{issues:?}");
}

#[test]
fn show_mark_off_sets_the_rating_without_the_mark() {
    let mut quiet = settings();
    quiet["show_mark"] = json!(false);
    let mut plugin = plugin_testkit::plugin!(quiet);
    let actions = plugin.handle(&Event::TagAssigned(tag(7)), &rated(None)).unwrap();
    assert!(
        matches!(
            &actions[0],
            Action::SetRating {
                stars: 5,
                mark: false,
                ..
            }
        ),
        "{actions:?}"
    );
    assert!(
        matches!(&actions[1], Action::Notify(n) if n.place == Place::Toast),
        "messages stay: {actions:?}"
    );

    plugin.configure(&settings()).unwrap();
    let actions = plugin.handle(&Event::TagAssigned(tag(7)), &rated(None)).unwrap();
    assert!(
        matches!(&actions[0], Action::SetRating { mark: true, .. }),
        "without the value (settings of 1.0) the mark shows: {actions:?}"
    );
}
