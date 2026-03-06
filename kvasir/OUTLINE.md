# Kvasir - Workspace Inspector

**Kvasir** (Norse: the wisest being, whose knowledge was distilled into the mead of poetry) is a desktop workspace inspector for exploring codebases with syntax highlighting and data visualization.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Kvasir (Tauri App)                       │
├─────────────────────────────────────────────────────────────┤
│  Frontend (Svelte 5)           │  Backend (Rust)            │
│  ─────────────────────         │  ────────────────          │
│  • Directory tree view         │  • list_directory()        │
│  • Code viewer (highlight.js)  │  • read_file()             │
│  • JSON/YAML viewer            │  • detect_language()       │
│  • Markdown preview            │  • get_file_stats()        │
│  • Token counting              │                            │
│  • Dotfile toggle              │                            │
└─────────────────────────────────────────────────────────────┘
```

## Tech Stack

- **Tauri 2.x**: Rust backend + system webview
- **Svelte 5**: Frontend with runes ($state, $derived)
- **@introspection/ui**: Shared component library
- **highlight.js**: Syntax highlighting

## Key Files

```
~/.ai/introspection/kvasir/
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
└── OUTLINE.md                # This file
```

## Running

```bash
cd ~/.ai/introspection/kvasir
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
| Dotfile toggle | Show/hide hidden files and directories |

## Related Apps

- **Svalinn**: Code quality dashboard
- **Ratatoskr**: Graph viewer
- **@introspection/ui**: Shared components
