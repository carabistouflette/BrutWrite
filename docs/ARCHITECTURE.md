# System Architecture

BrutWrite follows a standard hybrid architecture using **Tauri**, which bridges a **Rust** backend with a **Vue 3/TypeScript** frontend.

## High-Level Overview

```mermaid
graph TD
    User[User Interface] <--> WebView[WebView (Frontend)]
    WebView <-->|IPC| Core[Tauri Core (Backend)]
    Core <--> FS[File System]

    subgraph Frontend [Frontend (Vue 3 + TS)]
        Store[Pinia Stores]
        Composables[Composables]
        Components[Vue Components]
        Router[Vue Router]
    end

    subgraph Backend [Backend (Rust)]
        Commands[Tauri Commands]
        State[App State]
        Logic[Domain Logic]
        IO[Storage / IO]
    end

    WebView --- Frontend
    Core --- Backend
```

## Frontend Structure (`src/`)

The frontend is a Single Page Application (SPA) built with Vite.

- **`src/api/`**: TypeScript wrappers for Tauri commands. Acts as the anti-corruption layer between frontend and backend.
- **`src/stores/`**: Pinia stores for state management (e.g., `project.ts`, `editor.ts`).
- **`src/composables/`**: Reusable stateful logic (Vue Composition API).
- **`src/components/`**: UI components.
  - **`atoms/`**: Basic building blocks (Buttons, Inputs).
  - **`molecules/`**: Combinations of atoms.
  - **`organisms/`**: Complex sections.
- **`src/views/`** (if applicable) or **`router/`**: Page definitions.

## Backend Structure (`src-tauri/src/`)

The backend handles heavy lifting, file operations, and business logic validation.

- **`lib.rs`**: Entry point. Sets up the Tauri application, menus, and state.
- **`commands/`**: The public API exposed to the frontend. Each function here is a `#[tauri::command]`. It delegates work to the Logic layer.
- **`models/`**: shared data structures (Structs, Enums). Many are serialized to JSON for the frontend.
- **`storage/`**: Handles direct file system I/O (Read/Write files, JSON serialization).
- **`research/`**: Specific logic for the Research module.
- **`integrations/`**: External tool integrations or watchers (e.g., File Watchers).
- **`state.rs`** (or in `lib.rs`): Defines the `AppState` struct managed by Tauri.

## Data Flow

1. **User Action**: User clicks "Save Chapter".
2. **Frontend**:
   - Component calls a store action `projectStore.saveChapter()`.
   - Store calls API wrapper `api.chapters.save()`.
3. **IPC**: Tauri bridges the call to Rust.
4. **Backend**:
   - `commands::chapters::save_chapter` is invoked.
   - It acquires a lock on `AppState`.
   - It calls `storage::write_chapter_file`.
   - It updates the in-memory `ProjectMetadata` (word count, etc.).
   - It returns the updated metadata.
5. **Frontend**:
   - Promise resolves with new metadata.
   - Store updates reactive state.
   - UI re-renders.

## State Management

- **Frontend**: **Pinia** is the source of truth for the UI. It mirrors the state from the backend.
- **Backend**: **`AppState`** (Mutex-protected) holds the authoritative state of the open project.
- **Synchronization**: The frontend pulls state on load and updates it after every command that modifies data.
