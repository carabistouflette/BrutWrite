use crate::models::{Character, CharacterEngine, CharacterRole};
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn seed_demo_project(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> crate::errors::Result<String> {
    // 1. Create Project
    let metadata = crate::commands::create_project(
        app,
        state.clone(),
        path.clone(),
        "The Algorithms of Betrayal".to_string(),
        "Alexisr".to_string(),
    )
    .await?;

    let project_id_uuid = metadata.id;

    // 2. Add Characters
    let characters = get_demo_characters();

    for char in &characters {
        crate::commands::save_character(state.clone(), project_id_uuid, char.clone()).await?;
    }

    // 3. Add Chapters with mentions (Using Aliases!)
    let chapters_data = get_demo_chapters(&characters);

    for (title, content) in chapters_data {
        // Create node
        let md =
            crate::commands::create_node(state.clone(), project_id_uuid, None, title.to_string())
                .await?;

        // Find the node ID we just created
        let node = md
            .manifest
            .chapters
            .iter()
            .find(|c| c.title == title)
            .ok_or_else(|| {
                crate::errors::Error::Validation(format!(
                    "Failed to retrieve created chapter: {}",
                    title
                ))
            })?;

        // Save content
        crate::commands::save_chapter(state.clone(), project_id_uuid, node.id.clone(), content)
            .await?;
    }

    Ok(project_id_uuid.to_string())
}

// =============================================================================
// Internal Data Helpers
// =============================================================================

fn get_demo_characters() -> Vec<Character> {
    vec![
        Character {
            id: Uuid::new_v4(),
            name: "Cipher".to_string(),
            role: CharacterRole::Protagonist,
            description: "A rogue hacker with a neural shunt.".to_string(),
            archetype: "The Outlaw".to_string(),
            engine: CharacterEngine::default(),
            physical_features: "Cybernetic eye, leather jacket".to_string(),
            traits: vec!["Paranoid".to_string(), "Skilled".to_string()],
            arc: "Redemption".to_string(),
            notes: "".to_string(),
        },
        Character {
            id: Uuid::new_v4(),
            name: "The Architect".to_string(),
            role: CharacterRole::Antagonist,
            description: "An AI construct controlling the city grid.".to_string(),
            archetype: "The Ruler".to_string(),
            engine: CharacterEngine::default(),
            physical_features: "Holographic projection".to_string(),
            traits: vec!["Calculating".to_string(), "Omnipresent".to_string()],
            arc: "Corruption".to_string(),
            notes: "".to_string(),
        },
        Character {
            id: Uuid::new_v4(),
            name: "Glitch".to_string(),
            role: CharacterRole::Secondary,
            description: "Street urchin and informant.".to_string(),
            archetype: "The Jester".to_string(),
            engine: CharacterEngine::default(),
            physical_features: "Neon tattoos".to_string(),
            traits: vec!["Fast talker".to_string()],
            arc: "".to_string(),
            notes: "".to_string(),
        },
        Character {
            id: Uuid::new_v4(),
            name: "Echo".to_string(),
            role: CharacterRole::Secondary,
            description: "Memory broker.".to_string(),
            archetype: "The Sage".to_string(),
            engine: CharacterEngine::default(),
            physical_features: "Masked".to_string(),
            traits: vec!["Wise".to_string()],
            arc: "".to_string(),
            notes: "".to_string(),
        },
        Character {
            id: Uuid::new_v4(),
            name: "Neon".to_string(),
            role: CharacterRole::Extra,
            description: "Bartender at The Void.".to_string(),
            archetype: "Everyman".to_string(),
            engine: CharacterEngine::default(),
            physical_features: "Robot arm".to_string(),
            traits: vec![],
            arc: "".to_string(),
            notes: "".to_string(),
        },
    ]
}

fn get_demo_chapters(characters: &[Character]) -> Vec<(&'static str, String)> {
    // Helper to create mention HTML with alias support
    // Uses html_escape to prevent injection
    let mention = |char: &Character, alias: Option<&str>| -> String {
        let text = alias.unwrap_or(&char.name);
        let safe_text = html_escape::encode_text(text);
        format!(
            r#"<span data-id="{}" data-type="character-mention" class="mention">{}</span>"#,
            char.id, safe_text
        )
    };

    vec![
        (
            "Chapter 1: The Wakeup",
            format!(
                "<p>The neon rain fell hard on the pavement. {} adjusted his collar. He was waiting for {}. 'Where is the data?' asked {}.</p><p>{} laughed. 'You think {} lets anything slip?'</p>",
                mention(&characters[0], Some("The hacker")), // Cipher -> The hacker
                mention(&characters[2], Some("the kid")),    // Glitch -> the kid
                mention(&characters[0], None),               // Cipher
                mention(&characters[2], None),               // Glitch
                mention(&characters[1], Some("The Architect")) // The Architect
            )
        ),
        (
            "Chapter 2: The Grid",
            format!(
                "<p>{} connected to the mainframe. The presence of {} was overwhelming. 'I see you,' the voice boomed. {} tried to disconnect, but {} held the line.</p>",
                mention(&characters[0], None),
                mention(&characters[1], Some("the machine god")), // The Architect -> the machine god
                mention(&characters[0], Some("Cipher")),
                mention(&characters[1], None)
            )
        ),
        (
            "Chapter 3: Broken Memories",
            format!(
                "<p>{} visited {}. 'I need to remember,' said {}. {} handed over a drive. 'This contains files on {}.'</p>",
                mention(&characters[0], None),
                mention(&characters[3], Some("the memory broker")), // Echo -> the memory broker
                mention(&characters[0], None),
                mention(&characters[3], None),
                mention(&characters[1], None)
            )
        ),
        (
            "Chapter 4: The Void Bar",
            format!(
                "<p>{} was cleaning a glass. {} sat at the bar, looking defeated. 'Drink?' asked {}. 'Make it strong,' replied {}. {} slid a cred-chip across the counter.</p>",
                mention(&characters[4], None), // Neon
                mention(&characters[0], Some("The exhausted runner")), // Cipher
                mention(&characters[4], None),
                mention(&characters[0], None),
                mention(&characters[2], None)
            )
        ),
        (
            "Chapter 5: Confrontation",
            format!(
                "<p>The final showdown. {} vs {}. {} watched from the shadows. {} unleashed the virus. {} screamed in binary.</p>",
                mention(&characters[0], None),
                mention(&characters[1], None),
                mention(&characters[2], None),
                mention(&characters[0], None),
                mention(&characters[1], None)
            )
        )
    ]
}
