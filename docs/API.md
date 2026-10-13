# Tauri API

The Rust commands below are registered in `src-tauri/src/lib.rs` and called by the frontend through Tauri IPC. Argument names are shown in the camelCase form used by the TypeScript wrappers; Rust `Uuid` values are represented as strings in the frontend.

## Projects (`commands/project.rs`)

| Command                   | Arguments                                        | Return value      | Description                                                 |
| ------------------------- | ------------------------------------------------ | ----------------- | ----------------------------------------------------------- |
| `create_project`          | `path: string`, `name: string`, `author: string` | `ProjectMetadata` | Creates a project at `path`.                                |
| `load_project`            | `path: string`                                   | `ProjectMetadata` | Loads a project from a directory containing `project.json`. |
| `update_project_settings` | `projectId: string`, `settings: ProjectSettings` | `ProjectMetadata` | Updates project-level writing targets.                      |
| `update_plotlines`        | `projectId: string`, `plotlines: Plotline[]`     | `ProjectMetadata` | Replaces the project's plotline list.                       |

## Chapters (`commands/chapters.rs`)

| Command                | Arguments                                                   | Return value      | Description                                                                                                        |
| ---------------------- | ----------------------------------------------------------- | ----------------- | ------------------------------------------------------------------------------------------------------------------ |
| `load_chapter_content` | `projectId: string`, `chapterId: string`                    | `string`          | Returns the chapter content string.                                                                                |
| `save_chapter`         | `projectId: string`, `chapterId: string`, `content: string` | `ProjectMetadata` | Saves the content and updates the chapter word count. The editor sends HTML, stored in a file with a `.md` suffix. |

## Chapter tree (`commands/manifest.rs`)

| Command                | Arguments                                                           | Return value      | Description                                          |
| ---------------------- | ------------------------------------------------------------------- | ----------------- | ---------------------------------------------------- |
| `update_manifest`      | `projectId: string`, `manifest: Manifest`                           | `ProjectMetadata` | Replaces the project's manifest.                     |
| `create_node`          | `projectId: string`, `parentId: string \| null`, `name: string`     | `ProjectMetadata` | Adds a chapter to the manifest and creates its file. |
| `delete_node`          | `projectId: string`, `id: string`                                   | `ProjectMetadata` | Deletes a chapter-tree node.                         |
| `update_node_metadata` | `projectId: string`, `nodeId: string`, `update: NodeMetadataUpdate` | `ProjectMetadata` | Updates chapter title or timeline metadata.          |

## Characters (`commands/characters.rs`)

| Command            | Arguments                                   | Return value      | Description                                         |
| ------------------ | ------------------------------------------- | ----------------- | --------------------------------------------------- |
| `save_character`   | `projectId: string`, `character: Character` | `ProjectMetadata` | Creates or updates a character in project metadata. |
| `delete_character` | `projectId: string`, `characterId: string`  | `ProjectMetadata` | Removes a character from project metadata.          |

## Research (`commands/research.rs`)

| Command                    | Arguments                       | Return value         | Description                                                      |
| -------------------------- | ------------------------------- | -------------------- | ---------------------------------------------------------------- |
| `get_research_artifacts`   | —                               | `ResearchArtifact[]` | Lists indexed research files.                                    |
| `add_research_files`       | `paths: string[]`               | `void`               | Copies the selected files into the project's research directory. |
| `create_research_note`     | `name: string`                  | `ResearchArtifact`   | Creates a Markdown note in the research directory.               |
| `update_note_content`      | `id: string`, `content: string` | `void`               | Writes a research note's content.                                |
| `rename_research_artifact` | `id: string`, `newName: string` | `void`               | Renames a research file.                                         |
| `delete_research_artifact` | `id: string`                    | `void`               | Deletes a research file.                                         |

## Snapshots (`commands/snapshots.rs`)

| Command                 | Arguments                                                                    | Return value      | Description                                                         |
| ----------------------- | ---------------------------------------------------------------------------- | ----------------- | ------------------------------------------------------------------- |
| `list_snapshots`        | `projectId: string`, `chapterId: string`                                     | `string[]`        | Lists snapshot filenames for a chapter.                             |
| `load_snapshot_content` | `projectId: string`, `chapterId: string`, `filename: string`                 | `string`          | Reads a snapshot's content.                                         |
| `create_snapshot`       | `projectId: string`, `chapterId: string`, `content: string`                  | `void`            | Creates a snapshot if its content differs from the latest snapshot. |
| `restore_snapshot`      | `projectId: string`, `chapterId: string`, `snapshotFilename: string`         | `string`          | Restores a snapshot and returns its content.                        |
| `branch_snapshot`       | `projectId: string`, `snapshotChapterId: string`, `snapshotFilename: string` | `ProjectMetadata` | Creates a new sibling chapter from a snapshot.                      |

## Character graph (`commands/intelligence.rs`)

| Command                   | Arguments                                                                                           | Return value            | Description                                                                  |
| ------------------------- | --------------------------------------------------------------------------------------------------- | ----------------------- | ---------------------------------------------------------------------------- |
| `analyze_character_graph` | `projectId: string`, `proximityWindow?: number`, `pruneThreshold?: number`, `chapterIds?: string[]` | `CharacterGraphPayload` | Scans selected or all chapters and returns character-interaction graph data. |

The `seed_demo_project(path)` command is registered only in debug builds. Its module and embedded demo data are excluded from release builds.
