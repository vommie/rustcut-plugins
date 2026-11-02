//! The rules and which one a tag picks: pure, so it is tested natively.

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Message {
    #[default]
    None,
    Toast,
    Banner,
}

/// One line of the rules table, as RustCut stores it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Rule {
    #[serde(default)]
    pub tag: Option<u32>,
    #[serde(default)]
    pub stars: Option<u8>,
    #[serde(default = "yes")]
    pub only_if_none: bool,
    #[serde(default)]
    pub message: Message,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default = "yes")]
    pub show_messages: bool,
}

/// The rating to set when the user assigns `tag` to a clip rated `current`: the upper rule for the tag decides.
pub fn rating_for(rules: &[Rule], tag: u32, current: Option<u8>) -> Option<(u8, &Rule)> {
    let rule = rules.iter().find(|rule| rule.tag == Some(tag))?;
    let stars = rule.stars?.min(5);
    if rule.only_if_none && current.is_some() {
        return None;
    }
    (current != Some(stars)).then_some((stars, rule))
}

/// A finding about one rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub row: usize,
    pub text: String,
}

/// Rules without a tag or a rating, and rules whose tag an upper rule already has.
pub fn findings(rules: &[Rule]) -> Vec<Finding> {
    let mut out = Vec::new();
    for (row, rule) in rules.iter().enumerate() {
        let number = row + 1;
        match rule.tag {
            None => out.push(Finding {
                row,
                text: format!("Rule {number} has no tag, so it never applies."),
            }),
            Some(tag) => {
                if let Some(first) = rules.iter().take(row).position(|other| other.tag == Some(tag)) {
                    let first = first + 1;
                    out.push(Finding {
                        row,
                        text: format!("Rule {number} has the same tag as rule {first}; rule {first} wins, so this one never applies."),
                    });
                }
            }
        }
        if rule.stars.is_none() {
            out.push(Finding {
                row,
                text: format!("Rule {number} has no rating, so it changes nothing."),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(tag: u32, stars: u8, only_if_none: bool) -> Rule {
        Rule {
            tag: Some(tag),
            stars: Some(stars),
            only_if_none,
            message: Message::None,
        }
    }

    #[test]
    fn the_upper_rule_for_the_tag_decides() {
        let rules = [rule(7, 5, true), rule(7, 2, false), rule(8, 3, false)];
        assert_eq!(rating_for(&rules, 7, None).map(|(s, _)| s), Some(5));
        assert_eq!(rating_for(&rules, 7, Some(3)), None, "only if unrated");
        assert_eq!(rating_for(&rules, 8, Some(1)).map(|(s, _)| s), Some(3));
        assert_eq!(rating_for(&rules, 8, Some(3)), None, "already that rating");
        assert_eq!(rating_for(&rules, 9, None), None);
    }

    #[test]
    fn findings_name_empty_and_duplicate_rules() {
        let rules = [
            rule(7, 5, true),
            rule(7, 2, false),
            Rule {
                tag: None,
                stars: None,
                only_if_none: true,
                message: Message::None,
            },
        ];
        let found = findings(&rules);
        assert!(
            found
                .iter()
                .any(|f| f.row == 1 && f.text.contains("same tag as rule 1")),
            "{found:?}"
        );
        assert!(
            found.iter().any(|f| f.row == 2 && f.text.contains("no tag")),
            "{found:?}"
        );
        assert!(
            found.iter().any(|f| f.row == 2 && f.text.contains("no rating")),
            "{found:?}"
        );
        assert!(!found.iter().any(|f| f.row == 0));
    }
}
