# BrutWrite

> **The Brutalist Novel Writing Tool**

BrutWrite is a distraction-free, privacy-focused application for writers who want to focus on their words, not the tool. Built with Tauri, Vue 3, and Rust, it combines the performance of a native app with the flexibility of web technologies.

![App Screenshot](docs/assets/screenshot_placeholder.png)
_(Screenshot placeholder - to be added)_

## ✨ Features

- **Project Management**: Organize your novel with a structured chapter hierarchy.
- **Character Sheets**: Keep track of your cast with detailed profiles.
- **Research Hub**: Manage images, notes, and references alongside your manuscript.
- **Distraction-Free Editor**: A clean, brutalist interface that puts your text front and center.
- **Local First**: Your data lives on your machine in standard file formats (Markdown, JSON). No proprietary lock-in.
- **Fast**: Built on Rust and Tauri for blazing fast performance.

## 🚀 Installation

### Prerequisites

- **Node.js** & **pnpm**: For the frontend.
- **Rust Toolchain**: For the backend.

### Setup

1. **Clone the repository**

   ```bash
   git clone https://github.com/your-username/BrutWrite.git
   cd BrutWrite
   ```

2. **Install dependencies**

   ```bash
   pnpm install
   ```

3. **Run in Development Mode**
   ```bash
   # Starts the Vue dev server and the Tauri window
   pnpm tauri dev
   ```

## 🛠️ Development

- **Frontend**: Vue 3 + TypeScript + Vite. Located in `src/`.
- **Backend**: Rust + Tauri. Located in `src-tauri/`.

For detailed architecture documentation, see [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md).

## 📄 License

MIT License. See [LICENSE](./LICENSE) for details.
