use crate::models::Character;
use aho_corasick::{AhoCorasick, MatchKind};
use uuid::Uuid;

pub const MAX_NAME_LEN: usize = 256;

/// Pre-compiled search patterns for all characters.
#[derive(Clone, Debug)]
pub struct CharacterScanner {
    /// Aho-Corasick automaton for O(n) multi-pattern search.
    ac: AhoCorasick,
    /// Maps pattern index from AC -> index in `ids` vector
    pattern_to_char_idx: Vec<usize>,
    /// Unique character IDs
    ids: Vec<Uuid>,
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
            // We still need the string ID for pattern matching data-id attributes
            let id_str = c.id.to_string();

            // 1. Literal Name (Requires \b checks)
            patterns.push(safe_name.clone());
            pattern_to_char_idx.push(i);
            requires_boundary.push(true);

            // 2. Aliases
            for alias in &c.aliases {
                if alias.trim().is_empty() {
                    continue;
                }
                let safe_alias = alias[..alias.len().min(MAX_NAME_LEN)].to_lowercase();
                patterns.push(safe_alias); // Aliases act like names
                pattern_to_char_idx.push(i);
                requires_boundary.push(true);
            }

            // 3. @Mention (No left boundary check needed if @ is non-word, but we rely on AC)
            // Ideally we just want to match "@Name"
            patterns.push(format!("@{}", safe_name));
            pattern_to_char_idx.push(i);
            requires_boundary.push(false); // explicit symbol prefix usually suffices

            // 4. data-id="..." (Exact machine match)
            // Handle both single and double quotes if needed, though usually standardizing is better.
            // We'll add both common variants to be safe.
            patterns.push(format!("data-id=\"{}\"", id_str));
            pattern_to_char_idx.push(i);
            requires_boundary.push(false);

            patterns.push(format!("data-id='{}'", id_str));
            pattern_to_char_idx.push(i);
            requires_boundary.push(false);

            // Store ID as Uuid
            ids.push(c.id);
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
    pub fn scan(&self, text: &str) -> Vec<(usize, Uuid)> {
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
                mentions.push((start, *id));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Character, CharacterRole};

    fn make_char(id: &str, name: &str, aliases: Vec<String>) -> Character {
        Character {
            id: uuid::Uuid::parse_str(id).unwrap_or(uuid::Uuid::new_v4()),
            name: name.to_string(),
            role: CharacterRole::Protagonist,
            archetype: "Hero".to_string(),
            description: "Desc".to_string(),
            engine: Default::default(),
            physical_features: "".to_string(),
            traits: vec![],
            arc: "".to_string(),
            notes: "".to_string(),
            aliases,
        }
    }

    #[test]
    fn test_scanner_with_aliases() {
        let id1_str = "00000000-0000-0000-0000-000000000001";
        let char1 = make_char(
            id1_str,
            "Robert",
            vec!["Bob".to_string(), "Bobby".to_string()],
        );

        let scanner = CharacterScanner::try_new(&[char1]).expect("Failed to create scanner");

        let text = "Robert met Bob and they called Bobby over.";
        let mentions = scanner.scan(text);

        // Should detect: Robert, Bob, Bobby
        assert_eq!(mentions.len(), 3);

        let found_ids: Vec<Uuid> = mentions.iter().map(|(_, id)| *id).collect();
        let expected_uuid = uuid::Uuid::parse_str(id1_str).unwrap();
        assert_eq!(found_ids, vec![expected_uuid, expected_uuid, expected_uuid]);

        // Verify names/offsets
        // "Robert" at 0
        assert_eq!(mentions[0].0, 0);
        // "Bob" at 11
        assert_eq!(mentions[1].0, 11);
        // "Bobby" at 31
        assert_eq!(mentions[2].0, 31);
    }

    #[test]
    fn test_scanner_alias_word_boundary() {
        let id1_str = "00000000-0000-0000-0000-000000000001";
        let char1 = make_char(id1_str, "Robert", vec!["Bob".to_string()]);

        let scanner = CharacterScanner::try_new(&[char1]).expect("Failed to create scanner");

        // "Bobby" contains "Bob", but should not match "Bob" because of boundary check
        let text = "Bobby is not Bob";
        let mentions = scanner.scan(text);

        assert_eq!(mentions.len(), 1);
        // Should only match the second "Bob"
        assert_eq!(mentions[0].0, 13);
    }
}
