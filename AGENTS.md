# AGENTS.md — Bassical

Instructions for AI coding agents working in this repository.

## Project Overview

Bassical is a Tauri 2 desktop app for electric bass practice. Frontend: React 19 + TypeScript + Tailwind CSS 4 + Zustand. Backend: Rust. Targets Windows 10/11.

## Build / Dev / Lint Commands

```bash
# Install frontend deps
pnpm install

# Dev mode (starts both Vite frontend + Rust backend with hot-reload)
pnpm tauri dev

# Typecheck frontend
pnpm exec tsc --noEmit

# Build frontend only (typecheck + vite build)
pnpm build

# Build production installer (.exe)
pnpm tauri build

# Rust: check backend compiles
cargo check            # run from src-tauri/

# Rust: run clippy linter
cargo clippy -- -D warnings   # run from src-tauri/

# Rust: format
cargo fmt              # run from src-tauri/

# Rust: run tests
cargo test             # run from src-tauri/
```

No frontend linter (ESLint/Biome) or test framework (Vitest/Jest) is configured yet. When adding one, update this section.

## Architecture

Two-layer architecture connected via Tauri IPC:

- **Frontend** (`src/`): React components, Zustand stores, IPC calls via `invoke()` from `@tauri-apps/api/core`
- **Backend** (`src-tauri/src/`): Rust modules — `commands/` (IPC handlers), `models/` (domain types), `persistence/` (JSON storage), `audio/`, `calibration/`, `parser/`

IPC pattern: Frontend calls `invoke<ReturnType>('command_name', { args })`. Backend exposes `#[tauri::command]` functions registered in `lib.rs`.

## Code Style — TypeScript / React

- **Strict mode** enabled in tsconfig (`strict: true`, `noUnusedLocals`, `noUnusedParameters`)
- **Path alias**: `@/*` maps to `src/*` — use `@/lib/types` not `../../../lib/types`
- **Components**: Functional components with named exports, no default exports for components
  ```tsx
  export function MyComponent({ prop }: MyComponentProps) { ... }
  ```
- **Views**: Place in `src/views/`, named exports (e.g., `LibraryView.tsx`)
- **Component folders**: `src/components/FeatureName/index.tsx` pattern
- **Types**: Centralized in `src/lib/types/`. Use interfaces, not type aliases for objects. Use camelCase for properties.
- **State**: Zustand stores in `src/lib/store/`. Export via barrel `index.ts`. Store pattern:
  ```ts
  export const useMyStore = create<MyState>((set) => ({ ... }));
  ```
- **IPC calls**: Wrap in `src/lib/` modules (e.g., `audio.ts`, `calibration.ts`), not directly in components
- **Imports order**: External libs → `@/` aliased imports → relative imports (no enforced rule, but follow existing pattern)
- **Styling**: Tailwind CSS 4 utility classes directly in JSX. No CSS modules. Single `App.css` with `@import "tailwindcss"`
- **No comments** unless requested by the user

## Code Style — Rust

- **Edition**: 2021
- **Naming**: snake_case for functions/variables, CamelCase for types, SCREAMING_SNAKE_CASE for constants
- **Newtype pattern** for domain primitives: `SongId(String)`, `AudioPath(String)`, `SongTitle(String)` — defined in `models/song.rs`
- **Serialization**: `serde` with `#[serde(rename_all = "camelCase")]` for JSON interop with frontend
- **Error handling**: Commands return `Result<T, String>`. Use `.map_err(|e| e.to_string())` to convert errors.
- **Module structure**: Each domain area is a module with `mod.rs` + submodules. Public API exposed via `pub mod`.
- **Tauri commands**: `#[tauri::command]` attribute, parameters are deserialized from frontend args
- **Persistence**: `persistence/storage.rs` provides generic `read_json<T>` / `write_json<T>` using `serde_json`. Data stored in `%APPDATA%\Bassical\`.
- **No comments** unless requested by the user

## Data Conventions

- **IDs**: UUID v4 strings
- **Timestamps**: RFC 3339 strings (via `chrono`)
- **JSON schema versioned**: `"schema_version": 1` in `.bassical.json` files
- **Property casing**: camelCase in JSON (frontend + serde `rename_all`), snake_case in Rust source
- **Audio paths**: Absolute paths to user's files — never copy or modify audio files

## Project Structure Reference

```
src/                          # Frontend
├── components/Layout/        # Shared layout components
├── views/                    # Page-level view components
├── lib/
│   ├── audio.ts              # IPC wrappers for audio commands
│   ├── types/                # TypeScript interfaces
│   └── store/                # Zustand state stores
├── App.tsx                   # Root component
└── main.tsx                  # React entry point

src-tauri/src/                # Backend
├── commands/                 # #[tauri::command] IPC handlers
│   ├── mod.rs
│   └── library.rs
├── models/                   # Domain types (newtypes, structs)
│   ├── song.rs
│   └── tab.rs
├── persistence/              # JSON read/write to AppData
│   └── storage.rs
├── audio/                    # Audio engine (cpal) - Sprint 3
├── calibration/              # Timing point logic - Sprint 4
├── parser/                   # Guitar Pro parser - Sprint 6
├── lib.rs                    # Tauri builder + command registration
└── main.rs                   # Entry point
```

## Sprint Status

Currently in **Sprint 1** (Fundamentals). Audio, calibration, parser modules are stubs. When implementing new commands: add to `commands/`, register in `lib.rs` `invoke_handler`, create IPC wrapper in `src/lib/`.

## Key Constraints

- Windows 10/11 only (v1.0)
- Audio files are never modified — store paths only
- Calibration tap latency ≤ 10 ms — avoid blocking Rust main thread
- RAM ≤ 300 MB during 30-min session
- No internet, no auth, no telemetry
