# Svalinn - Code Quality Viewer

**Svalinn** (Norse: Odin's high seat from which he could see all worlds) is a desktop dashboard for viewing code quality reports from multiple static analysis tools.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Svalinn (Tauri App)                   │
├─────────────────────────────────────────────────────────────┤
│  Frontend (Svelte 5)           │  Backend (Rust)            │
│  ─────────────────────         │  ────────────────          │
│  • Directory picker            │  • Tool runners:           │
│  • Tool toggles                │    - ruff                  │
│  • View modes (file/code/tool) │    - basedpyright          │
│  • Severity filters            │    - radon                 │
│  • Search                      │    - gleipnir              │
│  • Grouped issue display       │  • File counting           │
│  • Click-to-open in VS Code    │  • Result aggregation      │
└─────────────────────────────────────────────────────────────┘
```

## Tech Stack

- **Tauri 2.x**: Rust backend + system webview
- **Svelte 5**: Frontend with runes ($state, $derived)
- **TypeScript**: Type-safe frontend code

## Key Files

```
~/.ai/svalinn/
├── src/
│   └── routes/
│       └── +page.svelte      # Main UI (all frontend code)
├── src-tauri/
│   ├── src/
│   │   └── lib.rs            # Rust backend (tool runners, commands)
│   ├── Cargo.toml            # Rust dependencies
│   └── capabilities/
│       └── default.json      # Tauri permissions
├── package.json              # Node dependencies
└── OVERVIEW.md               # This file
```

## Running

```bash
cd ~/.ai/svalinn
npm run tauri dev
```

## Tool Integration

Each tool is called via subprocess and parsed:

| Tool | Command | Output Format |
|------|---------|---------------|
| ruff | `ruff check --output-format=json <dir>` | JSON array |
| basedpyright | `basedpyright --outputjson <dir>` | JSON with generalDiagnostics |
| radon | `radon cc --json <dir>` | JSON file map |
| gleipnir | `python file_guardrails.py <dir> --json` | JSON array |

## Future Plans

- **Saga integration**: Read from .phoenix/ cache instead of running tools directly
- **D3 visualizations**: Dependency graphs, complexity heatmaps
- **Real-time updates**: Watch mode with file system events
- **MITM viewer**: Repurpose for Claude Code traffic inspection
