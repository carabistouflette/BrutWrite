# Tauri API Documentation

This document lists the available Tauri commands that can be invoked from the frontend.

## Project Management (`project.rs`)

| Command | Arguments | Return Type | Description |
|Str|Str|Str|Str|
| `create_project` | `path: string`, `name: string`, `author: string` | `ProjectMetadata` | Creates a new project structure at the specified path. |
| `load_project` | `path: string` | `ProjectMetadata` | Loads an existing project from disk. |
| `update_project_settings` | `project_id: Uuid`, `settings: ProjectSettings` | `ProjectMetadata` | Updates global settings (theme, targets, etc.). |
| `update_plotlines` | `project_id: Uuid`, `plotlines: Plotline[]` | `ProjectMetadata` | Updates the list of plotlines. |

## Chapters & Content (`chapters.rs`)

| Command | Arguments | Return Type | Description |
|Str|Str|Str|Str|
| `load_chapter_content` | `project_id: Uuid`, `chapter_id: string` | `string` (Markdown) | Reads the raw markdown content of a chapter. |
| `save_chapter` | `project_id: Uuid`, `chapter_id: string`, `content: string` | `ProjectMetadata` | Saves content to disk and updates word count metadata. |

## Manifest (`manifest.rs`)

| Command | Arguments | Return Type | Description |
|Str|Str|Str|Str|
| `create_node` | `project_id: Uuid`, `parent_id: string \| null`, `name: string` | `ProjectMetadata` | Creates a new chapter or folder in the manifest. |
| `delete_node` | `project_id: Uuid`, `id: string` | `ProjectMetadata` | Deletes a node and its files recursively. |
| `update_node_metadata` | `project_id: Uuid`, `node_id: string`, `update: NodeMetadataUpdate` | `ProjectMetadata` | Partially updates a node's properties (title, date, tags). |
| `update_manifest` | `project_id: Uuid`, `manifest: Manifest` | `ProjectMetadata` | (Advanced) Batch update the entire manifest structure (e.g., for drag-and-drop reordering). |

## Characters (`characters.rs`)

| Command | Arguments | Return Type | Description |
|Str|Str|Str|Str|
| `save_character` | `project_id: Uuid`, `character: Character` | `ProjectMetadata` | Creates or updates a character sheet. |
| `delete_character` | `project_id: Uuid`, `character_id: Uuid` | `ProjectMetadata` | Removes a character. |

## Research (`research.rs`)

| Command | Arguments | Return Type | Description |
|Str|Str|Str|Str|
| `get_research_artifacts` | - | `ResearchArtifact[]` | Returns all research items in the index. |
| `add_research_files` | `paths: string[]` | `void` | Imports external files into the research folder. |
| `create_research_note` | `name: string` | `ResearchArtifact` | Creates a new markdown note in research. |
| `update_note_content` | `id: string`, `content: string` | `void` | Updates the content of a research note. |
| `rename_research_artifact` | `id: string`, `new_name: string` | `void` | Renames a research artifact file. |
| `delete_research_artifact` | `id: string` | `void` | Deletes a research artifact. |
