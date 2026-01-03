use regex::Regex;
use std::sync::OnceLock;

static HTML_TAG_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn count_words(content: &str) -> u32 {
    let re =
        HTML_TAG_REGEX.get_or_init(|| Regex::new(r"<[^>]*>").expect("static regex must compile"));
    let plain_text = re.replace_all(content, " ");
    plain_text.split_whitespace().count() as u32
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
