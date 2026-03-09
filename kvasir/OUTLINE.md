# Kvasir - Workspace Inspector

**Kvasir** (Norse: the wisest being, whose knowledge was distilled into the mead of poetry) is a desktop workspace inspector for exploring codebases with syntax highlighting, format conversion, and schema inspection.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Kvasir (Tauri App)                       │
├─────────────────────────────────────────────────────────────┤
│  Frontend (Svelte 5)           │  Backend (Rust)            │
│  ─────────────────────         │  ────────────────          │
│  • Directory tree view         │  • list_directory()        │
│  • Code viewer (highlight.js)  │  • read_file()             │
│  • Format converter            │  • convert_to_all_formats()│
│  • Schema inspector            │  • detect_data_format()    │
│  • Dotfile toggle              │  • open_in_editor()        │
└─────────────────────────────────────────────────────────────┘
```

## Tech Stack

- **Tauri 2.x**: Rust backend + system webview
- **Svelte 5**: Frontend with runes ($state, $derived)
- **@yggdrasil/ui**: Shared component library
- **highlight.js**: Syntax highlighting

## Key Files

```
kvasir/
├── src/
│   └── lib/
│       ├── KvasirView.svelte       # Main UI (git add -f required)
│       ├── SchemaInspector.svelte   # JSON Schema analysis renderer
│       └── schema-inspect.ts       # Pure schema analysis functions
│   └── routes/
│       └── +page.svelte            # Thin wrapper: <KvasirView />
├── src-tauri/
│   ├── src/
│   │   ├── main.rs                 # kvasir_lib::run()
│   │   └── lib.rs                  # Tauri wrappers
│   ├── Cargo.toml                  # deps: kvasir_core, common_core, tauri
│   └── capabilities/
│       └── default.json            # Tauri permissions
├── package.json
└── OUTLINE.md
```

## Commands

| Standalone | Yggdrasil (prefixed) | Description |
|-----------|---------------------|-------------|
| `list_directory` | `kvas_list_directory` | List directory contents |
| `read_file` | `kvas_read_file` | Read file with language detection |
| `open_in_editor` | `kvas_open_in_editor` | Open file in VS Code |
| `convert_to_all_formats` | `kvas_convert_to_all_formats` | Convert between JSON/YAML/TOML/TOON |
| `detect_data_format` | `kvas_detect_data_format` | Detect data format from file extension |

## Related Apps

- **Hlidskjalf**: Agent monitor
- **Svalinn**: Code quality viewer
- **Ratatoskr**: Graph viewer
- **@yggdrasil/ui**: Shared components
