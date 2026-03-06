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
- **@introspection/ui**: Shared component library

## Key Files

```
~/.ai/introspection/ratatoskr/
├── src/
│   └── routes/
│       └── +page.svelte      # D3 graph UI
├── src-tauri/
│   ├── src/
│   │   └── lib.rs            # Rust backend (JSON-LD, graph ops)
│   ├── Cargo.toml            # Rust dependencies
│   └── capabilities/
│       └── default.json      # Tauri permissions
├── package.json              # Node dependencies
└── OVERVIEW.md               # This file
```

## Running

```bash
cd ~/.ai/introspection/ratatoskr
npm run tauri dev
```

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
  "join_on": "APPLICATION",     // Shared node (not prefixed)
  "prefix": "West: ",           // Prefix for other nodes
  "color": "#ff4444",           // Default color for unmatched nodes
  "stylesheet": {               // Label → color mapping
    "APPLICATION": "#4169e1",
    "COMMAND GATE": "#ffd700"
  }
}
```

**Color priority:**
1. Stylesheet match (exact label)
2. Gate default (`color` field)
3. Type-based fallback

## Features

| Feature | Description |
|---------|-------------|
| Force layout | D3 force-directed positioning |
| Zoom/pan | Mouse wheel + drag on background |
| Node drag | Drag nodes to reposition |
| Selection | Click nodes to see details |
| Stylesheet | JSON-LD defined node colors |
| Reference resolution | Load linked documents |
| Graph merging | Combine multiple graphs with shared nodes |

## Related Apps

- **Svalinn**: Code quality dashboard
- **Hlidskjalf**: File viewer
- **@introspection/ui**: Shared components
