//! Which file comes next (pure): the step through the folder's list, files with a job passed over, the end of the
//! folder, and the words for the order that counted.

/// One file as the walk sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub name: String,
    pub has_job: bool,
}

/// Forward (next) or back (previous).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Next,
    Previous,
}

/// Where the walk landed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// The file to open; `None` at the end of the folder.
    pub target: Option<usize>,
    /// Files with a job that were passed over.
    pub skipped: usize,
    /// The walk went past the end and started over.
    pub wrapped: bool,
}

/// The next file after `current` in `direction`; files with a job are passed over when `skip_with_job`, and `wrap`
/// goes on from the other end (never back to `current` itself). Without `current` (the open video is not in the
/// list) the walk starts before the first (next) or after the last file (previous).
pub fn step(files: &[File], current: Option<usize>, direction: Direction, skip_with_job: bool, wrap: bool) -> Step {
    let len = files.len();
    let mut result = Step {
        target: None,
        skipped: 0,
        wrapped: false,
    };
    if len == 0 {
        return result;
    }
    let forward = direction == Direction::Next;
    // Positions as offsets from the start, so a wrap is one more lap of the same count.
    let start: isize = match current {
        Some(i) => isize::try_from(i).unwrap_or(0),
        None if forward => -1,
        None => isize::try_from(len).unwrap_or(0),
    };
    let lap = isize::try_from(len).unwrap_or(0);
    let mut position = start;
    for _ in 0..len {
        position += if forward { 1 } else { -1 };
        if !(0..lap).contains(&position) {
            if !wrap {
                return result;
            }
            result.wrapped = true;
            position = position.rem_euclid(lap);
        }
        if Some(position) == current.and_then(|c| isize::try_from(c).ok()) {
            return result;
        }
        let index = usize::try_from(position).unwrap_or(0);
        let Some(file) = files.get(index) else { return result };
        if skip_with_job && file.has_job {
            result.skipped += 1;
            continue;
        }
        result.target = Some(index);
        return result;
    }
    result
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

    fn files(jobs: &[bool]) -> Vec<File> {
        jobs.iter()
            .enumerate()
            .map(|(i, &has_job)| File {
                name: format!("{i}.mp4"),
                has_job,
            })
            .collect()
    }

    #[test]
    fn next_and_previous_stop_at_the_ends() {
        let list = files(&[false, false, false]);
        assert_eq!(step(&list, Some(1), Direction::Next, true, false).target, Some(2));
        assert_eq!(step(&list, Some(1), Direction::Previous, true, false).target, Some(0));
        assert_eq!(
            step(&list, Some(2), Direction::Next, true, false),
            Step {
                target: None,
                skipped: 0,
                wrapped: false
            }
        );
        assert_eq!(step(&list, Some(0), Direction::Previous, true, false).target, None);
    }

    #[test]
    fn files_with_a_job_are_passed_over_and_counted() {
        let list = files(&[false, true, true, false]);
        assert_eq!(
            step(&list, Some(0), Direction::Next, true, false),
            Step {
                target: Some(3),
                skipped: 2,
                wrapped: false
            }
        );
        assert_eq!(step(&list, Some(0), Direction::Next, false, false).target, Some(1));
        let all = files(&[false, true, true]);
        assert_eq!(
            step(&all, Some(0), Direction::Next, true, false),
            Step {
                target: None,
                skipped: 2,
                wrapped: false
            }
        );
    }

    #[test]
    fn wrapping_starts_over_but_never_returns_to_the_open_file() {
        let list = files(&[true, false, false]);
        assert_eq!(
            step(&list, Some(2), Direction::Next, true, true),
            Step {
                target: Some(1),
                skipped: 1,
                wrapped: true
            }
        );
        assert_eq!(
            step(&list, Some(1), Direction::Previous, true, true),
            Step {
                target: Some(2),
                skipped: 1,
                wrapped: true
            }
        );
        let alone = files(&[false]);
        assert_eq!(step(&alone, Some(0), Direction::Next, true, true).target, None);
    }

    #[test]
    fn an_open_file_outside_the_list_starts_at_an_end() {
        let list = files(&[false, false]);
        assert_eq!(step(&list, None, Direction::Next, true, false).target, Some(0));
        assert_eq!(step(&list, None, Direction::Previous, true, false).target, Some(1));
        assert_eq!(step(&[], None, Direction::Next, true, true).target, None);
    }

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
