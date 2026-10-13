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

| Command                | Arguments                                                           | Return value      | Description                                                                    |
| ---------------------- | ------------------------------------------------------------------- | ----------------- | ------------------------------------------------------------------------------ |
| `update_manifest`      | `projectId: string`, `manifest: Manifest`                           | `ProjectMetadata` | Reorders/reparents existing chapters; IDs and filenames must remain unchanged. |
| `create_node`          | `projectId: string`, `parentId: string \| null`, `name: string`     | `ProjectMetadata` | Adds a chapter and its file; a non-null parent must exist.                     |
| `delete_node`          | `projectId: string`, `id: string`                                   | `ProjectMetadata` | Deletes the full subtree, its files and snapshots, and incoming dependencies.  |
| `update_node_metadata` | `projectId: string`, `nodeId: string`, `update: NodeMetadataUpdate` | `ProjectMetadata` | Updates chapter title or timeline metadata.                                    |

Manifest updates reject missing/duplicate IDs, invalid parents, cycles, dangling dependencies, and duplicate sibling orders. They preserve saved word counts and omitted timeline metadata; creation and deletion use their dedicated commands.

For optional timeline fields in `NodeMetadataUpdate`, an omitted key preserves the existing value, `null` clears it, and a string sets it. Chapter titles are display text, not filenames: punctuation such as `:`, `/`, and `?` is allowed; blank titles, control characters, and titles exceeding 256 UTF-8 bytes are rejected.

Metadata is published in memory only after `project.json` has been saved through a temporary file and rename. Deletion commits the manifest before removing subtree files; a subsequent cleanup failure is reported explicitly rather than hidden.

## Characters (`commands/characters.rs`)

| Command            | Arguments                                   | Return value      | Description                                                                                       |
| ------------------ | ------------------------------------------- | ----------------- | ------------------------------------------------------------------------------------------------- |
| `save_character`   | `projectId: string`, `character: Character` | `ProjectMetadata` | Creates or updates a character in project metadata.                                               |
| `delete_character` | `projectId: string`, `characterId: string`  | `ProjectMetadata` | Removes the character and clears matching chapter POV metadata without rewriting manuscript HTML. |

Character roles serialize as `protagonist`, `antagonist`, `secondary`, or `extra`. Alias matching is case-insensitive; orphaned mention IDs remain visible until the writer removes the reference or associates it with another character. Character sheets preserve unsaved edits until save or an explicit discard.

## Research (`commands/research.rs`)

| Command                         | Arguments                                               | Return value         | Description                                                                        |
| ------------------------------- | ------------------------------------------------------- | -------------------- | ---------------------------------------------------------------------------------- |
| `get_research_artifacts`        | —                                                       | `ResearchArtifact[]` | Lists indexed research files.                                                      |
| `add_research_files`            | `paths: string[]`                                       | `void`               | Copies the selected files into the project's research directory.                   |
| `create_research_note`          | `name: string`                                          | `ResearchArtifact`   | Creates a Markdown note in the research directory.                                 |
| `update_note_content`           | `id: string`, `expectedPath: string`, `content: string` | `void`               | Writes a research note only if its current indexed path matches the captured path. |
| `update_research_artifact_tags` | `id: string`, `tags: string[]`                          | `void`               | Updates tags and persists the research index.                                      |
| `rename_research_artifact`      | `id: string`, `newName: string`                         | `void`               | Renames a research file.                                                           |
| `delete_research_artifact`      | `id: string`                                            | `void`               | Deletes a research file.                                                           |

Research note editors capture the artifact path and flush pending or in-flight saves before project changes and native window close. A failed save retains its payload and blocks the transition; a copied artifact ID cannot write to a different project's path. When a project is moved or copied, index reconciliation preserves IDs and tags by filename and rebases paths to the current research directory.

A failed note read leaves the editor read-only and displays the error; it cannot overwrite an unread document with an empty draft. Image previews read bytes through the existing filesystem permissions and revoke their Blob URLs when selection changes.

## Snapshots (`commands/snapshots.rs`)

| Command                 | Arguments                                                                    | Return value      | Description                                                         |
| ----------------------- | ---------------------------------------------------------------------------- | ----------------- | ------------------------------------------------------------------- |
| `list_snapshots`        | `projectId: string`, `chapterId: string`                                     | `string[]`        | Lists snapshot filenames for a chapter.                             |
| `load_snapshot_content` | `projectId: string`, `chapterId: string`, `filename: string`                 | `string`          | Reads a snapshot's content.                                         |
| `create_snapshot`       | `projectId: string`, `chapterId: string`, `content: string`                  | `void`            | Creates a snapshot if its content differs from the latest snapshot. |
| `restore_snapshot`      | `projectId: string`, `chapterId: string`, `snapshotFilename: string`         | `string`          | Restores a snapshot and returns its content.                        |
| `branch_snapshot`       | `projectId: string`, `snapshotChapterId: string`, `snapshotFilename: string` | `ProjectMetadata` | Creates a new sibling chapter from a snapshot.                      |

Snapshot filenames are validated against the chapter's snapshot directory. Empty content is a valid restored version. A branch is a new sibling with its own ID/file, word count, and copied scene metadata; the frontend inserts it into the live tree before subsequent reorder operations.

## Character graph (`commands/intelligence.rs`)

| Command                   | Arguments                                                                                           | Return value            | Description                                                                  |
| ------------------------- | --------------------------------------------------------------------------------------------------- | ----------------------- | ---------------------------------------------------------------------------- |
| `analyze_character_graph` | `projectId: string`, `proximityWindow?: number`, `pruneThreshold?: number`, `chapterIds?: string[]` | `CharacterGraphPayload` | Scans selected or all chapters and returns character-interaction graph data. |

Graph scanning truncates long names and aliases only at UTF-8 character boundaries and preserves non-ASCII spelling. ASCII matching is case-insensitive. In the graph UI, clearing chapter selection analyzes all chapters; stale refresh results cannot replace a newer request or another project's graph.

The `seed_demo_project(path)` command creates the embedded example project and returns its project ID. It is registered in both development and release builds because the welcome screen offers demo creation in both.
