# BrutWrite Project Format Specification

This document defines the specialized file structure and data models for a BrutWrite project.

## Directory Structure

A BrutWrite project is a directory with a `.brut` extension (conceptually) or just a folder.

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
│   ├── image.png
│   └── notes.md
└── .snapshots/        # Global or Chapter-level snapshots
    └── ...
```

## Data Models

### Project Metadata (`project.json`)

```json
{
  "id": "uuid-v4",
  "title": "My Great Novel",
  "author": "Jane Doe",
  "created_at": "ISO-8601",
  "updated_at": "ISO-8601",
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
