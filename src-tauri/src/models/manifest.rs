use super::chapter::Chapter;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Manifest {
    pub chapters: Vec<Chapter>,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            chapters: Vec::new(),
        }
    }
}

impl Manifest {
    pub fn create_chapter(&self, parent_id: Option<String>, title: String) -> Chapter {
        let new_id = format!("chapter-{}", Uuid::new_v4());
        let filename = format!("{}.md", new_id);

        let siblings: Vec<&Chapter> = self
            .chapters
            .iter()
            .filter(|c| c.parent_id == parent_id)
            .collect();

        let max_order = siblings.iter().map(|c| c.order).max().unwrap_or(0);
        let new_order = if siblings.is_empty() {
            0
        } else {
            max_order + 1
        };

        Chapter {
            id: new_id.clone(),
            parent_id,
            title,
            filename: filename.clone(),
            word_count: 0,
            order: new_order,
            chronological_date: None,
            abstract_timeframe: None,
            duration: None,
            plotline_tag: None,
            depends_on: None,
            pov_character_id: None,
        }
    }

    pub fn remove_node_recursively(&mut self, node_id: String) -> Vec<String> {
        // Build efficient lookup
        let mut children_map: std::collections::HashMap<Option<String>, Vec<String>> =
            std::collections::HashMap::new();

        for c in &self.chapters {
            children_map
                .entry(c.parent_id.clone())
                .or_default()
                .push(c.id.clone());
        }

        let mut ids_to_remove = std::collections::HashSet::new();
        let mut stack = vec![node_id];

        while let Some(current_id) = stack.pop() {
            if ids_to_remove.insert(current_id.clone()) {
                if let Some(children) = children_map.get(&Some(current_id)) {
                    stack.extend(children.clone());
                }
            }
        }

        // Collect filenames and remove chapters
        let mut filenames = Vec::new();
        self.chapters.retain(|c| {
            if ids_to_remove.contains(&c.id) {
                filenames.push(c.filename.clone());
                false
            } else {
                true
            }
        });

        filenames
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_dummy_chapter(id: &str, parent: Option<&str>, filename: &str) -> Chapter {
        Chapter {
            id: id.to_string(),
            parent_id: parent.map(|s| s.to_string()),
            title: "Title".to_string(),
            filename: filename.to_string(),
            word_count: 0,
            order: 0,
            chronological_date: None,
            abstract_timeframe: None,
            duration: None,
            plotline_tag: None,
            depends_on: None,
            pov_character_id: None,
        }
    }

    #[test]
    fn test_remove_node_recursively() {
        let mut manifest = Manifest::default();

        // Tree:
        // root
        //  - c1 (c1.md)
        //     - c1_1 (c1_1.md)
        //  - c2 (c2.md)

        manifest
            .chapters
            .push(create_dummy_chapter("c1", None, "c1.md"));
        manifest
            .chapters
            .push(create_dummy_chapter("c1_1", Some("c1"), "c1_1.md"));
        manifest
            .chapters
            .push(create_dummy_chapter("c2", None, "c2.md"));

        let removed_files = manifest.remove_node_recursively("c1".to_string());

        assert_eq!(removed_files.len(), 2);
        assert!(removed_files.contains(&"c1.md".to_string()));
        assert!(removed_files.contains(&"c1_1.md".to_string()));
        assert!(!removed_files.contains(&"c2.md".to_string()));

        assert_eq!(manifest.chapters.len(), 1);
        assert_eq!(manifest.chapters[0].id, "c2");
    }
}
