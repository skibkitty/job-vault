use std::collections::HashSet;

use crate::normalization::TextNormalizer;

pub const DEFAULT_SIMILARITY_THRESHOLD: f64 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentChange {
    pub change_type: ChangeType,
    pub content: String,
    pub previous_content: Option<String>,
    pub old_index: Option<usize>,
    pub new_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiffResult {
    pub changes: Vec<SegmentChange>,
    pub unchanged_count: usize,
}

pub struct TextDiffEngine {
    similarity_threshold: f64,
}

struct PendingChange {
    index: usize,
    text: String,
    normalized: String,
}

impl TextDiffEngine {
    pub fn new() -> Self {
        Self {
            similarity_threshold: DEFAULT_SIMILARITY_THRESHOLD,
        }
    }

    pub fn with_similarity_threshold(threshold: f64) -> Self {
        Self {
            similarity_threshold: threshold,
        }
    }

    pub fn split_paragraphs(text: &str) -> Vec<String> {
        let mut paragraphs = Vec::new();
        let mut current: Vec<&str> = Vec::new();
        let normalized = TextNormalizer::normalize_line_breaks(text);

        for line in normalized.lines() {
            if line.trim().is_empty() {
                if !current.is_empty() {
                    paragraphs.push(current.join("\n").trim().to_string());
                    current.clear();
                }
            } else {
                current.push(line);
            }
        }

        if !current.is_empty() {
            paragraphs.push(current.join("\n").trim().to_string());
        }

        paragraphs
    }

    pub fn split_sentences(text: &str) -> Vec<String> {
        let normalized = TextNormalizer::normalize_line_breaks(text);
        let mut sentences = Vec::new();
        let mut current = String::new();
        let mut chars = normalized.chars().peekable();

        while let Some(ch) = chars.next() {
            current.push(ch);

            if ch == '.' || ch == '!' || ch == '?' {
                while let Some(&next) = chars.peek() {
                    if next == '"' || next == '\'' || next == ')' || next == ']' {
                        current.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }

                let mut probe = chars.clone();
                let is_boundary = match probe.find(|c| !c.is_whitespace()) {
                    Some(next) => next.is_uppercase(),
                    None => true,
                };

                if !is_boundary {
                    continue;
                }

                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
                current.clear();
            }
        }

        let rest = current.trim().to_string();
        if !rest.is_empty() {
            sentences.push(rest);
        }

        sentences
    }

    pub fn split_bullets(text: &str) -> Vec<String> {
        let normalized = TextNormalizer::normalize_line_breaks(text);
        let mut bullets = Vec::new();
        let mut current: Option<String> = None;

        for line in normalized.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                if let Some(c) = current.take() {
                    bullets.push(c);
                }
                continue;
            }

            if Self::is_bullet_marker(trimmed) {
                if let Some(c) = current.take() {
                    bullets.push(c);
                }
                let body = Self::bullet_body(trimmed);
                current = Some(body);
            } else if let Some(c) = current.as_mut() {
                c.push(' ');
                c.push_str(trimmed);
            }
        }

        if let Some(c) = current.take() {
            let trimmed = c.trim().to_string();
            if !trimmed.is_empty() {
                bullets.push(trimmed);
            }
        }

        bullets
    }

    fn is_bullet_marker(line: &str) -> bool {
        let trimmed = line.trim_start();
        let mut chars = trimmed.chars();

        match chars.next() {
            Some('-') | Some('*') | Some('•') | Some('·') => true,
            Some(c) if c.is_ascii_digit() => {
                let mut rest = trimmed.trim_start_matches(|ch: char| ch.is_ascii_digit());
                if let Some(first) = rest.chars().next() {
                    first == '.' || first == ')' || first == ']'
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn bullet_body(line: &str) -> String {
        let trimmed = line.trim_start();
        match trimmed.chars().next() {
            Some(first) if matches!(first, '-' | '*' | '•' | '·') => {
                let body = trimmed[first.len_utf8()..].trim_start().to_string();
                return body;
            }
            _ => {}
        }

        let mut rest = trimmed.trim_start_matches(|ch: char| ch.is_ascii_digit());
        if let Some(first) = rest.chars().next() {
            if first == '.' || first == ')' || first == ']' {
                rest = &rest[first.len_utf8()..];
            }
        }
        rest.trim().to_string()
    }

    pub fn diff_bullets(&self, old_text: &str, new_text: &str) -> DiffResult {
        self.diff_segments(&Self::split_bullets(old_text), &Self::split_bullets(new_text))
    }

    pub fn similarity(a_normalized: &str, b_normalized: &str) -> f64 {
        let a_tokens: HashSet<&str> = a_normalized
            .split_whitespace()
            .map(|t| t.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|t| !t.is_empty())
            .collect();
        let b_tokens: HashSet<&str> = b_normalized
            .split_whitespace()
            .map(|t| t.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|t| !t.is_empty())
            .collect();

        if a_tokens.is_empty() && b_tokens.is_empty() {
            return 1.0;
        }

        let intersection = a_tokens.intersection(&b_tokens).count();

        let a_count = a_tokens.len();
        let b_count = b_tokens.len();

        if a_count + b_count == 0 {
            return 1.0;
        }

        2.0 * intersection as f64 / (a_count + b_count) as f64
    }

    pub fn diff_paragraphs(&self, old_text: &str, new_text: &str) -> DiffResult {
        self.diff_segments(&Self::split_paragraphs(old_text), &Self::split_paragraphs(new_text))
    }

    pub fn diff_sentences(&self, old_text: &str, new_text: &str) -> DiffResult {
        self.diff_segments(&Self::split_sentences(old_text), &Self::split_sentences(new_text))
    }

    pub fn diff_texts(&self, old_text: &str, new_text: &str) -> DiffResult {
        self.diff_sentences(old_text, new_text)
    }

    pub fn diff_segments(&self, old_segments: &[String], new_segments: &[String]) -> DiffResult {
        let old_normalized: Vec<String> = old_segments
            .iter()
            .map(|s| TextNormalizer::normalize_for_comparison(s))
            .collect();
        let new_normalized: Vec<String> = new_segments
            .iter()
            .map(|s| TextNormalizer::normalize_for_comparison(s))
            .collect();

        let table = Self::lcs_table(&old_normalized, &new_normalized);

        let mut changes = Vec::new();
        let mut unchanged_count = 0usize;
        let mut removed: Vec<PendingChange> = Vec::new();
        let mut added: Vec<PendingChange> = Vec::new();

        let mut i = 0usize;
        let mut j = 0usize;

        while i < old_segments.len() && j < new_segments.len() {
            if old_normalized[i] == new_normalized[j] {
                Self::flush_run(&removed, &added, self.similarity_threshold, &mut changes);
                removed.clear();
                added.clear();
                unchanged_count += 1;
                i += 1;
                j += 1;
            } else if table[i + 1][j] >= table[i][j + 1] {
                removed.push(PendingChange {
                    index: i,
                    text: old_segments[i].clone(),
                    normalized: old_normalized[i].clone(),
                });
                i += 1;
            } else {
                added.push(PendingChange {
                    index: j,
                    text: new_segments[j].clone(),
                    normalized: new_normalized[j].clone(),
                });
                j += 1;
            }
        }

        while i < old_segments.len() {
            removed.push(PendingChange {
                index: i,
                text: old_segments[i].clone(),
                normalized: old_normalized[i].clone(),
            });
            i += 1;
        }

        while j < new_segments.len() {
            added.push(PendingChange {
                index: j,
                text: new_segments[j].clone(),
                normalized: new_normalized[j].clone(),
            });
            j += 1;
        }

        Self::flush_run(&removed, &added, self.similarity_threshold, &mut changes);

        DiffResult {
            changes,
            unchanged_count,
        }
    }

    fn flush_run(
        removed: &[PendingChange],
        added: &[PendingChange],
        threshold: f64,
        changes: &mut Vec<SegmentChange>,
    ) {
        let mut matched = vec![false; added.len()];

        for r in removed {
            let mut best: Option<(usize, f64)> = None;
            for (a_index, a) in added.iter().enumerate() {
                if matched[a_index] {
                    continue;
                }
                let score = Self::similarity(&r.normalized, &a.normalized);
                if score >= threshold && best.map_or(true, |(_, s)| score > s) {
                    best = Some((a_index, score));
                }
            }

            match best {
                Some((a_index, _)) => {
                    matched[a_index] = true;
                    changes.push(SegmentChange {
                        change_type: ChangeType::Modified,
                        content: added[a_index].text.clone(),
                        previous_content: Some(r.text.clone()),
                        old_index: Some(r.index),
                        new_index: Some(added[a_index].index),
                    });
                }
                None => {
                    changes.push(SegmentChange {
                        change_type: ChangeType::Removed,
                        content: r.text.clone(),
                        previous_content: None,
                        old_index: Some(r.index),
                        new_index: None,
                    });
                }
            }
        }

        for (a_index, a) in added.iter().enumerate() {
            if !matched[a_index] {
                changes.push(SegmentChange {
                    change_type: ChangeType::Added,
                    content: a.text.clone(),
                    previous_content: None,
                    old_index: None,
                    new_index: Some(a.index),
                });
            }
        }
    }

    fn lcs_table(a: &[String], b: &[String]) -> Vec<Vec<usize>> {
        let mut table = vec![vec![0usize; b.len() + 1]; a.len() + 1];

        for i in (0..a.len()).rev() {
            for j in (0..b.len()).rev() {
                table[i][j] = if a[i] == b[j] {
                    table[i + 1][j + 1] + 1
                } else {
                    table[i + 1][j].max(table[i][j + 1])
                };
            }
        }

        table
    }
}

impl Default for TextDiffEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_paragraphs_on_blank_line() {
        let result = TextDiffEngine::split_paragraphs("First paragraph.\n\nSecond paragraph.");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "First paragraph.");
        assert_eq!(result[1], "Second paragraph.");
    }

    #[test]
    fn split_paragraphs_multiple_blank_lines() {
        let result =
            TextDiffEngine::split_paragraphs("First.\n\n\n\nSecond.\n \nThird.");
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn split_paragraphs_single_paragraph_no_blank_lines() {
        let result = TextDiffEngine::split_paragraphs("Line one.\nLine two.");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "Line one.\nLine two.");
    }

    #[test]
    fn split_paragraphs_trims_whitespace() {
        let result = TextDiffEngine::split_paragraphs("   First.   \n\n   Second.   ");
        assert_eq!(result[0], "First.");
        assert_eq!(result[1], "Second.");
    }

    #[test]
    fn split_paragraphs_empty_and_whitespace_only_input() {
        assert!(TextDiffEngine::split_paragraphs("").is_empty());
        assert!(TextDiffEngine::split_paragraphs("  \n\n  \n").is_empty());
    }

    #[test]
    fn split_paragraphs_handles_crlf() {
        let result = TextDiffEngine::split_paragraphs("First.\r\n\r\nSecond.");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "First.");
        assert_eq!(result[1], "Second.");
    }

    #[test]
    fn split_sentences_basic_periods() {
        let result = TextDiffEngine::split_sentences("One sentence. Two sentences. Three.");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "One sentence.");
        assert_eq!(result[2], "Three.");
    }

    #[test]
    fn split_sentences_exclamation_and_question_marks() {
        let result = TextDiffEngine::split_sentences("What a role! Are you interested? Yes.");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "What a role!");
        assert_eq!(result[1], "Are you interested?");
    }

    #[test]
    fn split_sentences_does_not_split_decimals() {
        let result = TextDiffEngine::split_sentences("The salary is 3.5 million per year.");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "The salary is 3.5 million per year.");
    }

    #[test]
    fn split_sentences_includes_closing_quotes() {
        let result = TextDiffEngine::split_sentences("He said \"Stop!\" and left.");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "He said \"Stop!\" and left.");
    }

    #[test]
    fn split_sentences_without_terminal_punctuation() {
        let result = TextDiffEngine::split_sentences("No punctuation here");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "No punctuation here");
    }

    #[test]
    fn split_sentences_empty_input() {
        assert!(TextDiffEngine::split_sentences("").is_empty());
        assert!(TextDiffEngine::split_sentences("   ").is_empty());
    }

    #[test]
    fn similarity_identical_is_one() {
        let score = TextDiffEngine::similarity("quick brown fox", "quick brown fox");
        assert!((score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn similarity_disjoint_is_zero() {
        let score = TextDiffEngine::similarity("alpha beta", "gamma delta");
        assert!(score.abs() < f64::EPSILON);
    }

    #[test]
    fn similarity_partial_overlap_between_zero_and_one() {
        let score = TextDiffEngine::similarity("alpha beta gamma", "alpha beta delta");
        assert!(score > 0.0 && score < 1.0);
    }

    #[test]
    fn similarity_empty_inputs_are_one() {
        assert!((TextDiffEngine::similarity("", "") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn diff_identical_texts_has_no_changes() {
        let engine = TextDiffEngine::new();
        let text = "First paragraph.\n\nSecond paragraph.";
        let result = engine.diff_texts(text, text);
        assert!(result.changes.is_empty());
        assert_eq!(result.unchanged_count, 2);
    }

    #[test]
    fn diff_both_empty_has_no_changes() {
        let engine = TextDiffEngine::new();
        let result = engine.diff_texts("", "");
        assert!(result.changes.is_empty());
        assert_eq!(result.unchanged_count, 0);
    }

    #[test]
    fn diff_added_sentence_detected() {
        let engine = TextDiffEngine::new();
        let old = "The team is small.";
        let new = "The team is small. We use modern tooling every day.";
        let result = engine.diff_texts(old, new);
        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.changes[0].change_type, ChangeType::Added);
        assert_eq!(
            result.changes[0].content,
            "We use modern tooling every day."
        );
        assert_eq!(result.changes[0].new_index, Some(1));
        assert_eq!(result.changes[0].old_index, None);
    }

    #[test]
    fn diff_removed_sentence_detected() {
        let engine = TextDiffEngine::new();
        let old = "The team is small. This line will vanish soon.";
        let new = "The team is small.";
        let result = engine.diff_texts(old, new);
        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.changes[0].change_type, ChangeType::Removed);
        assert_eq!(
            result.changes[0].content,
            "This line will vanish soon."
        );
        assert_eq!(result.changes[0].old_index, Some(1));
        assert_eq!(result.changes[0].new_index, None);
    }

    #[test]
    fn diff_modified_sentence_detected_with_previous_content() {
        let engine = TextDiffEngine::new();
        let old = "The role offers a competitive salary.";
        let new = "The role offers a highly competitive salary package.";
        let result = engine.diff_texts(old, new);
        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.changes[0].change_type, ChangeType::Modified);
        assert_eq!(
            result.changes[0].previous_content.as_deref(),
            Some("The role offers a competitive salary.")
        );
        assert_eq!(
            result.changes[0].content,
            "The role offers a highly competitive salary package."
        );
    }

    #[test]
    fn diff_dissimilar_replacement_is_added_and_removed_not_modified() {
        let engine = TextDiffEngine::new();
        let old = "Great benefits are included always.";
        let new = "Candidates must know Rust deeply.";
        let result = engine.diff_texts(old, new);
        assert_eq!(result.changes.len(), 2);
        assert!(result
            .changes
            .iter()
            .any(|c| c.change_type == ChangeType::Removed));
        assert!(result
            .changes
            .iter()
            .any(|c| c.change_type == ChangeType::Added));
    }

    #[test]
    fn diff_case_insensitive_change_ignored() {
        let engine = TextDiffEngine::new();
        let result = engine.diff_texts("Hello World.", "hello world.");
        assert!(result.changes.is_empty());
        assert_eq!(result.unchanged_count, 1);
    }

    #[test]
    fn diff_html_and_whitespace_normalization_ignored() {
        let engine = TextDiffEngine::new();
        let result = engine.diff_texts(
            "<p>Hello&nbsp;&nbsp;world.</p>",
            "<p>Hello world!</p>",
        );
        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.changes[0].change_type, ChangeType::Modified);
    }

    #[test]
    fn diff_mixed_scenario_classifies_all_types() {
        let engine = TextDiffEngine::new();
        let old = "Keep this line.\n\nRemove that old line entirely now.\n\nChange this particular line please.";
        let new = "Keep this line.\n\nThis brand new line appears here.\n\nChange this particular sentence today.";
        let result = engine.diff_paragraphs(old, new);

        assert!(result
            .changes
            .iter()
            .any(|c| c.change_type == ChangeType::Added));
        assert!(result
            .changes
            .iter()
            .any(|c| c.change_type == ChangeType::Removed));
        assert!(result
            .changes
            .iter()
            .any(|c| c.change_type == ChangeType::Modified));
        assert_eq!(result.unchanged_count, 1);
    }

    #[test]
    fn diff_preserves_original_unnormalized_text_in_changes() {
        let engine = TextDiffEngine::new();
        let result = engine.diff_texts("<p>Brand new content.</p>", "");
        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.changes[0].content, "<p>Brand new content.</p>");
    }

    #[test]
    fn diff_multiple_additions_all_reported() {
        let engine = TextDiffEngine::new();
        let old = "Base line stays.";
        let new = "Base line stays. Added number one here. Added number two there.";
        let result = engine.diff_texts(old, new);
        let added: Vec<_> = result
            .changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Added)
            .collect();
        assert_eq!(added.len(), 2);
    }

    #[test]
    fn diff_is_deterministic_across_runs() {
        let engine = TextDiffEngine::new();
        let old = "Alpha beta gamma.\n\nDelta epsilon zeta eta.";
        let new = "Alpha beta gamma.\n\nTheta iota kappa lambda mu.";
        let first = engine.diff_texts(old, new);
        let second = engine.diff_texts(old, new);
        assert_eq!(first, second);
    }

    #[test]
    fn diff_segments_works_directly_on_provided_segments() {
        let engine = TextDiffEngine::new();
        let old = vec!["one".to_string(), "two".to_string()];
        let new = vec!["one".to_string(), "two".to_string(), "three".to_string()];
        let result = engine.diff_segments(&old, &new);
        assert_eq!(result.unchanged_count, 2);
        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.changes[0].change_type, ChangeType::Added);
    }

    #[test]
    fn custom_similarity_threshold_controls_pairing() {
        let strict = TextDiffEngine::with_similarity_threshold(1.0);
        let lenient = TextDiffEngine::with_similarity_threshold(0.1);
        let old = "Alpha beta gamma delta epsilon.";
        let new = "Alpha beta gamma delta epsilon zeta.";

        let strict_result = strict.diff_texts(old, new);
        let lenient_result = lenient.diff_texts(old, new);

        assert!(strict_result
            .changes
            .iter()
            .all(|c| c.change_type != ChangeType::Modified));
        assert!(lenient_result
            .changes
            .iter()
            .any(|c| c.change_type == ChangeType::Modified));
    }

    #[test]
    fn split_bullets_dash_marker() {
        let result = TextDiffEngine::split_bullets("- First item\n- Second item\n- Third item");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "First item");
        assert_eq!(result[1], "Second item");
        assert_eq!(result[2], "Third item");
    }

    #[test]
    fn split_bullets_star_and_bullet_char() {
        let star = TextDiffEngine::split_bullets("* Alpha\n* Beta");
        assert_eq!(star, vec!["Alpha".to_string(), "Beta".to_string()]);
        let bullet = TextDiffEngine::split_bullets("• Alpha\n• Beta");
        assert_eq!(bullet, vec!["Alpha".to_string(), "Beta".to_string()]);
    }

    #[test]
    fn split_bullets_numbered_markers() {
        let dotted = TextDiffEngine::split_bullets("1. One\n2. Two\n3. Three");
        assert_eq!(dotted, vec!["One".to_string(), "Two".to_string(), "Three".to_string()]);
        let paren = TextDiffEngine::split_bullets("1) One\n2) Two");
        assert_eq!(paren, vec!["One".to_string(), "Two".to_string()]);
    }

    #[test]
    fn split_bullets_continuation_lines_append() {
        let result = TextDiffEngine::split_bullets("- First item that wraps\n  onto a second line\n- Second item");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "First item that wraps onto a second line");
        assert_eq!(result[1], "Second item");
    }

    #[test]
    fn split_bullets_ignores_blank_separators() {
        let result = TextDiffEngine::split_bullets("- One\n\n- Two\n\n- Three");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "One");
    }

    #[test]
    fn split_bullets_no_bullets_returns_empty() {
        assert!(TextDiffEngine::split_bullets("Just a plain paragraph.").is_empty());
        assert!(TextDiffEngine::split_bullets("").is_empty());
    }

    #[test]
    fn split_bullets_mixed_with_prose_only_returns_bullet_blocks() {
        let text = "Intro line.\n- Bullet one\n- Bullet two";
        let result = TextDiffEngine::split_bullets(text);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "Bullet one");
    }

    #[test]
    fn split_bullets_trims_marker_whitespace() {
        let result = TextDiffEngine::split_bullets("-  padded item   ");
        assert_eq!(result, vec!["padded item".to_string()]);
    }

    #[test]
    fn diff_bullet_added_detected() {
        let engine = TextDiffEngine::new();
        let old = "- Alpha\n- Beta";
        let new = "- Alpha\n- Beta\n- Gamma";
        let result = engine.diff_bullets(old, new);
        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.changes[0].change_type, ChangeType::Added);
        assert_eq!(result.changes[0].content, "Gamma");
        assert_eq!(result.changes[0].new_index, Some(2));
        assert_eq!(result.unchanged_count, 2);
    }

    #[test]
    fn diff_bullet_removed_detected() {
        let engine = TextDiffEngine::new();
        let old = "- Alpha\n- Beta\n- Gamma";
        let new = "- Alpha\n- Gamma";
        let result = engine.diff_bullets(old, new);
        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.changes[0].change_type, ChangeType::Removed);
        assert_eq!(result.changes[0].content, "Beta");
        assert_eq!(result.changes[0].old_index, Some(1));
    }

    #[test]
    fn diff_bullet_modified_detected() {
        let engine = TextDiffEngine::new();
        let old = "- Build the CI pipeline.\n- Write docs.";
        let new = "- Build and maintain the CI pipeline.\n- Write docs.";
        let result = engine.diff_bullets(old, new);
        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.changes[0].change_type, ChangeType::Modified);
        assert_eq!(
            result.changes[0].previous_content.as_deref(),
            Some("Build the CI pipeline.")
        );
        assert_eq!(
            result.changes[0].content,
            "Build and maintain the CI pipeline."
        );
        assert_eq!(result.unchanged_count, 1);
    }

    #[test]
    fn diff_bullet_identical_has_no_changes() {
        let engine = TextDiffEngine::new();
        let text = "- Alpha\n- Beta\n- Gamma";
        let result = engine.diff_bullets(text, text);
        assert!(result.changes.is_empty());
        assert_eq!(result.unchanged_count, 3);
    }

    #[test]
    fn diff_bullets_normalization_ignored() {
        let engine = TextDiffEngine::new();
        let old = "- Fluent in Rust.";
        let new = "- fluent in rust.";
        let result = engine.diff_bullets(old, new);
        assert!(result.changes.is_empty());
        assert_eq!(result.unchanged_count, 1);
    }

    #[test]
    fn diff_bullets_empty_inputs() {
        let engine = TextDiffEngine::new();
        let result = engine.diff_bullets("", "");
        assert!(result.changes.is_empty());
        assert_eq!(result.unchanged_count, 0);
    }

    #[test]
    fn diff_bullets_mixed_add_remove_modify() {
        let engine = TextDiffEngine::new();
        let old = "- Keep alpha\n- Mnemonics removed oris\n- Change this line please";
        let new = "- Keep alpha\n- New line introduced here\n- Change this impactful line please";
        let result = engine.diff_bullets(old, new);
        assert!(result
            .changes
            .iter()
            .any(|c| c.change_type == ChangeType::Added));
        assert!(result
            .changes
            .iter()
            .any(|c| c.change_type == ChangeType::Removed));
        assert!(result
            .changes
            .iter()
            .any(|c| c.change_type == ChangeType::Modified));
    }

    #[test]
    fn diff_bullets_is_deterministic() {
        let engine = TextDiffEngine::new();
        let old = "- Alpha\n- Beta\n- Gamma";
        let new = "- Gamma\n- Alpha\n- Delta";
        let first = engine.diff_bullets(old, new);
        let second = engine.diff_bullets(old, new);
        assert_eq!(first, second);
    }
}
