# Contributing to BrutWrite

Thank you for contributing. Install the prerequisites and dependencies described in the [README](./README.md), then work from the repository root.

## Commit messages

Use a Conventional Commit-style subject to make the history easy to scan. This format does not by itself generate changelogs or version bumps.

**Format:** `<type>(<scope>): <subject>`

Common types include:

- **feat**: a new feature
- **fix**: a bug fix
- **docs**: documentation changes
- **style**: changes that do not affect program behavior
- **refactor**: a code change that neither fixes a bug nor adds a feature
- **perf**: a performance improvement
- **test**: adding or updating tests
- **chore**: build or maintenance work

Examples:

```text
feat(editor): add chapter snapshots
fix(research): refresh the artifact list after import
docs(readme): clarify native development setup
```

## Code conventions

### Rust backend

- Keep Tauri command entry points and application-state access in `src-tauri/src/commands/`; keep serialized domain types in `models/` and filesystem persistence in `storage/`.
- Follow `rustfmt` and Clippy conventions. For Rust changes, check formatting with `cargo +stable fmt --manifest-path src-tauri/Cargo.toml -- --check` and run `cargo +stable clippy --manifest-path src-tauri/Cargo.toml --all-targets`.
- Add or update focused tests for changed Rust behavior. Run backend tests with `cargo +stable test --locked --manifest-path src-tauri/Cargo.toml`.

### Frontend

- Use Vue's Composition API and `<script setup lang="ts">` for components.
- Use PascalCase for Vue component filenames and the `use` prefix for composables.
- Use Pinia for shared application state and keep TypeScript types in `src/types/` or next to the code that uses them.
- Follow the existing component styling patterns: scoped styles for component-specific CSS, shared styles in `src/style.css`, and utility classes where already used.
- Use plain `<style scoped>` blocks, not `lang="postcss"`. Tailwind runs through `@tailwindcss/vite`; use `@reference` to `src/style.css` when a scoped block needs theme tokens or `@apply`. Keep theme tokens in the shared CSS `@theme` block.

## Checks

Run the project checks from the repository root before opening a pull request:

```bash
pnpm type-check
pnpm lint
pnpm format:check
pnpm test:run
cargo +stable test --locked --manifest-path src-tauri/Cargo.toml
```

For UI changes, also run `pnpm tauri dev` and verify the affected flow in the native application. `pnpm dev` starts only the Vite frontend; native Tauri commands and file dialogs are unavailable in that browser-only session.

## Dependency maintenance

Keep direct npm dependencies on stable releases. This dependency set keeps TypeScript at `~6.0.3`, the only direct dependency not on the latest stable release, because `typescript-eslint` currently declares the peer range `>=4.8.4 <6.1`.

The pnpm overrides for `source-map-js`, `uuid`, and `moment` target transitive dependencies with security advisories. Reassess or remove an override when upstream dependency updates resolve the issue. The dependency refresh was checked with `pnpm audit`, which reported no advisories.

CI currently uses `actions/checkout@v7.0.1`, `actions/setup-node@v7.1.0`, `actions/upload-artifact@v7.0.2`, and `pnpm/action-setup@v6.1.0`. Keep `pnpm-lock.yaml` excluded from Prettier; regenerate it with pnpm when dependency manifests change.

## Pull request workflow

1. Fork the repository and create a branch for your change, for example `feat/chapter-history`.
2. Make the change and update relevant existing documentation when behavior or setup changes.
3. Run the checks above and describe the behavior changed in your pull request.
4. Push your branch and open a pull request. Address review feedback from maintainers.
