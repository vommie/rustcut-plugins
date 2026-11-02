//! The rules and how a video's folder picks one: pure, so it is tested natively.

use std::path::{Component, Path, PathBuf};

use regex::Regex;
use serde::Deserialize;

/// Where a message about a rule goes.
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
    pub source: String,
    #[serde(default = "yes")]
    pub subfolders: bool,
    #[serde(default)]
    pub regex: bool,
    #[serde(default)]
    pub target: Option<u32>,
    #[serde(default)]
    pub message: Message,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Trigger {
    #[default]
    NewVideo,
    AlsoJobs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NoMatch {
    #[default]
    Nothing,
    Default,
}

/// The plugin's settings (the fields of plugin.toml).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub trigger: Trigger,
    #[serde(default)]
    pub when_no_match: NoMatch,
    #[serde(default)]
    pub default_target: Option<u32>,
    #[serde(default)]
    pub default_message: Message,
    #[serde(default = "yes")]
    pub show_messages: bool,
}

/// A rule ready to match: its folder made plain, or its regular expression compiled.
#[derive(Debug, Clone)]
pub enum Matcher {
    Folder {
        path: PathBuf,
        subfolders: bool,
    },
    Regex(Regex),
    /// Empty or broken: never matches.
    Never,
}

impl Matcher {
    pub fn new(rule: &Rule) -> Self {
        let source = rule.source.trim();
        if source.is_empty() {
            Self::Never
        } else if rule.regex {
            Regex::new(source).map_or(Self::Never, Self::Regex)
        } else {
            Self::Folder {
                path: plain(Path::new(source)),
                subfolders: rule.subfolders,
            }
        }
    }

    /// How specific a match of `folder` is: `None` = no match; a folder rule counts its depth (+1000, so it beats
    /// any regular expression), a regular expression counts 0.
    pub fn score(&self, folder: &Path) -> Option<usize> {
        match self {
            Self::Folder { path, subfolders } => {
                let hit = if *subfolders {
                    folder.starts_with(path)
                } else {
                    folder == path
                };
                hit.then(|| 1000 + path.components().count())
            }
            Self::Regex(regex) => regex.is_match(&folder.to_string_lossy()).then_some(0),
            Self::Never => None,
        }
    }
}

/// The index of the rule that wins for a video in `folder`: the highest score, among equals the upper rule.
pub fn best(matchers: &[Matcher], folder: &Path) -> Option<usize> {
    let folder = plain(folder);
    let mut winner: Option<(usize, usize)> = None;
    for (index, matcher) in matchers.iter().enumerate() {
        if let Some(score) = matcher.score(&folder)
            && winner.is_none_or(|(_, best)| score > best)
        {
            winner = Some((index, score));
        }
    }
    winner.map(|(index, _)| index)
}

/// A path without `.`, `..` and trailing slashes (RustCut hands over sources with `~` already expanded).
pub fn plain(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

/// A finding about one rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub row: usize,
    pub error: bool,
    pub text: String,
}

/// What is wrong with or worth knowing about the rules: broken expressions, empty sources and targets, duplicates,
/// and rules that a more specific one overrides for part of their folders.
pub fn findings(rules: &[Rule]) -> Vec<Finding> {
    let mut out = Vec::new();
    let matchers: Vec<Matcher> = rules.iter().map(Matcher::new).collect();
    for (row, rule) in rules.iter().enumerate() {
        let number = row + 1;
        let source = rule.source.trim();
        if source.is_empty() {
            out.push(Finding {
                row,
                error: false,
                text: format!("Rule {number} has no source folder, so it never matches."),
            });
            continue;
        }
        if rule.regex
            && let Err(err) = Regex::new(source)
        {
            let reason = err.to_string().lines().last().unwrap_or_default().trim().to_owned();
            out.push(Finding {
                row,
                error: true,
                text: format!("Rule {number}: the regular expression is not valid ({reason})."),
            });
        }
        if rule.target.is_none() {
            out.push(Finding {
                row,
                error: false,
                text: format!("Rule {number} has no target folder, so it changes nothing."),
            });
        }
        let Some(Matcher::Folder { path, subfolders }) = matchers.get(row) else {
            continue;
        };
        for (other_row, other) in matchers.iter().enumerate() {
            let Matcher::Folder {
                path: other_path,
                subfolders: other_sub,
            } = other
            else {
                continue;
            };
            if other_row == row {
                continue;
            }
            let other_number = other_row + 1;
            if other_path == path && other_sub == subfolders {
                if other_row < row {
                    out.push(Finding {
                        row,
                        error: false,
                        text: format!("Rule {number} has the same source as rule {other_number}; rule {other_number} wins, so this one never applies."),
                    });
                }
            } else if *subfolders && other_path != path && other_path.starts_with(path) {
                out.push(Finding {
                    row,
                    error: false,
                    text: format!(
                        "For videos in {}, rule {other_number} is more specific and wins over this rule.",
                        other_path.display()
                    ),
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(source: &str, subfolders: bool, regex: bool, target: u32) -> Rule {
        Rule {
            source: source.into(),
            subfolders,
            regex,
            target: Some(target),
            message: Message::None,
        }
    }

    fn pick(rules: &[Rule], folder: &str) -> Option<usize> {
        let matchers: Vec<Matcher> = rules.iter().map(Matcher::new).collect();
        best(&matchers, Path::new(folder))
    }

    #[test]
    fn the_deepest_folder_rule_wins_whatever_the_order() {
        let rules = [rule("/v", true, false, 1), rule("/v/holiday", true, false, 2)];
        assert_eq!(pick(&rules, "/v/holiday/2026"), Some(1));
        assert_eq!(pick(&rules, "/v/other"), Some(0));
        assert_eq!(pick(&rules, "/w"), None);
        let reversed = [rules[1].clone(), rules[0].clone()];
        assert_eq!(pick(&reversed, "/v/holiday/2026"), Some(0));
    }

    #[test]
    fn subfolders_off_matches_only_the_folder_itself() {
        let rules = [rule("/v/holiday/", false, false, 1)];
        assert_eq!(pick(&rules, "/v/holiday"), Some(0));
        assert_eq!(pick(&rules, "/v/holiday/2026"), None);
        assert_eq!(pick(&rules, "/v/holiday-old"), None, "a prefix of the name is no match");
    }

    #[test]
    fn regular_expressions_lose_to_folder_rules_and_the_upper_one_wins() {
        let rules = [
            rule("^/mnt/usb[0-9]+/", false, true, 1),
            rule("usb", false, true, 2),
            rule("/mnt/usb1", true, false, 3),
        ];
        assert_eq!(pick(&rules, "/mnt/usb1/clips"), Some(2));
        assert_eq!(pick(&rules, "/mnt/usb2/clips"), Some(0));
    }

    #[test]
    fn broken_and_empty_rules_never_match() {
        let rules = [rule("(", false, true, 1), rule("  ", true, false, 2)];
        assert_eq!(pick(&rules, "/x"), None);
    }

    #[test]
    fn paths_are_compared_plain() {
        let rules = [rule("/v/./a/../holiday", true, false, 1)];
        assert_eq!(pick(&rules, "/v/holiday/"), Some(0));
    }

    #[test]
    fn findings_name_broken_duplicate_and_overridden_rules() {
        let rules = [
            rule("/v", true, false, 1),
            rule("/v/holiday", true, false, 2),
            rule("/v/holiday", true, false, 3),
            rule("([", false, true, 4),
            Rule {
                target: None,
                ..rule("/w", true, false, 0)
            },
            rule("", true, false, 5),
        ];
        let texts: Vec<(usize, bool, String)> =
            findings(&rules).into_iter().map(|f| (f.row, f.error, f.text)).collect();
        assert!(
            texts
                .iter()
                .any(|(row, _, t)| *row == 0 && t.contains("rule 2 is more specific")),
            "{texts:#?}"
        );
        assert!(
            texts
                .iter()
                .any(|(row, _, t)| *row == 2 && t.contains("same source as rule 2")),
            "{texts:#?}"
        );
        assert!(
            texts
                .iter()
                .any(|(row, error, t)| *row == 3 && *error && t.contains("not valid")),
            "{texts:#?}"
        );
        assert!(
            texts.iter().any(|(row, _, t)| *row == 4 && t.contains("no target")),
            "{texts:#?}"
        );
        assert!(
            texts.iter().any(|(row, _, t)| *row == 5 && t.contains("no source")),
            "{texts:#?}"
        );
        assert!(!texts.iter().any(|(row, _, _)| *row == 1), "{texts:#?}");
    }
}
