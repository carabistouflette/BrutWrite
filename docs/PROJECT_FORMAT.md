# BrutWrite Project Format Specification

This document defines the specialized file structure and data models for a BrutWrite project.

## Directory Structure

A BrutWrite project is a directory with a `.brut` extension (conceptually) or just a standard folder.

```
MyBook/
├── project.json       # Project metadata and configuration
├── manuscript/        # The ordered content of the book
│   ├── 01-chapter1.md
│   ├── 02-chapter2.md
│   └── 99-notes.md
├── characters/        # Character sheets
│   ├── alice.json
│   └── bob.json
├── research/          # Research materials
│   ├── index.json     # Index of research artifacts
│   ├── images/        # Image assets
│   │   └── reference.png
│   └── notes/         # Research notes (markdown)
│       └── history.md
└── .snapshots/        # Global or Chapter-level snapshots
    └── ...
```

## Data Models

### Project Metadata (`project.json`)

Contains global settings and the ordered manifest of chapters.

```json
{
  "id": "uuid-v4",
  "title": "My Great Novel",
  "author": "Jane Doe",
  "created_at": "2023-10-27T10:00:00Z",
  "updated_at": "2023-10-28T15:30:00Z",
  "settings": {
    "daily_target": 2000,
    "theme": "brutalist-dark"
  },
  "manifest": {
    "chapters": [
      {
        "id": "chap-1",
        "title": "Chapter 1",
        "filename": "01-chapter1.md",
        "order": 1
      },
      {
        "id": "chap-2",
        "title": "Chapter 2",
        "filename": "02-chapter2.md",
        "order": 2
      }
    ]
  }
}
```

### Character Sheet (`characters/*.json`)

Stores details about characters in the story.

```json
{
  "id": "char-uuid",
  "name": "Alice Corp",
  "role": "protagonist", // protagonist, antagonist, secondary, extra
  "description": "A cyberpunk hacker with a grudge.",
  "traits": ["smart", "cynical"],
  "arc": "Learns to trust again.",
  "notes": "Markdown content for extra notes..."
}
```

### Research Index (`research/index.json`)

The research module tracks files in the `research/` folder. It maintains an index to quickly load metadata without scanning all files.

```json
[
  {
    "id": "res-uuid-1",
    "type": "image",
    "filename": "images/map.png",
    "title": "World Map",
    "tags": ["location", "reference"]
  },
  {
    "id": "res-uuid-2",
    "type": "note",
    "filename": "notes/magic-system.md",
    "title": "Magic System Definition",
    "tags": ["worldbuilding"]
  }
]
```

### Snapshots (`.snapshots/`)

Snapshots are backups of specific files or the whole project state at a point in time. They are managed internally by the application.
