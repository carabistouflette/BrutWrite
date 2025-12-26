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

### Backend (Rust)

- **Style**: We use standard `rustfmt` and `clippy`. Please run `cargo clippy` and `cargo fmt` before submitting.
- **Separation of Concerns**:
  - **Logic Layer** (`storage.rs`, `models.rs`): Must be pure Rust. **Do not imports `tauri` here.** This layer should be testable in isolation.
  - **Interface Layer** (`lib.rs`, `commands/`): Handles `[tauri::command]`, `AppHandle`, and State management. This is the only place where Tauri dependencies should interact with your logic.
- **Testing**: All backend logic must be covered by **unit tests**. Run `cargo test` in `src-tauri` before committing.

### Frontend (Vue 3 + TypeScript)

- **Composition API**: Use `<script setup lang="ts">` for all components.
- **Naming**:
  - Components: generic names (e.g., `Button.vue`), or specific (e.g., `ProjectList.vue`). PascalCase.
  - Composables: `use` prefix (e.g., `useProject.ts`).
- **State Management**: Use **Pinia** for global state. Avoid deeply nested props.
- **Typing**: Strict TypeScript. Avoid `any`. Define interfaces in `src/types/` or co-located if specific.

## 3. Pull Request Workflow

1.  **Fork & Clone**: Fork the repository and clone it locally.
2.  **Branch**: Create a branch for your feature (`feat/my-feature`).
3.  **Implement**: Make your changes.
4.  **Verify**:
    - Run `pnpm type-check` (frontend).
    - Run `cargo test` (backend).
    - Run linter/formatter.
5.  **Commit**: Use the conventional format.
6.  **Push & PR**: Push to your fork and open a Pull Request.
7.  **Review**: Wait for a maintainer to review your code. Address any feedback.
