pub struct TextNormalizer;

impl TextNormalizer {
    pub fn new() -> Self {
        Self
    }

    pub fn normalize_whitespace(text: &str) -> String {
        text.split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    pub fn normalize_line_breaks(text: &str) -> String {
        text.replace("\r\n", "\n")
            .replace("\r", "\n")
    }

    pub fn strip_html_tags(html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;
        let mut in_entity = false;
        let mut entity = String::new();

        for ch in html.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                '&' => {
                    in_entity = true;
                    entity.clear();
                    entity.push(ch);
                }
                ';' if in_entity => {
                    in_entity = false;
                    entity.push(ch);
                    match entity.as_str() {
                        "&amp;" => result.push('&'),
                        "&lt;" => result.push('<'),
                        "&gt;" => result.push('>'),
                        "&nbsp;" => result.push(' '),
                        "&quot;" => result.push('"'),
                        "&#39;" => result.push('\''),
                        _ => result.push_str(&entity),
                    }
                    entity.clear();
                }
                _ if in_entity => entity.push(ch),
                _ if !in_tag => result.push(ch),
                _ => {}
            }
        }

        result
    }

    pub fn lowercase(text: &str) -> String {
        text.to_lowercase()
    }

    pub fn normalize_for_comparison(text: &str) -> String {
        let stripped = Self::strip_html_tags(text);
        let lowered = Self::lowercase(&stripped);
        Self::normalize_whitespace(&lowered)
    }

    pub fn remove_filler_words(text: &str) -> String {
        let filler_words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
            "of", "with", "by", "from", "as", "is", "was", "are", "were", "be",
            "been", "being", "have", "has", "had", "do", "does", "did", "will",
            "would", "could", "should", "may", "might", "shall", "can", "this",
            "that", "these", "those", "i", "you", "he", "she", "it", "we", "they",
        ];

        let words: Vec<&str> = text.split_whitespace()
            .filter(|w| !filler_words.contains(&w.to_lowercase().as_str()))
            .collect();

        words.join(" ")
    }
}

impl Default for TextNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_whitespace_single_space() {
        let result = TextNormalizer::normalize_whitespace("  Hello   world  ");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn normalize_whitespace_tabs_and_newlines() {
        let result = TextNormalizer::normalize_whitespace("Hello\tworld\n");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn normalize_line_breaks_crlf() {
        let result = TextNormalizer::normalize_line_breaks("Hello\r\nWorld");
        assert_eq!(result, "Hello\nWorld");
    }

    #[test]
    fn normalize_line_breaks_cr() {
        let result = TextNormalizer::normalize_line_breaks("Hello\rWorld");
        assert_eq!(result, "Hello\nWorld");
    }

    #[test]
    fn strip_html_tags_basic() {
        let result = TextNormalizer::strip_html_tags("<p>Hello <b>world</b></p>");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn strip_html_entities() {
        let result = TextNormalizer::strip_html_tags("A &amp; B &lt; C &gt; D");
        assert_eq!(result, "A & B < C > D");
    }

    #[test]
    fn strip_html_entity_quotes() {
        let result = TextNormalizer::strip_html_tags("He said &quot;hello&quot;");
        assert_eq!(result, "He said \"hello\"");
    }

    #[test]
    fn strip_html_entity_apos() {
        let result = TextNormalizer::strip_html_tags("don&#39;t");
        assert_eq!(result, "don't");
    }

    #[test]
    fn lowercase_basic() {
        let result = TextNormalizer::lowercase("Hello WORLD");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn normalize_for_comparison_basic() {
        let result = TextNormalizer::normalize_for_comparison("<p>Hello   World</p>");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn normalize_for_comparison_with_entities() {
        let result = TextNormalizer::normalize_for_comparison("A &amp; B");
        assert_eq!(result, "a & b");
    }

    #[test]
    fn remove_filler_words_basic() {
        let result = TextNormalizer::remove_filler_words("The quick brown fox jumps over the lazy dog");
        assert_eq!(result, "quick brown fox jumps over lazy dog");
    }

    #[test]
    fn remove_filler_words_preserves_important_words() {
        let result = TextNormalizer::remove_filler_words("Software engineer with 5 years experience");
        assert_eq!(result, "Software engineer 5 years experience");
    }

    #[test]
    fn normalize_whitespace_empty_string() {
        let result = TextNormalizer::normalize_whitespace("");
        assert_eq!(result, "");
    }

    #[test]
    fn strip_html_tags_nested() {
        let result = TextNormalizer::strip_html_tags("<div><span>Hello</span> <b>World</b></div>");
        assert_eq!(result, "Hello World");
    }
}
