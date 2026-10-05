//! A small fzf-style fuzzy matcher used to filter the command list.

use std::cmp::Reverse;

const SCORE_MATCH: i64 = 16;
const BONUS_BOUNDARY: i64 = 8;
const BONUS_CONSECUTIVE: i64 = 8;
const PENALTY_GAP: i64 = 1;

/// A command that matched the query.
#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    /// Index of the command in the list that was filtered.
    pub index: usize,
    /// Higher is better.
    pub score: i64,
    /// Char indices of the matched characters, ascending.
    pub positions: Vec<usize>,
}

/// Filter `items` by `query`, best matches first.
///
/// Every whitespace-separated term must match as a subsequence, in any order.
/// Matching ignores case unless the term contains an uppercase letter. Ties,
/// and an empty query, keep the original order of `items`.
pub fn filter(query: &str, items: &[String]) -> Vec<Match> {
    let terms: Vec<&str> = query.split_whitespace().collect();

    let mut matches: Vec<Match> = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let text: Vec<char> = item.chars().collect();
            let mut score = 0;
            let mut positions = Vec::new();
            for term in &terms {
                let (term_score, term_positions) = match_term(term, &text)?;
                score += term_score;
                positions.extend(term_positions);
            }
            positions.sort_unstable();
            positions.dedup();
            Some(Match {
                index,
                score,
                positions,
            })
        })
        .collect();

    matches.sort_by_key(|m| Reverse(m.score));
    matches
}

/// Match a single term against `text`, returning its score and positions.
///
/// A forward scan finds where the leftmost match ends, then a backward scan
/// from there finds the tightest window ending at that point.
fn match_term(term: &str, text: &[char]) -> Option<(i64, Vec<usize>)> {
    let case_sensitive = term.chars().any(char::is_uppercase);
    let fold = |c: char| {
        if case_sensitive {
            c
        } else {
            c.to_lowercase().next().unwrap_or(c)
        }
    };
    let pattern: Vec<char> = term.chars().map(fold).collect();

    let mut matched = 0;
    let end = text.iter().position(|&c| {
        if fold(c) == pattern[matched] {
            matched += 1;
        }
        matched == pattern.len()
    })?;

    let mut positions = Vec::with_capacity(pattern.len());
    let mut remaining = pattern.len();
    for i in (0..=end).rev() {
        if fold(text[i]) == pattern[remaining - 1] {
            positions.push(i);
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        }
    }
    positions.reverse();

    Some((score(text, &positions), positions))
}

fn score(text: &[char], positions: &[usize]) -> i64 {
    let mut score = 0;
    let mut prev: Option<usize> = None;
    for &pos in positions {
        score += SCORE_MATCH;
        if pos == 0 || !text[pos - 1].is_alphanumeric() {
            score += BONUS_BOUNDARY;
        }
        match prev {
            Some(p) if pos == p + 1 => score += BONUS_CONSECUTIVE,
            Some(p) => score -= PENALTY_GAP * (pos - p - 1) as i64,
            None => {}
        }
        prev = Some(pos);
    }
    score
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn matched(query: &str, list: &[&str]) -> Vec<String> {
        let list = items(list);
        filter(query, &list)
            .into_iter()
            .map(|m| list[m.index].clone())
            .collect()
    }

    #[test]
    fn test_empty_query_keeps_everything_in_order() {
        let list = items(&["b", "a", "c"]);
        let result = filter("  ", &list);
        let indices: Vec<usize> = result.iter().map(|m| m.index).collect();
        assert_eq!(indices, vec![0, 1, 2]);
        assert!(result.iter().all(|m| m.positions.is_empty()));
    }

    #[test]
    fn test_subsequence_match_positions() {
        let list = items(&["git status"]);
        let result = filter("gst", &list);
        assert_eq!(result[0].positions, vec![0, 4, 5]);
    }

    #[test]
    fn test_non_matching_items_are_dropped() {
        assert_eq!(
            matched("xyz", &["git status", "cargo build"]),
            Vec::<String>::new()
        );
    }

    #[test]
    fn test_tight_matches_rank_first() {
        let result = matched("status", &["git stash && git log -u -s", "git status"]);
        assert_eq!(result, vec!["git status", "git stash && git log -u -s"]);
    }

    #[test]
    fn test_backward_scan_finds_tightest_window() {
        let list = items(&["git stash && git status"]);
        let result = filter("status", &list);
        assert_eq!(result[0].positions, (17..23).collect::<Vec<_>>());
    }

    #[test]
    fn test_all_terms_must_match_in_any_order() {
        let result = matched("status git", &["git status", "git stash", "status"]);
        assert_eq!(result, vec!["git status"]);
    }

    #[test]
    fn test_smart_case() {
        assert_eq!(matched("make", &["Makefile"]), vec!["Makefile"]);
        assert_eq!(matched("Make", &["makefile"]), Vec::<String>::new());
    }

    #[test]
    fn test_ties_keep_original_order() {
        let result = matched("ls", &["ls -a", "ls -l"]);
        assert_eq!(result, vec!["ls -a", "ls -l"]);
    }
}
