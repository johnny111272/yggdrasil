# Ratatoskr - Graph Viewer

**Ratatoskr** (Norse: the squirrel who runs up and down Yggdrasil carrying messages) is a desktop graph viewer for visualizing JSON-LD linked data with D3.js force-directed layouts.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Ratatoskr (Tauri App)                     │
├─────────────────────────────────────────────────────────────┤
│  Frontend (Svelte 5 + D3)      │  Backend (Rust)            │
│  ─────────────────────         │  ────────────────          │
│  • Force-directed graph        │  • load_graph()            │
│  • Zoom/pan/drag               │  • save_graph()            │
│  • Node selection panel        │  • get_graph_stats()       │
│  • Stats panel                 │  • JSON-LD parsing         │
│  • Node coloring (stylesheet)  │  • Reference resolution    │
│                                │  • Graph merging           │
└─────────────────────────────────────────────────────────────┘
```

## Tech Stack

- **Tauri 2.x**: Rust backend + system webview
- **Svelte 5**: Frontend with runes ($state, $derived)
- **D3.js**: Force-directed graph visualization
- **@yggdrasil/ui**: Shared component library

## Key Files

```
ratatoskr/
├── src/
│   └── lib/
│       └── RatatoskrView.svelte    # D3 graph UI (git add -f required)
│   └── routes/
│       └── +page.svelte            # Thin wrapper: <RatatoskrView />
├── src-tauri/
│   ├── src/
│   │   ├── main.rs                 # ratatoskr_lib::run()
│   │   └── lib.rs                  # Tauri wrappers
│   ├── Cargo.toml                  # deps: ratatoskr_core, tauri
│   └── capabilities/
│       └── default.json            # Tauri permissions
├── package.json
└── OUTLINE.md
```

## Commands

| Standalone | Yggdrasil (prefixed) | Description |
|-----------|---------------------|-------------|
| `load_graph` | `rata_load_graph` | Load JSON-LD graph from file |
| `save_graph` | `rata_save_graph` | Save graph to file |
| `get_graph_stats` | `rata_get_graph_stats` | Get node/edge counts |
| `generate_sample_graph` | `rata_generate_sample_graph` | Generate demo graph |

## JSON-LD Support

Ratatoskr handles multiple JSON-LD formats:

| Format | Description |
|--------|-------------|
| Embedded graph | `nodes` + `edges` arrays in document |
| @graph array | Standard JSON-LD @graph structure |
| References | Resolves `@id` references to other files |

### Merge Configuration

When loading files with references, use these fields:

```json
{
  "join_on": "APPLICATION",
  "prefix": "West: ",
  "color": "#ff4444",
  "stylesheet": {
    "APPLICATION": "#4169e1",
    "COMMAND GATE": "#ffd700"
  }
}
```

## Related Apps

- **Hlidskjalf**: Agent monitor
- **Svalinn**: Code quality viewer
- **Kvasir**: Workspace inspector
- **@yggdrasil/ui**: Shared components
