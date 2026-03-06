# Hlidskjalf - File Viewer

**Hlidskjalf** (Norse: Odin's high seat from which he could see all worlds) is a desktop file viewer for exploring codebases with syntax highlighting and data visualization.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Hlidskjalf (Tauri App)                    │
├─────────────────────────────────────────────────────────────┤
│  Frontend (Svelte 5)           │  Backend (Rust)            │
│  ─────────────────────         │  ────────────────          │
│  • Directory tree view         │  • list_directory()        │
│  • Code viewer (highlight.js)  │  • read_file()             │
│  • JSON/YAML viewer            │  • detect_language()       │
│  • Markdown preview            │  • get_file_stats()        │
│  • Token counting              │                            │
│  • Search/filter               │                            │
└─────────────────────────────────────────────────────────────┘
```

## Tech Stack

- **Tauri 2.x**: Rust backend + system webview
- **Svelte 5**: Frontend with runes ($state, $derived)
- **@introspection/ui**: Shared component library
- **highlight.js**: Syntax highlighting

## Key Files

```
~/.ai/introspection/hlidskjalf/
├── src/
│   └── routes/
│       └── +page.svelte      # Main UI (tree + viewer)
├── src-tauri/
│   ├── src/
│   │   └── lib.rs            # Rust backend (file ops)
│   ├── Cargo.toml            # Rust dependencies
│   └── capabilities/
│       └── default.json      # Tauri permissions
├── package.json              # Node dependencies
└── OVERVIEW.md               # This file
```

## Running

```bash
cd ~/.ai/introspection/hlidskjalf
npm run tauri dev
```

## Features

| Feature | Description |
|---------|-------------|
| Tree navigation | Expandable directory tree with file icons |
| Code viewing | Syntax-highlighted code with line numbers |
| Data viewing | Pretty-printed JSON/YAML/TOML |
| Markdown | Rendered markdown preview |
| Token count | Approximate token count for LLM context |

## Related Apps

- **Svalinn**: Code quality dashboard
- **Ratatoskr**: Graph viewer
- **@introspection/ui**: Shared components
