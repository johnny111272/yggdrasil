# Yggdrasil

Unified introspection platform for LLM agent infrastructure. Five Tauri 2.x desktop applications sharing one Cargo workspace with Svelte 5 frontends.

## What This Is

Yggdrasil is a suite of desktop tools for monitoring, inspecting, and debugging LLM agent systems. Four specialised applications are each available standalone or hosted together as tabbed views in the fifth application (Yggdrasil itself):

### Hlidskjalf — Agent Activity Monitor

Real-time event feed for LLM agent supervision. Listens for datagrams from hooks, quality scanners, the Bifrost traffic proxy, and other infrastructure components. Displays a filterable feed with typed priority levels, provides voice alerts for critical events, and maintains a lockfile-based kill mechanism — if the kill lock is set, no agent action proceeds.

### Svalinn — Code Quality Dashboard

Visual frontend to the Gleipnir/Saga/Syn guardrail system. Scans workspace directories for `.qa` sidecar files, aggregates issues by tool and severity, and displays them in a navigable tree with click-to-open in the editor. A read-only consumer of the same truth data that Syn reads.

### Kvasir — Workspace Inspector

Codebase explorer with syntax highlighting and multi-format conversion (JSON, YAML, TOML, TOON, RON, CSV, Parquet). Provides a directory tree browser, code viewer, and format converter showing the same data in all supported formats simultaneously. Includes JSON Schema inspection and analysis.

### Ratatoskr — Graph Viewer

Visualises JSON-LD linked data with D3.js force-directed layouts. Loads JSON-LD files (embedded graph, `@graph` array, or `@id` cross-file references), resolves references, and supports graph merging with configurable join keys and colour stylesheets.

## Architecture

All four specialised applications follow the same pattern:

- **Core crate** (`core/*_core/`) — pure Rust with zero Tauri dependency. All business logic. Compiles independently, testable without Tauri, reusable across all five apps.
- **Tauri shell** — thin wrapper of `#[tauri::command]` functions that call into the core crate.
- **Svelte 5 frontend** — UI components consuming shared `@yggdrasil/ui` design system.

The unified shell (Yggdrasil) imports all four views, maps command names with four-letter prefixes (`hlid_`, `sval_`, `kvas_`, `rata_`), and renders a tab strip. Nothing else.

```
core/           Pure Rust core crates (zero Tauri dependency)
hlidskjalf/     Agent activity monitor
svalinn/        Code quality dashboard
kvasir/         Workspace inspector
ratatoskr/      Graph viewer
yggdrasil/      Unified shell hosting all four as tabs
ui/             Shared Svelte component library (@yggdrasil/ui)
schemas/        Datagram and protocol schemas
```

## Building

```bash
cd <app> && npm run tauri build     # Build individual app
./deploy_apps.sh                     # Build, deploy, and restart all apps
```

## Licence

Copyright (c) 2025–2026 John Oker-Blom

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

See [LICENSE](LICENSE) for the full licence text.
