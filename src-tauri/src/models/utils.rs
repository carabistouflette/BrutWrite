use regex::Regex;
use std::sync::OnceLock;

static HTML_TAG_REGEX: OnceLock<Regex> = OnceLock::new();
static BLOCK_TAGS_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn count_words(content: &str) -> u32 {
    if content.is_empty() {
        return 0;
    }

    // 1. Replace block tags with space to ensure word separation
    let block_re = BLOCK_TAGS_REGEX.get_or_init(|| {
        Regex::new(r"(?i)</(?:p|div|h[1-6]|li|tr|br)>").expect("regex must compile")
    });
    let content_with_spaces = block_re.replace_all(content, " ");

    // 2. Strip remaining tags
    let tag_re =
        HTML_TAG_REGEX.get_or_init(|| Regex::new(r"<[^>]*>").expect("static regex must compile"));
    let text = tag_re.replace_all(&content_with_spaces, " ");

    // 3. Simple entity decoding for common cases
    let decoded = text
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");

    decoded.split_whitespace().count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_words() {
        assert_eq!(count_words("Hello world"), 2);
        assert_eq!(count_words("<p>Hello world</p>"), 2);
        assert_eq!(count_words("<p>Hello</p><p>world</p>"), 2);
        assert_eq!(count_words("Hello  world"), 2); // multiple spaces
        assert_eq!(count_words(""), 0);
        assert_eq!(count_words("<div>Nested <span>content</span></div>"), 2);
    }
}
