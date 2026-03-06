# Yggdrasil — Unified Introspection Shell

## Context Refresh

**READ THESE FILES to understand the current state before making changes:**

- This plan: `introspection/PLAN_UNIFIED_SHELL.md`
- Workspace root: `introspection/Cargo.toml` (once created)
- Each app's core crate: `introspection/core/*/src/lib.rs`
- Each app's Tauri shell: `introspection/*/src-tauri/src/lib.rs`
- Yggdrasil's Tauri shell: `introspection/yggdrasil/src-tauri/src/lib.rs`
- Frontend views: `introspection/*/src/lib/*View.svelte`
- Yggdrasil page: `introspection/yggdrasil/src/routes/+page.svelte`
- Shared UI: `introspection/ui/` (components + CSS tokens)

**Key constraints:**
- Global gitignore at `~/.config/git/ignore` line 43 excludes `lib/`.
  Files in `src/lib/` must be added with `git add -f`.
- Tauri apps need `tauri.conf.json`, `build.rs`, `main.rs` in `src-tauri/`.
- Each app's `lib.rs` calls `tauri::generate_context!()` which needs
  `tauri.conf.json` relative to the crate root.

## Goal

A single app (Yggdrasil) that hosts all four introspection apps as switchable
views, with a vertical right-edge tab strip showing full app names. Each app
also continues to work as a standalone Tauri binary.

All five apps share a single Cargo workspace and build target directory.
Deployment is via symlinks from `/Applications/` to the build output.

## Tech Stack

- **Tauri 2.x** — Rust backend + webview frontend
- **Svelte 5** — runes (`$state`, `$derived`, `$props`), snippets (`{#snippet}`, `{@render}`)
- **SvelteKit** — static adapter, `src/routes/+page.svelte` entry point
- **@introspection/ui** — shared Svelte component library (SidebarLayout, Button, ListItem, tokens.css, base.css)
- **Cargo workspace** — shared `target/` directory, workspace dependencies
- **Build**: `cd introspection/<app> && npm run tauri build`
- **Deploy**: symlinks from `/Applications/<App>.app` to `introspection/target/release/bundle/macos/<app>.app`

## The Four Apps

| Tab | Prefix | App        | Purpose                              |
|-----|--------|------------|--------------------------------------|
| H   | hlid_  | Hlidskjalf | Real-time agent activity monitor     |
| S   | sval_  | Svalinn    | Code quality viewer (saga/gleipnir)  |
| K   | kvas_  | Kvasir     | Workspace inspector + schema inspect |
| R   | rata_  | Ratatoskr  | Graph viewer with D3 visualization   |

Tab order: H / S / K / R (top to bottom, full names spelled vertically).

## Architecture

### Directory Structure

```
introspection/
  Cargo.toml                    # Workspace root
  .cargo/
    config.toml                 # target-dir = "target"
  target/                       # ALL builds land here (shared)
    release/bundle/macos/       # .app bundles

  core/                         # Backend library crates (NO Tauri dependency)
    hlidskjalf_core/            # Socket listener, HookEvent struct
      Cargo.toml                # deps: tokio, serde, serde_json
      src/lib.rs
    svalinn_core/               # Sidecar scanning, Issue structs, saga runner
      Cargo.toml                # deps: serde, serde_json, glob, dirs
      src/lib.rs
    kvasir_core/                # File browsing, format conversion, language detect
      Cargo.toml                # deps: serde, serde_json, serde_yaml, toml, serde_toon2
      src/lib.rs
    ratatoskr_core/             # Graph loading, JSON-LD parsing, merge config
      Cargo.toml                # deps: serde, serde_json
      src/lib.rs

  hlidskjalf/                   # Standalone Tauri app
    package.json
    svelte.config.js
    vite.config.js
    src/
      lib/
        HlidskjalfView.svelte   # All UI logic (git add -f required)
        GleipnirReport.svelte   # Gleipnir/syn report renderer
      routes/
        +page.svelte            # Thin wrapper: <HlidskjalfView />
    src-tauri/
      Cargo.toml                # deps: hlidskjalf_core, tauri, tauri-plugin-*
      tauri.conf.json
      build.rs
      src/
        main.rs                 # hlidskjalf_lib::run()
        lib.rs                  # Thin: #[tauri::command] wrappers + run()

  svalinn/                      # Same pattern as hlidskjalf
  kvasir/                       # Same pattern (+ SchemaInspector, schema-inspect.ts)
  ratatoskr/                    # Same pattern

  yggdrasil/                    # Unified shell
    package.json                # deps include d3, highlight.js, marked
    svelte.config.js
    vite.config.js              # Vite aliases: $hlidskjalf, $svalinn, $kvasir, $ratatoskr
    src/
      routes/
        +page.svelte            # Imports all 4 views, tab strip, command maps
    src-tauri/
      Cargo.toml                # deps: ALL 4 core crates, tauri, tauri-plugin-*
      tauri.conf.json
      build.rs
      src/
        main.rs                 # yggdrasil_lib::run()
        lib.rs                  # ALL commands registered with prefixed names

  ui/                           # Shared Svelte component library (unchanged)
    components/
    css/
```

### Core/Tauri Split

**Core crates** contain all business logic with NO Tauri dependency:
- Data structures (structs, enums)
- File I/O, parsing, scanning, format conversion
- Functions return `Result<T, String>`
- No `#[tauri::command]`, no `tauri::AppHandle`

**App Tauri crates** are thin shells:
- `#[tauri::command]` functions that call core
- `pub fn run()` that builds the Tauri app
- Tauri plugin registration

**Example — Svalinn:**
```rust
// core/svalinn_core/src/lib.rs
pub fn scan_directory(directory: &str, include_tests: bool) -> Result<ScanResult, String> { ... }
pub fn list_directory(directory: &str) -> Result<Vec<SvalFileTreeEntry>, String> { ... }

// svalinn/src-tauri/src/lib.rs
use svalinn_core::*;

#[tauri::command]
fn scan_directory(directory: String, include_tests: bool) -> Result<ScanResult, String> {
    svalinn_core::scan_directory(&directory, include_tests)
}
```

**Hlidskjalf special case** — the socket listener needs async + event emission.
Core provides the listener with a channel; the Tauri layer bridges to app events:
```rust
// core/hlidskjalf_core/src/lib.rs
pub async fn start_listener(
    sender: tokio::sync::mpsc::UnboundedSender<HookEvent>
) -> Result<(), String> { ... }

// hlidskjalf/src-tauri/src/lib.rs
#[tauri::command]
async fn start_listener(app: tauri::AppHandle) -> Result<(), String> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    hlidskjalf_core::start_listener(tx).await?;
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = app.emit("hook-event", &event);
        }
    });
    Ok(())
}
```

## Command Naming Convention

**ALL Tauri commands in Yggdrasil use a 4-letter app prefix.**
This is uniform — not "prefix the ones that collide." Every command
traces to its origin by name.

### Prefixes

| App        | Prefix | Applied to all its commands |
|------------|--------|-----------------------------|
| Hlidskjalf | hlid_  | hlid_start_listener         |
| Svalinn    | sval_  | sval_scan_directory, sval_list_directory, sval_open_in_editor, sval_run_saga |
| Kvasir     | kvas_  | kvas_list_directory, kvas_read_file, kvas_open_in_editor, kvas_convert_all_formats, kvas_is_data_file |
| Ratatoskr  | rata_  | rata_load_graph, rata_save_graph, rata_get_graph_stats, rata_generate_sample_graph |

### Frontend Command Maps

Each view component accepts a `commands` prop — a typed record mapping
logical names to actual Tauri command names. Defaults to unprefixed
for standalone use.

```typescript
// SvalinnView.svelte
let {
  commands = {
    scan_directory: "scan_directory",
    list_directory: "list_directory",
    open_in_editor: "open_in_editor",
    run_saga: "run_saga",
  }
}: { commands?: SvalinnCommands } = $props();

// Usage:
await invoke(commands.scan_directory, { directory, includeTests });
```

Yggdrasil passes prefixed versions:
```svelte
<SvalinnView commands={{
  scan_directory: "sval_scan_directory",
  list_directory: "sval_list_directory",
  open_in_editor: "sval_open_in_editor",
  run_saga: "sval_run_saga",
}} />
```

## Workspace Configuration

### Cargo.toml (workspace root)

```toml
[workspace]
resolver = "2"
members = [
    "core/hlidskjalf_core",
    "core/svalinn_core",
    "core/kvasir_core",
    "core/ratatoskr_core",
    "hlidskjalf/src-tauri",
    "svalinn/src-tauri",
    "kvasir/src-tauri",
    "ratatoskr/src-tauri",
    "yggdrasil/src-tauri",
]

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
toml = "0.8"
serde_toon2 = "0.1"
glob = "0.3"
dirs = "5"
tokio = { version = "1", features = ["net", "io-util", "rt-multi-thread", "sync"] }
tauri = { version = "2", features = [] }
tauri-build = { version = "2", features = [] }
tauri-plugin-opener = "2"
tauri-plugin-dialog = "2"
```

### .cargo/config.toml

```toml
[build]
target-dir = "target"
```

### Build & Deploy

```bash
# Build one standalone app
cd introspection/kvasir && npm run tauri build

# Build yggdrasil (compiles all 4 cores + yggdrasil shell)
cd introspection/yggdrasil && npm run tauri build

# All share target/ — incremental builds are fast after first compile

# Deploy (one-time symlink creation)
ln -sf /Users/johnny/.ai/introspection/target/release/bundle/macos/hlidskjalf.app /Applications/Hlidskjalf.app
ln -sf /Users/johnny/.ai/introspection/target/release/bundle/macos/svalinn.app /Applications/Svalinn.app
ln -sf /Users/johnny/.ai/introspection/target/release/bundle/macos/kvasir.app /Applications/Kvasir.app
ln -sf /Users/johnny/.ai/introspection/target/release/bundle/macos/ratatoskr.app /Applications/Ratatoskr.app
ln -sf /Users/johnny/.ai/introspection/target/release/bundle/macos/yggdrasil.app /Applications/Yggdrasil.app
```

After symlinks exist, every `npm run tauri build` automatically updates
what's in `/Applications/`. No copying.

## Data Format Reference

**READ THIS before modifying any viewer or backend.**

### Saga .qa Sidecar Format

Saga writes `.qa` sidecar files co-located with source files.
Path: `src/module.py` -> `.module.py.qa`

```json
{
  "file": "/absolute/path/to/module.py",
  "relative_path": "src/module.py",
  "content_hash": "sha256-of-file-content",
  "generated_at": "2026-03-05T12:34:56Z",
  "issues": [
    {
      "tool": "gleipnir|ruff|basedpyright",
      "code": "function_too_long|E501|reportUnknownMemberType",
      "severity": "blocked|error|warning|info",
      "line": 45,
      "column": 89,
      "message": "Human-readable description",
      "signal": "Why this matters (gleipnir only, absent for ruff/basedpyright)",
      "direction": "What to do instead (gleipnir only)",
      "canary": "LLM behavioral warning (gleipnir only)"
    }
  ]
}
```

**signal, direction, canary** are present on gleipnir issues, absent on
ruff/basedpyright issues. Backend structs use `Option<String>` with
`#[serde(default)]` to handle both.

### Syn Report Format (Hlidskjalf payload)

Syn groups issues and sends them via Unix socket as hook events:

```json
{
  "timestamp": 1646064000.0,
  "category": "quality",
  "decision": "allow|warn|deny",
  "event_name": "syn_check",
  "workspace": "bragi",
  "detail": "7 issues, 0 deny",
  "context_injected": "",
  "payload": {
    "type": "gleipnir_report|syn_report",
    "total": 7,
    "check_types": 3,
    "groups": [
      {
        "code": "function_too_long",
        "severity": "error",
        "count": 5,
        "file_count": 2,
        "signal": "...",
        "direction": "...",
        "canary": "...",
        "locations": [
          { "file": "src/module.py", "lines": [45, 67] }
        ]
      }
    ]
  }
}
```

GleipnirReport.svelte renders the `groups` array with expandable
details. HlidskjalfView discriminates on `payload.type`.

### Voice Level

Voice level (0=silent, 1=deny, 2=all) is **purely frontend state** in
HlidskjalfView. It controls which events trigger a voice prompt.
It does NOT filter events — all events are always displayed. There is
NO backend command for voice level.

## Frontend Architecture

### View Components

Each app's view lives in its own `src/lib/` directory:

| App | View File | Internal Imports |
|-----|-----------|------------------|
| Hlidskjalf | `src/lib/HlidskjalfView.svelte` | `./GleipnirReport.svelte` (relative, not $lib) |
| Svalinn | `src/lib/SvalinnView.svelte` | none |
| Kvasir | `src/lib/KvasirView.svelte` | `./SchemaInspector.svelte`, `./schema-inspect.ts` (relative) |
| Ratatoskr | `src/lib/RatatoskrView.svelte` | none |

**IMPORTANT:** Internal imports within view components MUST use relative
paths (`./`), NOT `$lib/`. The `$lib` alias resolves differently when
imported from Yggdrasil via vite aliases.

### Yggdrasil Vite Aliases

```javascript
// yggdrasil/vite.config.js
resolve: {
  alias: {
    "$hlidskjalf": path.resolve("../hlidskjalf/src/lib"),
    "$svalinn": path.resolve("../svalinn/src/lib"),
    "$kvasir": path.resolve("../kvasir/src/lib"),
    "$ratatoskr": path.resolve("../ratatoskr/src/lib"),
  },
},
```

### Tab Strip

Vertical right-edge strip, 22px wide, full app names spelled vertically.
Views are mounted once and shown/hidden (not destroyed/recreated) so
state persists across tab switches.

## Risk Notes

- D3 (Ratatoskr) may need resize/redraw triggered on tab switch
- Hlidskjalf's socket listener runs in background regardless of active tab
- Internal `$lib` imports in view components break when imported via
  Yggdrasil's vite aliases — use relative `./` imports only
- `src/lib/` files are hidden by global gitignore — always `git add -f`
- Tauri's `generate_context!()` macro needs `tauri.conf.json` at a path
  relative to the crate's `Cargo.toml` — workspace layout must preserve this

## Implementation Tasks

See task list in the current session. If no task list exists, create one
from the phases below:

1. Create workspace Cargo.toml and .cargo/config.toml
2. Extract hlidskjalf_core from hlidskjalf/src-tauri/src/lib.rs
3. Extract svalinn_core from svalinn/src-tauri/src/lib.rs
4. Extract kvasir_core from kvasir/src-tauri/src/lib.rs
5. Extract ratatoskr_core from ratatoskr/src-tauri/src/lib.rs
6. Slim each app's src-tauri/src/lib.rs to Tauri wrappers
7. Rewrite yggdrasil/src-tauri/src/lib.rs to import from cores
8. Add command map props to all 4 view components
9. Update yggdrasil +page.svelte to pass prefixed command maps
10. Verify all 5 apps build
11. Create /Applications symlinks
12. Commit
