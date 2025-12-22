# Contributing to BrutWrite

Thank you for your interest in contributing! To ensure a high quality and maintainable codebase, please follow these guidelines.

## 1. Commit Strategy

We follow the **Conventional Commits** specification. This allows us to automatically generate changelogs and version bumps.

**Format**: `<type>(<scope>): <subject>`

### Types

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (white-space, formatting, etc)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **chore**: Changes to the build process or auxiliary tools and libraries such as documentation generation

### Example

```bash
feat(backend): implement project management commands
fix(ui): resolve overlap in sidebar menu
docs(readme): add installation instructions
```

## 2. Code Quality & Architecture

### Separation of Concerns

- **Backend (Rust)**:
  - **Logic Layer** (`storage.rs`, `models.rs`): Must be pure Rust. **Do not imports `tauri` here.** This layer should be testable in isolation.
  - **Interface Layer** (`lib.rs`): Handles `[tauri::command]`, `AppHandle`, and State management. This is the only place where Tauri dependencies should interact with your logic.
- **Frontend**: Use properly typed interfaces when communicating with the backend.

### Testing

- All backend logic must be covered by **unit tests**.
- Run `cargo test` in `src-tauri` before committing.

## 3. Workflow

1.  Create a branch for your feature (`feat/my-feature`).
2.  Implement your changes.
3.  Verify with tests.
4.  Commit using the conventional format.
