# Composables

This directory contains reusable state and logic (composables) for the BrutWrite application.

## Structure

```
src/composables/
├── domain/           # Domain-specific logic (Business Logic)
│   ├── project/      # Project management, file I/O, session
│   ├── characters/   # Character management
│   ├── timeline/     # Timeline logic, paradoxes, time helpers
│   └── gamification/ # Writing stats and gamifiction
├── editor/           # Editor-specific logic (Tiptap, AutoSave)
├── timeline/         # Timeline UI logic (VisTimeline wrappers)
└── ui/               # Generic UI logic (Theme, Windows, Resizing)
```

## Domain Composables

| Composable | Domain | Responsibility |
|Data Store|---|---|
| `useProjectIO` | Project | Handling Loading/Saving/Creating projects and file system interactions. |
| `useProjectSession` | Project | Managing active project session, auto-save triggers, and state restoration. |
| `useProjectNodeOperations` | Project | CRUD operations for FileNodes (Chapters, Folders) in the project tree. |
| `useProjectSync` | Project | Syncing state changes to the backend/manifest. |
| `useRecentProjects` | Project | Managing the list of recently opened projects. |
| `useChapterSession` | Project | Managing the currently active chapter content and editing session. |
| `useCharacters` | Characters | CRUD operations for Characters. |
| `useCharacterSheetLogic` | Characters | Form logic for the character sheet UI. |
| `usePlotlines` | Timeline | Managing plotlines (creation, updates, colors). |
| `useTimeHelpers` | Timeline | Utilities for parsing and formatting abstract time durations. |
| `useParadoxDetection` | Timeline | Detecting logical inconsistencies in the timeline events. |
| `useGamification` | Gamification | Tracking word counts, writing streaks, and daily goals. |

## UI & Editor Composables

| Composable        | Category    | Responsibility                                                  |
| ----------------- | ----------- | --------------------------------------------------------------- |
| `useTheme`        | UI          | Managing Dark/Light mode and theme preferences.                 |
| `useAppStatus`    | UI          | Global status bar messages (Loading, Saving, Error).            |
| `useTiptapEditor` | Editor      | Wrapper around Tiptap editor instance and configuration.        |
| `useAutoSave`     | Editor      | Debounced auto-save logic for the editor content.               |
| `useVisTimeline`  | Timeline UI | Wrapper around the VisTimeline library for the visual timeline. |
