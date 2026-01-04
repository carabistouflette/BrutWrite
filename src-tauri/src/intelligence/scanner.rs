use crate::models::Character;
use aho_corasick::{AhoCorasick, MatchKind};

const MAX_NAME_LEN: usize = 64;

/// Pre-compiled search patterns for all characters.
#[derive(Clone, Debug)]
pub struct CharacterScanner {
    /// Aho-Corasick automaton for O(n) multi-pattern search.
    ac: AhoCorasick,
    /// Maps pattern index from AC -> index in `ids` vector
    pattern_to_char_idx: Vec<usize>,
    /// Unique character IDs
    ids: Vec<String>,
    /// Bitmask indicating which patterns require strict word boundary checks.
    requires_boundary: Vec<bool>,
}

impl CharacterScanner {
    pub fn try_new(characters: &[Character]) -> Result<Self, String> {
        let mut patterns = Vec::new();
        let mut pattern_to_char_idx = Vec::new();
        let mut requires_boundary = Vec::new();
        let mut ids = Vec::new();

        for (i, c) in characters.iter().enumerate() {
            if c.name.len() > MAX_NAME_LEN {
                log::warn!(
                    "Character name '{}' is too long, shortening for analysis",
                    c.name
                );
            }
            let safe_name = c.name[..c.name.len().min(MAX_NAME_LEN)].to_lowercase();
            let id_str = c.id.to_string();

            // 1. Literal Name (Requires \b checks)
            patterns.push(safe_name.clone());
            pattern_to_char_idx.push(i);
            requires_boundary.push(true);

            // 2. @Mention (No left boundary check needed if @ is non-word, but we rely on AC)
            // Ideally we just want to match "@Name"
            patterns.push(format!("@{}", safe_name));
            pattern_to_char_idx.push(i);
            requires_boundary.push(false); // explicit symbol prefix usually suffices

            // 3. data-id="..." (Exact machine match)
            // Handle both single and double quotes if needed, though usually standardizing is better.
            // We'll add both common variants to be safe.
            patterns.push(format!("data-id=\"{}\"", id_str));
            pattern_to_char_idx.push(i);
            requires_boundary.push(false);

            patterns.push(format!("data-id='{}'", id_str));
            pattern_to_char_idx.push(i);
            requires_boundary.push(false);

            // Store ID
            ids.push(id_str);
        }

        if patterns.is_empty() {
            return Err("No characters to analyze".into());
        }

        let ac = AhoCorasick::builder()
            .ascii_case_insensitive(true)
            .match_kind(MatchKind::LeftmostLongest) // Match "Christopher" over "Chris"
            .build(&patterns)
            .map_err(|e| {
                log::error!("Failed to build Aho-Corasick automaton: {}", e);
                e.to_string()
            })?;

        Ok(Self {
            ac,
            pattern_to_char_idx,
            ids,
            requires_boundary,
        })
    }

    /// Scans text and returns mentions as (offset, char_id)
    pub fn scan(&self, text: &str) -> Vec<(usize, String)> {
        let mut mentions = Vec::new();
        // Aho-Corasick find_iter returns non-overlapping matches by default
        for mat in self.ac.find_iter(text) {
            let pattern_idx = mat.pattern().as_usize();
            let start = mat.start();
            let end = mat.end();

            // Check boundaries if required
            if self.requires_boundary[pattern_idx] && !is_word_boundary(text, start, end) {
                continue;
            }

            // Map back to ID
            let char_idx = self.pattern_to_char_idx[pattern_idx];
            if let Some(id) = self.ids.get(char_idx) {
                mentions.push((start, id.clone()));
            }
        }
        mentions
    }
}

/// Check if a match at `start..end` is surrounded by word boundaries.
/// Equivalent to regex `\bMATCH\b`.
fn is_word_boundary(text: &str, start: usize, end: usize) -> bool {
    // Check left
    if start > 0 {
        if let Some(prev_char) = text[..start].chars().last() {
            if prev_char.is_alphanumeric() || prev_char == '_' {
                return false;
            }
        }
    }
    // Check right
    if end < text.len() {
        if let Some(next_char) = text[end..].chars().next() {
            if next_char.is_alphanumeric() || next_char == '_' {
                return false;
            }
        }
    }
    true
}
