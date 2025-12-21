use crate::models::{Manifest, ProjectMetadata};
use crate::storage;
use crate::AppState;
use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

// Helper to deduce common locking logic
 fn get_project_context(
    state: &State<'_, AppState>,
    project_id: Uuid,
) -> Result<(PathBuf, ProjectMetadata), String> {
    let root_path = {
        let projects = state
            .open_projects
            .lock()
            .map_err(|_| "Failed to lock projects")?;
        projects
            .get(&project_id)
            .cloned()
            .ok_or_else(|| "Project not loaded".to_string())?
    };

    let cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    let metadata = cache
        .get(&project_id)
        .cloned()
        .ok_or_else(|| "Metadata not in cache".to_string())?;

    Ok((root_path, metadata))
}

#[tauri::command]
pub async fn create_project(
    state: State<'_, AppState>,
    path: String,
    name: String,
    author: String,
) -> Result<ProjectMetadata, String> {
    let root_path = PathBuf::from(&path);
    let metadata =
        storage::create_project_structure(&root_path, &name, &author).map_err(|e| e.to_string())?;

    let mut projects = state
        .open_projects
        .lock()
        .map_err(|_| "Failed to lock projects")?;
    projects.insert(metadata.id, root_path);

    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    cache.insert(metadata.id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn load_project(
    state: State<'_, AppState>,
    path: String,
) -> Result<ProjectMetadata, String> {
    let root_path = PathBuf::from(&path);
    let metadata = storage::load_project_metadata(&root_path).map_err(|e| e.to_string())?;

    let mut projects = state
        .open_projects
        .lock()
        .map_err(|_| "Failed to lock projects")?;
    projects.insert(metadata.id, root_path);

    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    cache.insert(metadata.id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn update_manifest(
    state: State<'_, AppState>,
    project_id: Uuid,
    manifest: Manifest,
) -> Result<ProjectMetadata, String> {
    let (root_path, mut metadata) = get_project_context(&state, project_id)?;

    metadata.manifest = manifest;
    metadata.updated_at = chrono::Utc::now();

    storage::save_project_metadata(&root_path, &metadata).map_err(|e| e.to_string())?;

    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    cache.insert(project_id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn update_node_metadata(
    state: State<'_, AppState>,
    project_id: Uuid,
    node_id: String,
    update: crate::models::NodeMetadataUpdate,
) -> Result<ProjectMetadata, String> {
    let (root_path, mut metadata) = get_project_context(&state, project_id)?;

    if let Some(node) = metadata
        .manifest
        .chapters
        .iter_mut()
        .find(|c| c.id == node_id)
    {
        if let Some(t) = update.title { node.title = t; }
        if let Some(d) = update.chronological_date { node.chronological_date = Some(d); }
        if let Some(a) = update.abstract_timeframe { node.abstract_timeframe = Some(a); }
        if let Some(dur) = update.duration { node.duration = Some(dur); }
        if let Some(p) = update.plotline_tag { node.plotline_tag = Some(p); }
        if let Some(dep) = update.depends_on { node.depends_on = Some(dep); }
        if let Some(pov) = update.pov_character_id { node.pov_character_id = Some(pov); }
    } else {
        return Err("Node not found".to_string());
    }

    metadata.updated_at = chrono::Utc::now();
    storage::save_project_metadata(&root_path, &metadata).map_err(|e| e.to_string())?;

    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    cache.insert(project_id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn update_project_settings(
    state: State<'_, AppState>,
    project_id: Uuid,
    settings: crate::models::ProjectSettings,
) -> Result<ProjectMetadata, String> {
    let (root_path, mut metadata) = get_project_context(&state, project_id)?;

    metadata.settings = settings;
    metadata.updated_at = chrono::Utc::now();

    storage::save_project_metadata(&root_path, &metadata).map_err(|e| e.to_string())?;
    
    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    cache.insert(project_id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn update_plotlines(
    state: State<'_, AppState>,
    project_id: Uuid,
    plotlines: Vec<crate::models::Plotline>,
) -> Result<ProjectMetadata, String> {
    let (root_path, mut metadata) = get_project_context(&state, project_id)?;

    metadata.plotlines = plotlines;
    metadata.updated_at = chrono::Utc::now();

    storage::save_project_metadata(&root_path, &metadata).map_err(|e| e.to_string())?;

    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    cache.insert(project_id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn load_chapter_content(
    state: State<'_, AppState>,
    project_id: Uuid,
    chapter_id: String,
) -> Result<String, String> {
    let (root_path, metadata) = get_project_context(&state, project_id)?;
    storage::read_chapter_content(root_path, &metadata, &chapter_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_chapter(
    state: State<'_, AppState>,
    project_id: Uuid,
    chapter_id: String,
    content: String,
) -> Result<ProjectMetadata, String> {
    let (root_path, mut metadata) = get_project_context(&state, project_id)?;

    // 1. Resolve Path using cached metadata
    let chapter_path = storage::resolve_chapter_path(&root_path, &metadata, &chapter_id)
        .map_err(|e| e.to_string())?;

    // 2. Write content
    std::fs::write(&chapter_path, content.clone()).map_err(|e| e.to_string())?;

    // 3. Update metadata in cache
    if let Some(chapter) = metadata
        .manifest
        .chapters
        .iter_mut()
        .find(|c| c.id == chapter_id)
    {
        chapter.word_count = content.split_whitespace().count() as u32;
    }
    metadata.updated_at = chrono::Utc::now();

    // 4. Save metadata
    storage::save_project_metadata(&root_path, &metadata).map_err(|e| e.to_string())?;
    
    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    cache.insert(project_id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn create_node(
    state: State<'_, AppState>,
    project_id: Uuid,
    parent_id: Option<String>,
    name: String,
) -> Result<ProjectMetadata, String> {
    let (root_path, mut metadata) = get_project_context(&state, project_id)?;

    // Perform Creation using cached metadata
    storage::create_chapter_node(&root_path, &mut metadata, parent_id, name).map_err(|e| e.to_string())?;
    
    metadata.updated_at = chrono::Utc::now();

    // Save Metadata to Disk
    storage::save_project_metadata(&root_path, &metadata).map_err(|e| e.to_string())?;

    // Update Cache
    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    cache.insert(project_id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn delete_node(
    state: State<'_, AppState>,
    project_id: Uuid,
    id: String,
    filenames_to_delete: Vec<String>,
) -> Result<ProjectMetadata, String> {
    let (root_path, mut metadata) = get_project_context(&state, project_id)?;

    // 1. Delete files from disk
    for filename in filenames_to_delete {
        let file_path = root_path.join("manuscript").join(filename);
        if file_path.exists() {
            std::fs::remove_file(file_path).map_err(|e| e.to_string())?;
        }
    }

    // 2. Remove from manifest
    // metadata.manifest.chapters.retain(|c| c.id != id && c.parent_id != Some(id.clone()));
    
    // Note: The frontend sends the exact list of filenames to delete, implying recursive delete logic 
    // happens there to gather the list. Here we just need to cleanse the manifest of the specific node.
    // If we wanted to receive just the ID and do recursive deletion here, we'd need to traverse the tree.
    // Assuming the frontend call `delete_node` happens once per node or the node + children are handled?
    // Actually, `useProjectData.ts` collects filenames recursively but only calls `deleteNode` with the root ID to delete.
    // So we should strictly remove the node from manifest. 
    // Wait, if we delete a folder, its children in the manifest are now orphans if we don't delete them too.
    // Ideally, `delete_node` should effectively remove the subtree from the manifest.
    
    // BUT the current implementation in `useProjectData.ts` calls `collectFilenames(node, filesToDelete)`.
    // It passes `filenamesToDelete` to the backend.
    // The backend iterates and deletes files.
    // Then the backend filters the manifest: `metadata.manifest.chapters.retain(|c| c.id != id);`
    // This ONLY deletes the single node from the manifest array.
    // If that node had children, they are still in the manifest but their parent_id points to a non-existent node.
    // This creates orphans.
    
    // FIX: access the tree and remove all descendants or just trust `filenames_to_delete` aligns with descendants?
    // We don't have a map of ID -> Filename easily unless we scan.
    // A better approach for `delete_node` is to recursively remove all children of `id` from the manifest as well.
    metadata.manifest.chapters.retain(|c| c.id != id && c.parent_id != Some(id.clone())); 
    // This only handles one level.
    // To do it properly locally without full recursion logic again:
    // We can rely on the fact that if the file is deleted, maybe we should remove it?
    // But `manifest` is the source of truth.
    
    // For now, preserving the logic I saw in the file but cleaning it up.
    // The previous code had:
    // metadata.manifest.chapters.retain(|c| c.id != id && c.parent_id != Some(id.clone()));
    // AND then: metadata.manifest.chapters.retain(|c| c.id != id);
    // which was redundant.
    
    // Let's implement a simple recursive ID collection to filter properly?
    // Or just Keep it Simple given this is a refactor, not a logic rewrite (unless critical).
    // The User asked to search for architectural issues. Orphaned nodes is an issue.
    // I entered this knowing I'd fix issues.
    
    // Let's use a closure to find all descendant IDs.
    let mut ids_to_remove = vec![id.clone()];
    let mut i = 0;
    while i < ids_to_remove.len() {
        let current = ids_to_remove[i].clone();
        for c in &metadata.manifest.chapters {
            if c.parent_id == Some(current.clone()) {
                if !ids_to_remove.contains(&c.id) {
                     ids_to_remove.push(c.id.clone());
                }
            }
        }
        i += 1;
    }
    
    metadata.manifest.chapters.retain(|c| !ids_to_remove.contains(&c.id));

    metadata.updated_at = chrono::Utc::now();

    // 3. Save and Cache
    storage::save_project_metadata(&root_path, &metadata).map_err(|e| e.to_string())?;
    
    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    cache.insert(project_id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn save_character(
    state: State<'_, AppState>,
    project_id: Uuid,
    character: crate::models::Character,
) -> Result<ProjectMetadata, String> {
    let (root_path, mut metadata) = get_project_context(&state, project_id)?;

    if let Some(idx) = metadata
        .characters
        .iter()
        .position(|c| c.id == character.id)
    {
        metadata.characters[idx] = character;
    } else {
        metadata.characters.push(character);
    }
    metadata.updated_at = chrono::Utc::now();

    storage::save_project_metadata(&root_path, &metadata).map_err(|e| e.to_string())?;
    
    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    cache.insert(project_id, metadata.clone());

    Ok(metadata)
}

#[tauri::command]
pub async fn delete_character(
    state: State<'_, AppState>,
    project_id: Uuid,
    character_id: Uuid,
) -> Result<ProjectMetadata, String> {
    let (root_path, mut metadata) = get_project_context(&state, project_id)?;

    metadata.characters.retain(|c| c.id != character_id);
    metadata.updated_at = chrono::Utc::now();

    storage::save_project_metadata(&root_path, &metadata).map_err(|e| e.to_string())?;
    
    let mut cache = state
        .project_cache
        .lock()
        .map_err(|_| "Failed to lock cache")?;
    cache.insert(project_id, metadata.clone());

    Ok(metadata)
}
