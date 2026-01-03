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

// =============================================================================
// Graph Analysis Helpers
// =============================================================================

/// Helper to map character positions to word indices efficiently.
pub struct WordIndexer {
    /// Byte offsets where words start.
    word_starts: Vec<usize>,
}

impl WordIndexer {
    pub fn new(text: &str) -> Self {
        let mut word_starts = Vec::new();
        // Use standard whitespace splitting logic consistent with split_whitespace()
        // We track the start of each word token.
        let mut in_word = false;

        for (idx, c) in text.char_indices() {
            if c.is_whitespace() {
                in_word = false;
            } else if !in_word {
                in_word = true;
                word_starts.push(idx);
            }
        }
        Self { word_starts }
    }

    /// Convert character position to approximate word index using binary search.
    /// Returns 0-based index.
    pub fn get_word_index(&self, char_pos: usize) -> usize {
        if self.word_starts.is_empty() {
            return 0;
        }
        match self.word_starts.binary_search(&char_pos) {
            Ok(idx) => idx,
            // If not found, `idx` is where it could be inserted.
            // This means it is before word[idx], so it belongs to word[idx-1] (or is in gap after it).
            Err(idx) => idx.saturating_sub(1),
        }
    }
}

/// Union-Find data structure for connected components.
pub struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    pub fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    pub fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    pub fn union(&mut self, x: usize, y: usize) {
        let px = self.find(x);
        let py = self.find(y);
        if px == py {
            return;
        }
        if self.rank[px] < self.rank[py] {
            self.parent[px] = py;
        } else if self.rank[px] > self.rank[py] {
            self.parent[py] = px;
        } else {
            self.parent[py] = px;
            self.rank[px] += 1;
        }
    }

    pub fn component_sizes(&mut self) -> Vec<u32> {
        let n = self.parent.len();
        let mut sizes: std::collections::HashMap<usize, u32> = std::collections::HashMap::new();
        for i in 0..n {
            let root = self.find(i);
            *sizes.entry(root).or_default() += 1;
        }
        sizes.into_values().collect()
    }
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
