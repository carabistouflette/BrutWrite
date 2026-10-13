use super::chapter::Chapter;
use crate::errors::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub struct Manifest {
    pub chapters: Vec<Chapter>,
}

impl Manifest {
    pub fn create_chapter(&self, parent_id: Option<String>, title: String) -> Chapter {
        let new_id = format!("chapter-{}", Uuid::new_v4());
        let filename = format!("{}.md", new_id);

        let max_order = self
            .chapters
            .iter()
            .filter(|chapter| chapter.parent_id == parent_id)
            .map(|chapter| chapter.order)
            .max();
        let new_order = max_order.map_or(0, |max_order| max_order.saturating_add(1));

        Chapter {
            id: new_id,
            parent_id,
            title,
            filename,
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

    pub fn validate_parent(&self, parent_id: Option<&str>) -> Result<()> {
        if let Some(parent_id) = parent_id {
            if !self.chapters.iter().any(|chapter| chapter.id == parent_id) {
                return Err(Error::ChapterNotFound {
                    id: parent_id.to_string(),
                });
            }
        }
        Ok(())
    }

    /// Validate all tree relationships and identifiers before storing a manifest.
    pub fn validate(&self) -> Result<()> {
        let mut ids = HashSet::with_capacity(self.chapters.len());
        let mut filenames = HashSet::with_capacity(self.chapters.len());
        let mut sibling_orders = HashSet::with_capacity(self.chapters.len());

        for chapter in &self.chapters {
            if chapter.id.trim().is_empty() {
                return Err(Error::Validation(
                    "Manifest chapter ID cannot be empty".to_string(),
                ));
            }
            if !ids.insert(chapter.id.as_str()) {
                return Err(Error::Validation(format!(
                    "Duplicate chapter ID: {}",
                    chapter.id
                )));
            }
            crate::validation::validate_name(&chapter.id)?;
            if chapter.id.contains('/') || chapter.id.contains('\\') {
                return Err(Error::Validation(format!(
                    "Invalid chapter ID: {}",
                    chapter.id
                )));
            }
            let mut id_components = Path::new(&chapter.id).components();
            if !matches!(id_components.next(), Some(Component::Normal(_)))
                || id_components.next().is_some()
            {
                return Err(Error::Validation(format!(
                    "Invalid chapter ID: {}",
                    chapter.id
                )));
            }
            crate::validation::validate_chapter_title(&chapter.title)?;
            crate::validation::validate_name(&chapter.filename)?;
            if chapter.filename.contains('/') || chapter.filename.contains('\\') {
                return Err(Error::Validation(format!(
                    "Invalid chapter filename: {}",
                    chapter.filename
                )));
            }

            let mut components = Path::new(&chapter.filename).components();
            if !matches!(components.next(), Some(Component::Normal(_)))
                || components.next().is_some()
            {
                return Err(Error::Validation(format!(
                    "Invalid chapter filename: {}",
                    chapter.filename
                )));
            }
            if !filenames.insert(chapter.filename.as_str()) {
                return Err(Error::Validation(format!(
                    "Duplicate chapter filename: {}",
                    chapter.filename
                )));
            }
            if !sibling_orders.insert((chapter.parent_id.as_deref(), chapter.order)) {
                return Err(Error::Validation(format!(
                    "Duplicate chapter order {} among siblings",
                    chapter.order
                )));
            }
        }

        for chapter in &self.chapters {
            if let Some(parent_id) = &chapter.parent_id {
                if parent_id == &chapter.id {
                    return Err(Error::Validation(format!(
                        "Chapter {} cannot be its own parent",
                        chapter.id
                    )));
                }
                if !ids.contains(parent_id.as_str()) {
                    return Err(Error::ChapterNotFound {
                        id: parent_id.clone(),
                    });
                }
            }
            if let Some(dependency_id) = &chapter.depends_on {
                if dependency_id == &chapter.id {
                    return Err(Error::Validation(format!(
                        "Chapter {} cannot depend on itself",
                        chapter.id
                    )));
                }
                if !ids.contains(dependency_id.as_str()) {
                    return Err(Error::ChapterNotFound {
                        id: dependency_id.clone(),
                    });
                }
            }
        }

        let parent_by_id: HashMap<&str, Option<&str>> = self
            .chapters
            .iter()
            .map(|chapter| (chapter.id.as_str(), chapter.parent_id.as_deref()))
            .collect();
        let mut visit_state = HashMap::with_capacity(self.chapters.len());

        fn visit_parent<'a>(
            id: &'a str,
            parent_by_id: &HashMap<&'a str, Option<&'a str>>,
            visit_state: &mut HashMap<&'a str, u8>,
        ) -> Result<()> {
            match visit_state.get(id).copied().unwrap_or(0) {
                1 => {
                    return Err(Error::Validation(format!(
                        "Chapter hierarchy contains a cycle at {}",
                        id
                    )))
                }
                2 => return Ok(()),
                _ => {}
            }

            visit_state.insert(id, 1);
            if let Some(parent_id) = parent_by_id.get(id).copied().flatten() {
                visit_parent(parent_id, parent_by_id, visit_state)?;
            }
            visit_state.insert(id, 2);
            Ok(())
        }

        for id in &ids {
            visit_parent(id, &parent_by_id, &mut visit_state)?;
        }

        Ok(())
    }

    /// Apply a full client tree while preserving server-owned counts and omitted metadata.
    pub fn merge_structure_update(&self, mut update: Manifest) -> Result<Manifest> {
        if update.chapters.len() != self.chapters.len() {
            return Err(Error::Validation(
                "Manifest updates cannot add or remove chapters; use the chapter commands"
                    .to_string(),
            ));
        }

        let mut existing: HashMap<&str, &Chapter> = self
            .chapters
            .iter()
            .map(|chapter| (chapter.id.as_str(), chapter))
            .collect();

        for chapter in &mut update.chapters {
            let previous = existing.remove(&chapter.id.as_str()).ok_or_else(|| {
                Error::Validation(format!(
                    "Manifest update contains an unknown or duplicate chapter ID: {}",
                    chapter.id
                ))
            })?;
            if chapter.filename != previous.filename {
                return Err(Error::Validation(format!(
                    "Chapter filename cannot change for {}",
                    chapter.id
                )));
            }

            chapter.word_count = previous.word_count;
            if chapter.chronological_date.is_none() {
                chapter.chronological_date = previous.chronological_date.clone();
            }
            if chapter.abstract_timeframe.is_none() {
                chapter.abstract_timeframe = previous.abstract_timeframe.clone();
            }
            if chapter.duration.is_none() {
                chapter.duration = previous.duration.clone();
            }
            if chapter.plotline_tag.is_none() {
                chapter.plotline_tag = previous.plotline_tag.clone();
            }
            if chapter.depends_on.is_none() {
                chapter.depends_on = previous.depends_on.clone();
            }
            if chapter.pov_character_id.is_none() {
                chapter.pov_character_id = previous.pov_character_id.clone();
            }
        }

        if let Some(missing_id) = existing.keys().next() {
            return Err(Error::Validation(format!(
                "Manifest update omitted chapter {}",
                missing_id
            )));
        }

        Ok(update)
    }

    pub fn remove_node_recursively(&mut self, node_id: &str) -> Result<Vec<(String, String)>> {
        if !self.chapters.iter().any(|chapter| chapter.id == node_id) {
            return Err(Error::ChapterNotFound {
                id: node_id.to_string(),
            });
        }

        let mut children_map: HashMap<Option<String>, Vec<String>> = HashMap::new();
        for chapter in &self.chapters {
            children_map
                .entry(chapter.parent_id.clone())
                .or_default()
                .push(chapter.id.clone());
        }

        let mut ids_to_remove = HashSet::new();
        let mut stack = vec![node_id.to_string()];
        while let Some(current_id) = stack.pop() {
            if ids_to_remove.insert(current_id.clone()) {
                if let Some(children) = children_map.get(&Some(current_id)) {
                    stack.extend(children.iter().cloned());
                }
            }
        }

        let mut removed = Vec::new();
        self.chapters.retain(|chapter| {
            if ids_to_remove.contains(&chapter.id) {
                removed.push((chapter.id.clone(), chapter.filename.clone()));
                false
            } else {
                true
            }
        });
        for chapter in &mut self.chapters {
            if chapter
                .depends_on
                .as_ref()
                .is_some_and(|dependency_id| ids_to_remove.contains(dependency_id))
            {
                chapter.depends_on = None;
            }
        }

        Ok(removed)
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
    fn recursive_delete_removes_descendants_and_clears_dependencies() {
        let mut manifest = Manifest::default();
        manifest
            .chapters
            .push(create_dummy_chapter("root", None, "root.md"));
        manifest
            .chapters
            .push(create_dummy_chapter("child", Some("root"), "child.md"));
        let mut dependent = create_dummy_chapter("dependent", None, "dependent.md");
        dependent.order = 1;
        dependent.depends_on = Some("child".to_string());
        manifest.chapters.push(dependent);

        let removed = manifest.remove_node_recursively("root").unwrap();

        assert_eq!(removed.len(), 2);
        assert!(removed.contains(&("root".to_string(), "root.md".to_string())));
        assert!(removed.contains(&("child".to_string(), "child.md".to_string())));
        assert_eq!(manifest.chapters.len(), 1);
        assert_eq!(manifest.chapters[0].id, "dependent");
        assert_eq!(manifest.chapters[0].depends_on, None);
    }

    #[test]
    fn create_rename_and_reorder_accept_punctuated_titles() {
        let empty = Manifest::default();
        let first = empty.create_chapter(None, "Chapter: A/B?".to_string());
        let one_chapter = Manifest {
            chapters: vec![first.clone()],
        };
        let second = one_chapter.create_chapter(None, "Another <scene> / maybe?".to_string());
        let original = Manifest {
            chapters: vec![first.clone(), second.clone()],
        };
        original.validate().unwrap();

        let mut renamed_first = first.clone();
        renamed_first.title = "Renamed: A/B? <draft>".to_string();
        renamed_first.order = 1;
        let mut reordered_second = second.clone();
        reordered_second.order = 0;
        let updated = original
            .merge_structure_update(Manifest {
                chapters: vec![reordered_second, renamed_first],
            })
            .unwrap();
        updated.validate().unwrap();

        assert_eq!(updated.chapters[0].id, second.id);
        assert_eq!(updated.chapters[1].title, "Renamed: A/B? <draft>");
    }
    #[test]
    fn structure_update_preserves_counts_and_existing_optional_metadata() {
        let mut root_a = create_dummy_chapter("a", None, "a.md");
        root_a.order = 0;
        root_a.word_count = 23;
        root_a.chronological_date = Some("2030-01-01".to_string());
        let mut root_b = create_dummy_chapter("b", None, "b.md");
        root_b.order = 1;
        let mut child = create_dummy_chapter("child", Some("a"), "child.md");
        child.word_count = 41;
        child.plotline_tag = Some("red".to_string());
        child.depends_on = Some("b".to_string());

        let original = Manifest {
            chapters: vec![root_a, root_b, child],
        };
        let mut moved_child = create_dummy_chapter("child", Some("b"), "child.md");
        moved_child.order = 0;
        let mut reordered_a = create_dummy_chapter("a", None, "a.md");
        reordered_a.order = 1;
        let reordered_b = create_dummy_chapter("b", None, "b.md");
        let updated = original
            .merge_structure_update(Manifest {
                chapters: vec![reordered_b, reordered_a, moved_child],
            })
            .unwrap();
        updated.validate().unwrap();

        assert_eq!(
            updated
                .chapters
                .iter()
                .map(|chapter| chapter.id.as_str())
                .collect::<Vec<_>>(),
            vec!["b", "a", "child"]
        );
        assert_eq!(updated.chapters[2].parent_id.as_deref(), Some("b"));
        assert_eq!(updated.chapters[1].order, 1);
        assert_eq!(updated.chapters[0].word_count, 0);
        assert_eq!(updated.chapters[1].word_count, 23);
        assert_eq!(updated.chapters[2].word_count, 41);
        assert_eq!(
            updated.chapters[1].chronological_date.as_deref(),
            Some("2030-01-01")
        );
        assert_eq!(updated.chapters[2].plotline_tag.as_deref(), Some("red"));
        assert_eq!(updated.chapters[2].depends_on.as_deref(), Some("b"));
    }

    #[test]
    fn structure_update_rejects_membership_changes() {
        let original = Manifest {
            chapters: vec![create_dummy_chapter("a", None, "a.md")],
        };

        let result = original.merge_structure_update(Manifest::default());

        assert!(result.is_err());
    }

    #[test]
    fn validation_rejects_missing_parents_and_hierarchy_cycles() {
        let mut missing_parent = Manifest {
            chapters: vec![create_dummy_chapter("child", Some("missing"), "child.md")],
        };
        assert!(missing_parent.validate().is_err());

        missing_parent.chapters[0].parent_id = Some("other".to_string());
        missing_parent
            .chapters
            .push(create_dummy_chapter("other", Some("child"), "other.md"));
        assert!(missing_parent.validate().is_err());
    }

    #[test]
    fn recursive_delete_reports_unknown_nodes() {
        let mut manifest = Manifest::default();
        assert!(manifest.remove_node_recursively("missing").is_err());
    }

    #[test]
    fn parent_validation_rejects_nonexistent_parent() {
        let manifest = Manifest::default();
        assert!(manifest.validate_parent(Some("missing-parent")).is_err());
        assert!(manifest.validate_parent(None).is_ok());
    }
}
