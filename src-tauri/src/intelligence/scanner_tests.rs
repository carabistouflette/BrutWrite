#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Character, CharacterRole};

    fn make_char(id: &str, name: &str, aliases: Vec<String>) -> Character {
        Character {
            id: uuid::Uuid::nil(), // Just for test structure, we use string id map really
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
        let mut char1 = make_char(
            "id1",
            "Robert",
            vec!["Bob".to_string(), "Bobby".to_string()],
        );
        char1.id = uuid::Uuid::new_v4();
        let id1 = char1.id.to_string();

        let scanner = CharacterScanner::try_new(&[char1]).expect("Failed to create scanner");

        let text = "Robert met Bob and they called Bobby over.";
        let mentions = scanner.scan(text);

        // Should detect: Robert, Bob, Bobby
        assert_eq!(mentions.len(), 3);

        let found_ids: Vec<String> = mentions.iter().map(|(_, id)| id.clone()).collect();
        assert_eq!(found_ids, vec![id1.clone(), id1.clone(), id1.clone()]);

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
        let mut char1 = make_char("id1", "Robert", vec!["Bob".to_string()]);
        char1.id = uuid::Uuid::new_v4();

        let scanner = CharacterScanner::try_new(&[char1]).expect("Failed to create scanner");

        // "Bobby" contains "Bob", but should not match "Bob" because of boundary check
        let text = "Bobby is not Bob";
        let mentions = scanner.scan(text);

        assert_eq!(mentions.len(), 1);
        // Should only match the second "Bob"
        assert_eq!(mentions[0].0, 13);
    }
}
