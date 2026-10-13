# BrutWrite Project Format

A project is a directory selected by the user. It does not require a special filename extension. `project.json` is the required project metadata file; manuscript and research files are stored alongside it.

## Directory layout

```text
MyNovel/
├── project.json
├── manuscript/
│   ├── chapter-550e8400-e29b-41d4-a716-446655440001.md
│   └── .snapshots/
│       └── chapter-550e8400-e29b-41d4-a716-446655440001/
│           └── 2026-10-13T120000_01234567.md
└── research/
    ├── .research-index.json
    ├── source.pdf
    └── research-note.md
```

The project manager creates `project.json` when a project is created. Chapter files are created under `manuscript/` as chapters are added. Imported research files and Markdown notes are stored directly in `research/`; `.research-index.json` tracks their metadata. Character sheets and plotlines are stored in `project.json`, not in separate `characters/` files.

## Project metadata (`project.json`)

`ProjectMetadata` contains the project identity, timestamps, settings, chapter manifest, characters, and plotlines. The following abbreviated example shows the serialized field names:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "The Glass Harbor",
  "author": "A. Writer",
  "created_at": "2026-10-13T12:00:00Z",
  "updated_at": "2026-10-13T12:00:00Z",
  "settings": {
    "daily_target": 2000,
    "word_target": 50000
  },
  "manifest": {
    "chapters": [
      {
        "id": "chapter-550e8400-e29b-41d4-a716-446655440001",
        "parent_id": null,
        "title": "Opening",
        "filename": "chapter-550e8400-e29b-41d4-a716-446655440001.md",
        "word_count": 0,
        "order": 0,
        "chronological_date": null,
        "abstract_timeframe": null,
        "duration": null,
        "plotline_tag": null,
        "depends_on": null,
        "pov_character_id": null
      }
    ]
  },
  "characters": [],
  "plotlines": [
    {
      "id": "main",
      "name": "Main Plot",
      "color": "#3b82f6"
    }
  ]
}
```

`settings` contains `daily_target` and `word_target`. Each manifest chapter records its `id`, optional `parent_id`, title, filename, word count, sibling order, and optional timeline fields. `chronological_date` stores an ISO 8601 date/time; `abstract_timeframe` can represent story time such as `"Day 1"`; `duration`, `plotline_tag`, `depends_on`, and `pov_character_id` provide additional scene metadata.

Chapter files use a `.md` filename suffix, but the rich-text editor currently saves its HTML serialization in those files. The content is not guaranteed to be plain Markdown.

## Character sheets

Characters are entries in the `characters` array of `project.json`. Roles serialize as `protagonist`, `antagonist`, `secondary`, or `extra`.

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440002",
  "name": "Mara Venn",
  "role": "protagonist",
  "archetype": "",
  "description": "A harbor pilot with a hidden past.",
  "engine": {
    "desire": "Find her missing brother.",
    "fear": "Being responsible for another loss.",
    "wound": "",
    "secret": ""
  },
  "physical_features": "",
  "traits": ["observant", "stubborn"],
  "arc": "",
  "notes": "",
  "aliases": ["Mara"]
}
```

## Research index (`research/.research-index.json`)

The research directory contains imported files and Markdown notes. The hidden JSON index is an object keyed by artifact ID; each artifact records `id`, `path`, `name`, `file_type`, and `tags`. Supported `file_type` values include `pdf`, `image`, `text`, and `other`.

```json
{
  "550e8400-e29b-41d4-a716-446655440003": {
    "id": "550e8400-e29b-41d4-a716-446655440003",
    "path": "/home/writer/MyNovel/research/source.pdf",
    "name": "source.pdf",
    "file_type": "pdf",
    "tags": []
  }
}
```

## Snapshots

In the example project, chapter snapshots are stored under `manuscript/.snapshots/chapter-550e8400-e29b-41d4-a716-446655440001/`. Snapshot filenames contain a UTC timestamp and a short content hash. Saving identical content consecutively does not create another snapshot, and the application retains at most 50 snapshots per chapter.
