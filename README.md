# HaloDesk

A floating desktop AI widget for Windows and macOS, powered by [Tauri](https://tauri.app/) + [SvelteKit](https://kit.svelte.dev/).

HaloDesk pairs with **HaloRouter** — a local-only API router that runs on your machine, stores your API keys securely, and routes all AI requests to your chosen provider. There is **no cloud backend**; every AI call goes directly from your machine to the provider you configure.

---

## Table of contents

- [Prerequisites](#prerequisites)
- [Get this branch](#get-this-branch)
- [Install dependencies](#install-dependencies)
- [Run in development mode](#run-in-development-mode)
  - [Web UI only (no desktop shell)](#web-ui-only-no-desktop-shell)
  - [Full Tauri desktop app](#full-tauri-desktop-app)
    - [Windows](#windows-tauri-dev)
    - [macOS / Linux](#macos--linux-tauri-dev)
- [Build for production](#build-for-production)
- [Project structure](#project-structure)
- [Tech stack](#tech-stack)

---

## Prerequisites

Install all of the following before running the project.

### 1. Node.js ≥ 18

Download from <https://nodejs.org/> or use a version manager such as [nvm](https://github.com/nvm-sh/nvm).

```bash
node -v   # should print v18.x or higher
npm -v    # should print 9.x or higher
```

### 2. Rust (stable)

Install via <https://rustup.rs/>:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustc --version   # should print rustc 1.76 or higher
```

### 3. Tauri system dependencies

Follow the official Tauri v1 prerequisites guide for your OS:  
<https://tauri.app/v1/guides/getting-started/prerequisites>

**Windows (summary)**

- [Microsoft Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the **Desktop development with C++** workload
- [Windows 10 SDK](https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/) (installed via Build Tools or manually)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) runtime (already shipped on Windows 10/11)

**macOS (summary)**

```bash
xcode-select --install   # installs Xcode command-line tools
```

**Linux (Ubuntu/Debian summary)**

```bash
sudo apt update && sudo apt install -y \
  libwebkit2gtk-4.0-dev build-essential curl wget \
  libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

---

## Get this branch

```bash
# Clone the repository
git clone https://github.com/rahulbedjavalge/halodesk.git
cd halodesk

# Check out this branch
git checkout copilot/add-readme-execute-instructions
```

---

## Install dependencies

```bash
npm install
```

This installs all JavaScript/TypeScript dependencies (SvelteKit, Vite, Tauri CLI, etc.).  
Rust crate dependencies are downloaded automatically on the first `cargo build`, which happens inside the Tauri commands below.

---

## Run in development mode

### Web UI only (no desktop shell)

Use this mode to develop and iterate on the Svelte frontend quickly without the Rust/Tauri compile step.

```bash
npm run dev
```

Open <http://localhost:5173> in your browser. Hot-module replacement is enabled; the page updates instantly on file save.

> **Note:** Tauri APIs (global hotkeys, clipboard, window management) are not available in browser mode. Use the full Tauri dev mode below to test those features.

---

### Full Tauri desktop app

This builds the Rust backend and launches the native desktop window.

#### Windows (Tauri dev)

Run the provided helper script that sets up the Visual Studio and Windows SDK environment variables automatically:

```cmd
scripts\dev-tauri.cmd
```

Or, if your environment variables are already configured:

```cmd
npm run tauri dev
```

#### macOS / Linux (Tauri dev)

```bash
npm run tauri dev
```

The first run downloads and compiles all Rust crates, which can take **3–5 minutes**. Subsequent runs are much faster.

---

## Build for production

```bash
# 1. Build the SvelteKit frontend
npm run build

# 2. Bundle the Tauri desktop app (creates installer / .app)
npm run tauri build
```

Output locations:

| Platform | Artifact location |
|----------|------------------|
| Windows  | `src-tauri/target/release/bundle/msi/` (`.msi` installer) |
| macOS    | `src-tauri/target/release/bundle/macos/` (`.app` bundle) |
| Linux    | `src-tauri/target/release/bundle/appimage/` (`.AppImage`) |

---

## Project structure

```
halodesk/
├── src/                    # SvelteKit frontend (Svelte + TypeScript)
│   ├── app.html            # HTML shell
│   ├── app.d.ts            # TypeScript ambient declarations
│   └── routes/             # SvelteKit file-based routes
│       ├── +page.svelte    # Main HaloDesk UI
│       └── founder-docs/   # Founder documentation chatbot
├── src-tauri/              # Rust / Tauri backend (HaloRouter + desktop shell)
│   ├── src/                # Rust source (Tauri commands, HaloRouter local API)
│   ├── Cargo.toml          # Rust dependencies
│   ├── tauri.conf.json     # Tauri window and bundle configuration
│   └── icons/              # App icons
├── scripts/
│   └── dev-tauri.cmd       # Windows helper: sets MSVC env vars then runs tauri dev
├── svelte.config.js        # SvelteKit / adapter-static config
├── vite.config.ts          # Vite config
├── package.json            # npm scripts and JS dependencies
├── tsconfig.json           # TypeScript config
└── PRD.md                  # Product Requirements Document
```

---

## Tech stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | [Tauri v1](https://tauri.app/) (Rust) |
| Frontend framework | [SvelteKit](https://kit.svelte.dev/) |
| Language | TypeScript + Rust |
| Bundler | [Vite](https://vitejs.dev/) |
| Local API router | [Axum](https://github.com/tokio-rs/axum) (Rust, bundled inside Tauri) |
| Local storage | SQLite via [rusqlite](https://github.com/rusqlite/rusqlite) |
| Secure key storage | OS keychain via [keyring](https://github.com/hwchen/keyring-rs) |
| AI provider (default) | OpenRouter (user-supplied API key) |

---

## Troubleshooting

| Problem | Fix |
|---------|-----|
| `vcvars64.bat not found` (Windows) | Install **Visual Studio Build Tools 2022** with the *Desktop development with C++* workload |
| `WebView2 not found` (Windows) | Download and install the [WebView2 runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) |
| `libwebkit2gtk` missing (Linux) | Run the apt install command in the [Prerequisites](#prerequisites) section |
| Slow first `tauri dev` | Normal — Rust is compiling all crates for the first time. Wait for `Compiling halodesk` to finish |
| Port 5173 already in use | Stop any other Vite/dev servers, or set `VITE_PORT` before running `npm run dev` |
| Screen capture permission denied (macOS) | Go to **System Settings → Privacy & Security → Screen Recording** and enable HaloDesk |
