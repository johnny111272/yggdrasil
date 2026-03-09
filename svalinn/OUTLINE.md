# Svalinn - Code Quality Viewer

**Svalinn** (Norse: the shield that stands before the sun) is a desktop dashboard for viewing code quality reports from saga/gleipnir QA sidecars.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Svalinn (Tauri App)                       │
├─────────────────────────────────────────────────────────────┤
│  Frontend (Svelte 5)           │  Backend (Rust)            │
│  ─────────────────────         │  ────────────────          │
│  • QA tree navigation          │  • scan_directory()        │
│  • Severity filtering          │  • list_qa_tree()          │
│  • Search                      │  • open_in_editor()        │
│  • Grouped issue display       │  • run_saga()              │
│  • Click-to-open in VS Code    │  • QA sidecar parsing      │
└─────────────────────────────────────────────────────────────┘
```

## Tech Stack

- **Tauri 2.x**: Rust backend + system webview
- **Svelte 5**: Frontend with runes ($state, $derived)
- **@yggdrasil/ui**: Shared component library

## Key Files

```
svalinn/
├── src/
│   └── lib/
│       └── SvalinnView.svelte      # Main UI (git add -f required)
│   └── routes/
│       └── +page.svelte            # Thin wrapper: <SvalinnView />
├── src-tauri/
│   ├── src/
│   │   ├── main.rs                 # svalinn_lib::run()
│   │   └── lib.rs                  # Tauri wrappers
│   ├── Cargo.toml                  # deps: svalinn_core, common_core, tauri
│   └── capabilities/
│       └── default.json            # Tauri permissions
├── package.json
└── OUTLINE.md
```

## Commands

| Standalone | Yggdrasil (prefixed) | Description |
|-----------|---------------------|-------------|
| `scan_directory` | `sval_scan_directory` | Scan QA sidecars, aggregate issues |
| `list_qa_tree` | `sval_list_qa_tree` | List directory with QA sidecar info |
| `open_in_editor` | `sval_open_in_editor` | Open file in VS Code |
| `run_saga` | `sval_run_saga` | Run saga quality scanner |

## QA Sidecar Format

Saga writes `.qa` sidecar files co-located with source files.
Path: `src/module.py` → `.module.py.qa`

Issues come from: gleipnir (custom guardrails), ruff (linting), basedpyright (type checking).

## Related Apps

- **Hlidskjalf**: Agent monitor
- **Kvasir**: Workspace inspector
- **Ratatoskr**: Graph viewer
- **@yggdrasil/ui**: Shared components
