# AGENTS.md

## Git Workflow

Always use **no-fast-forward merges** to keep the git graph visible:

```bash
git merge --no-ff <branch>
```

This creates merge commits that produce a readable branch graph instead of a flat linear history.

### Branching Convention

- `feat/*` — new features
- `fix/*` — bug fixes
- `refactor/*` — code restructuring
- `chore/*` — maintenance tasks

### Commit Messages

Use conventional commits:

```
feat: add new feature
fix: resolve issue
refactor: improve code structure
```

### Workflow

1. Create a feature branch from `main`
2. Make commits on the feature branch
3. Open a PR or merge locally with `--no-ff`
4. Delete the branch after merge

### Example

```bash
git checkout -b feat/my-feature
# ... make commits ...
git checkout main
git merge --no-ff feat/my-feature -m "Merge branch 'feat/my-feature'"
git branch -d feat/my-feature
```

---

## Project Overview

MineMate is an AI-powered Minecraft bot desktop app built with **Tauri v2** (Rust backend + React frontend). The bot joins Minecraft servers as a player while acting as an AI assistant for all players.

### Tech Stack

- **Backend**: Rust (nightly required for Azalea's `portable_simd`)
- **Frontend**: React 18 + TypeScript + Vite
- **Desktop Framework**: Tauri v2
- **Minecraft Client**: Azalea (Rust)
- **AI**: NVIDIA NIM API (Llama 3.3 70B / Qwen / DeepSeek)
- **Database**: SQLite
- **Config**: TOML

---

## Development Commands

### Run in Development Mode

```bash
cargo tauri dev
```

This runs both the Vite dev server (frontend) and compiles/launches the Rust backend.

### Build for Production

```bash
cargo tauri build
```

### Frontend Only

```bash
npm run dev
```

### Install Dependencies

```bash
npm install
```

### Rust Prerequisites

```bash
rustup install nightly
rustup default nightly
```

---

## Project Structure

```
minemate/
├── src-tauri/                 # Rust backend
│   ├── src/
│   │   ├── ai/               # NVIDIA NIM client, tools, context builder
│   │   ├── bot/              # Azalea client wrapper, event system
│   │   ├── commands/         # Tauri IPC commands
│   │   ├── config/           # TOML config management
│   │   ├── executor/         # Tool execution, automations, security
│   │   └── memory/           # SQLite database operations
│   └── Cargo.toml
├── src/                       # React frontend
│   ├── components/
│   │   ├── shell/            # TopNavBar, SideNavBar
│   │   ├── dashboard/        # HUD, PlayerList, EventLog
│   │   ├── chat/             # Chat messages, input
│   │   ├── config/           # Settings panel
│   │   └── tasks/            # Task queue
│   ├── hooks/                # Tauri IPC hooks
│   └── styles/               # Pixel-Brutalism CSS
├── config/                    # Default TOML config (gitignored, generated at runtime)
└── database/                  # SQLite database (gitignored)
```

---

## Key Architecture Notes

### Bot System (`src-tauri/src/bot/`)
- `client.rs`: Core `BotClient` struct wrapping Azalea with shared state (`Arc<RwLock<>>`)
- `handler.rs`: Event handling for Minecraft protocol events
- `follow.rs`: Player following logic
- `guard.rs`: Guard/protection mode

### AI System (`src-tauri/src/ai/`)
- 14 approved tools: move, mine, build, craft, attack, place_block, build_structure, reply, execute_command, scan_area, give_item, teleport, sort_chests, protect_player
- Uses NVIDIA NIM API for LLM inference

### Tauri IPC Commands (`src-tauri/src/commands/`)
- All Tauri commands registered in `lib.rs` via `invoke_handler`
- Frontend calls these via `@tauri-apps/api`

### Configuration
- Runtime config stored in OS app data dir (`dirs::config_dir()`)
- Path: `{config_dir}/MineMate/config/default.toml`
- Config file is **gitignored** — never commit API keys or server addresses

---

## Important Quirks

1. **Rust nightly is mandatory** — Azalea requires `#![feature(portable_simd)]`
2. **Debug builds are slow** — Azalea's ECS systems run 10-100x slower unoptimized in debug; dev profile sets `opt-level = 2` for dependencies
3. **Config is not in repo** — generated at runtime in app data dir; `config/default.toml` is gitignored
4. **Tauri security** — CSP is set to `null` in tauri.conf.json (needed for AI API calls)

---

## Testing

No test suite is currently configured. When tests are added:
- Rust: `cargo test`
- Frontend: check for test framework setup in package.json
