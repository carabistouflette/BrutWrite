# BrutWrite

> The Brutalist Novel Writing Tool

BrutWrite is a desktop novel-writing application built with Tauri 2, Vue 3, TypeScript, and Rust. Project files are stored on the local filesystem, and project and file-management features use the native Tauri backend.

## Features

- Organize a manuscript as a hierarchical chapter tree.
- Write in a rich-text editor with configurable interval auto-save and per-chapter snapshots that can be restored or branched.
- Maintain character sheets, associate character aliases with manuscript text, and analyze character interactions across chapters.
- Plan narrative timelines using dates or abstract timeframes, plotlines, durations, and dependencies; inspect warnings and reorder chapters chronologically.
- Keep research files and notes beside a project, including PDF, image, Markdown, and text references.
- Track daily writing progress and a project word-count target.

PDF previews request PDF.js worker, font, and character-map assets from jsDelivr. The current Tauri Content Security Policy does not allow those cross-origin requests, so PDF rendering may fail in the desktop app; do not rely on it being available offline.

## Frontend stack

- **Vue 3 + TypeScript** for the interface, **Pinia** for shared state, and **Tiptap** for rich-text editing.
- **Vite 8** builds a static SPA consumed by the Tauri WebView; no Node.js server or SSR framework runs in the packaged app.
- **Tailwind CSS 4** uses its native Vite plugin. Design tokens live in `src/style.css`, without a separate Tailwind or PostCSS configuration.

This follows [Tauri's recommended Vite integration](https://v2.tauri.app/start/frontend/). Vue is retained to preserve the existing component and editor integrations rather than replace them with manual DOM management.

## Requirements

- Node.js 24 LTS.
- pnpm 11.28.5.
- The stable Rust toolchain, installed with [rustup](https://rustup.rs/).
- The platform-specific system dependencies required by Tauri 2. See the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

On Debian or Ubuntu, install Tauri's Linux development dependencies:

```bash
sudo apt update
sudo apt install \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

For macOS, install Xcode Command Line Tools. For Windows, install Microsoft C++ Build Tools with the Desktop development with C++ workload and the WebView2 Runtime. Consult Tauri's prerequisites for current platform-specific instructions.

## Development setup

```bash
git clone git@github.com:carabistouflette/BrutWrite.git
cd BrutWrite
npm install --global pnpm@11.28.5
pnpm install
pnpm tauri dev
```

`pnpm tauri dev` launches the Vite frontend in the native Tauri window. The frontend-only `pnpm dev` command can serve the interface in a browser, but it does not provide Tauri IPC, native file access, or native dialogs; it is not a supported way to create or manage projects.

## Development commands

Run these from the repository root:

```bash
pnpm build
pnpm type-check
pnpm lint
pnpm format:check
pnpm test:run
cargo +stable test --locked --manifest-path src-tauri/Cargo.toml
```

To build the desktop application and Linux installation packages:

```bash
pnpm tauri build
```

The current bundle configuration produces `.deb` and `.rpm` packages under
`src-tauri/target/release/bundle/`. Packaging targets for macOS and Windows must
be configured before building installers for those platforms.

See [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md), [`docs/API.md`](./docs/API.md), and [`docs/PROJECT_FORMAT.md`](./docs/PROJECT_FORMAT.md) for implementation details.

## License

BrutWrite is licensed under the GNU General Public License v3.0. See [`LICENSE`](./LICENSE).
