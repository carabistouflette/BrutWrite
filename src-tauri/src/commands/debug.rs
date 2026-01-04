use crate::models::{Character, CharacterEngine, CharacterRole};
use crate::AppState;
use html_escape::encode_text;
use regex::Regex;
use serde::Deserialize;
use std::sync::OnceLock;
use tauri::State;
use uuid::Uuid;

const DEMO_JSON: &str = include_str!("../data/demo.json");

static CHAR_PLACEHOLDER_REGEX: OnceLock<Regex> = OnceLock::new();

#[derive(Deserialize)]
struct DemoData {
    characters: Vec<DemoCharacter>,
    chapters: Vec<DemoChapter>,
}

#[derive(Deserialize)]
struct DemoCharacter {
    name: String,
    role: CharacterRole,
    description: String,
    archetype: String,
    physical_features: String,
    traits: Vec<String>,
    arc: String,
}

#[derive(Deserialize)]
struct DemoChapter {
    title: String,
    content: String,
}

#[tauri::command]
#[cfg(debug_assertions)]
pub async fn seed_demo_project(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> crate::errors::Result<String> {
    // 1. Parse Data
    let demo_data: DemoData = serde_json::from_str(DEMO_JSON).map_err(|e| {
        crate::errors::Error::Validation(format!("Failed to parse embedded demo data: {}", e))
    })?;

    // 2. Create Project
    let metadata = crate::commands::create_project(
        app,
        state.clone(),
        path.clone(),
        "The Algorithms of Betrayal".to_string(),
        "Alexisr".to_string(),
    )
    .await?;

    let project_id_uuid = metadata.id;

    // 3. Create Characters & Map to Indices
    let mut created_characters = Vec::new();

    for dc in demo_data.characters {
        // Create full struct with safe defaults
        let char = Character {
            id: Uuid::new_v4(),
            name: dc.name,
            role: dc.role,
            description: dc.description,
            archetype: dc.archetype,
            engine: CharacterEngine::default(),
            physical_features: dc.physical_features,
            traits: dc.traits,
            arc: dc.arc,
            notes: String::new(),
        };

        crate::commands::save_character(state.clone(), project_id_uuid, char.clone()).await?;
        created_characters.push(char);
    }

    // 4. Create Chapters with Template Substitution
    // Regex matches {CHAR_0} or {CHAR_0:Alias}
    let re = CHAR_PLACEHOLDER_REGEX.get_or_init(|| {
        Regex::new(r"\{CHAR_(\d+)(?::([^}]+))?\}").expect("Static regex compilation failed")
    });

    for chapter in demo_data.chapters {
        // Safe substitution:
        // We match identifying patterns in the demo text and strictly replace them with a valid HTML mention span.
        // We use `encode_text` on the content to prevent XSS if the demo json itself was malicious (though it's embedded).
        let result = re.replace_all(&chapter.content, |caps: &regex::Captures| {
            let index: usize = caps[1].parse().unwrap_or(0);
            let alias = caps.get(2).map(|m| m.as_str());

            if let Some(char) = created_characters.get(index) {
                let text = alias.unwrap_or(&char.name);
                // Security: Explicitly escape text before inserting into HTML attribute and body
                let safe_text = encode_text(text);
                format!(
                    r#"<span data-id="{}" data-type="character-mention" class="mention">{}</span>"#,
                    char.id, safe_text
                )
            } else {
                // Fallback for bad indices in demo data
                "???".to_string()
            }
        });

        // 5. Create Node & Save Content
        let md = crate::commands::create_node(
            state.clone(),
            project_id_uuid,
            None,
            chapter.title.clone(),
        )
        .await?;

        // Find the node ID from the returned manifest
        let node = md
            .manifest
            .chapters
            .iter()
            .find(|c| c.title == chapter.title)
            .ok_or_else(|| {
                crate::errors::Error::Validation(format!(
                    "Failed to retrieve created chapter: {}",
                    chapter.title
                ))
            })?;

        // Save content
        crate::commands::save_chapter(
            state.clone(),
            project_id_uuid,
            node.id.clone(),
            result.to_string(), // Cow<str> -> String
        )
        .await?;
    }

    Ok(project_id_uuid.to_string())
}
