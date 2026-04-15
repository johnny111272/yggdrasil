# Yggdrasil

Unified introspection platform for LLM agent infrastructure. Five Tauri 2.x desktop applications sharing one Cargo workspace with Svelte 5 frontends.

## What This Is

Yggdrasil is a suite of desktop tools for monitoring, inspecting, and debugging LLM agent systems. Four specialised applications are each available standalone or hosted together as tabbed views in the fifth application (Yggdrasil itself).

All components in the ecosystem — hooks, quality scanners, the Bifrost traffic proxy — communicate through a **UDP multicast datagram protocol** (`239.0.0.1:9899`). Emission is fire-and-forget with zero cost if nobody is listening. Multiple dashboards can bind the same port simultaneously (`SO_REUSEADDR` + `SO_REUSEPORT`), so standalone apps and the unified shell receive the same events without conflict. This decoupled architecture means any new component can emit or receive events by speaking the datagram protocol — no registration, no configuration, no central broker.

---

### Hlidskjalf — Agent Activity Monitor

Real-time event feed for LLM agent supervision. Listens for multicast datagrams from hooks, quality scanners, the Bifrost traffic proxy, and other infrastructure components. Displays a filterable feed with typed priority levels and provides voice alerts for critical events via macOS speech synthesis — the speech text travels with the datagram, so the event source decides what gets spoken.

The lockfile system provides physical-world-style safety interlocks. `KILL.lock` stops all agent operations immediately. A missing `KEEP_ALIVE.lock` signals integrity compromise. Both trigger CRITICAL priority alerts with voice notification, checked every five seconds by an independent monitoring task. This is the mechanism that a hardware kill switch on a dedicated proxy appliance would actuate.

### Svalinn — Code Quality Dashboard

Visual frontend to the Gleipnir/Saga/Syn guardrail pipeline. Scans workspace directories for `.qa` sidecar files and displays issues in a navigable tree with click-to-open in the editor.

What makes this more than a lint viewer is that every issue carries three fields from Gleipnir's educational message system: **signal** (what was detected), **direction** (how to address it), and **canary** (how to detect if the fix is superficial rather than genuine). These flow through Saga's truth records all the way to the dashboard — you do not just see "function too long," you see why it matters, what to do about it, and how to tell if someone gamed the metric rather than solving the problem.

### Kvasir — Universal Workspace Inspector

The single inspection tool for an LLM-generated workspace. LLMs and pipelines produce TOML checkpoints, JSON Schemas, JSONL traffic logs, YAML type definitions, `.qa` sidecar reports — all in whatever format the pipeline demands. Without a unified viewer, you are switching between tools, trying to read raw JSONL in a terminal (which is effectively unreadable), mentally parsing structures that would be obvious in a different format, or opening files in editors that do not understand the content.

With Kvasir: one tool, everything. Source code is syntax-highlighted. JSONL is browsable record by record as if each line were a standalone file — select a record, view it as pretty-printed JSON, or switch to TOML if that makes the structure clearer, or export it and open in Zed. Any structured data is viewable in whichever format your brain parses fastest: JSON, YAML, TOML, TOON, RON, XML, or Markdown — all shown simultaneously with **actual LLM token counts** via tiktoken (`cl100k_base`). When the same data costs 400 tokens in TOON and 1,200 in XML, the format choice for context window injection stops being an aesthetic preference and becomes a measurable engineering decision.

Small details matter: Kvasir reads `.bak` files by the extension that precedes `.bak`, so `checkpoint.toml.bak` opens as TOML — not as an unknown binary. It also supports JSON Schema structural analysis and tabular data viewing for CSV, TSV, and Parquet. One click opens anything in the editor.

### Ratatoskr — Graph Viewer

Visualises linked data as interactive force-directed graphs using D3.js. Parses three JSON-LD input formats (embedded graph, `@graph` array, and `@id` cross-file references) and **recursively resolves references across files** — building merged graphs from distributed definitions.

Merge configuration supports join keys (shared nodes that should not be namespaced), colour stylesheets, and namespace prefixes for disambiguating nodes from different source files. This is the tool for visualising the Verdandi type hierarchy, agent definition relationships, and workspace dependency structures as navigable, interactive graphs.

---

## Architecture

All four specialised applications follow the same pattern:

- **Core crate** (`core/*_core/`) — pure Rust with zero Tauri dependency. All business logic. Compiles independently, testable without Tauri, reusable across all five apps.
- **Tauri shell** — thin wrapper of `#[tauri::command]` functions that call into the core crate.
- **Svelte 5 frontend** — UI components consuming the shared `@yggdrasil/ui` design system.

The unified shell (Yggdrasil) imports all four views, maps command names with four-letter prefixes (`hlid_`, `sval_`, `kvas_`, `rata_`), and renders a tab strip. Nothing else.

```
core/           Pure Rust core crates (zero Tauri dependency)
hlidskjalf/     Agent activity monitor
svalinn/        Code quality dashboard
kvasir/         Universal workspace inspector
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
