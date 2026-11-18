//! The direction of a step and the words for the order that counted (RustCut does the step itself, API 1.3).

/// Forward (next) or back (previous).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Next,
    Previous,
}

/// Dolphin's role id in words: `modificationtime` -> "Modified".
pub fn role_label(role: &str) -> String {
    match role {
        "text" | "" => "Name".to_owned(),
        "size" => "Size".to_owned(),
        "modificationtime" => "Modified".to_owned(),
        "creationtime" => "Created".to_owned(),
        "accesstime" => "Accessed".to_owned(),
        "type" => "Type".to_owned(),
        other => {
            let mut chars = other.chars();
            chars
                .next()
                .map_or_else(String::new, |first| first.to_uppercase().chain(chars).collect())
        }
    }
}

/// The direction in words for a role: "newest first", "Z to A", "largest first".
pub fn direction_label(role: &str, descending: bool) -> &'static str {
    match (role, descending) {
        ("modificationtime" | "creationtime" | "accesstime", false) => "oldest first",
        ("modificationtime" | "creationtime" | "accesstime", true) => "newest first",
        ("size", false) => "smallest first",
        ("size", true) => "largest first",
        (_, false) => "A to Z",
        (_, true) => "Z to A",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_in_words() {
        assert_eq!(
            format!(
                "{}, {}",
                role_label("modificationtime"),
                direction_label("modificationtime", true)
            ),
            "Modified, newest first"
        );
        assert_eq!(direction_label("text", false), "A to Z");
        assert_eq!(direction_label("size", true), "largest first");
        assert_eq!(role_label("rating"), "Rating");
    }
}
