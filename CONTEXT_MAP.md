# CONTEXT_MAP — Yggdrasil

**Last updated:** 2026-03-08

## 1. Purpose Statement

Yggdrasil is a unified introspection platform — 5 Tauri 2.x desktop apps for macOS sharing one Cargo workspace. Four specialized apps (Hlidskjalf, Svalinn, Kvasir, Ratatoskr) each have a core Rust crate for pure logic and a Tauri shell for command wrappers, with Svelte 5 frontends. The fifth app (Yggdrasil) hosts all four as switchable tabbed views via Vite aliases and command prefixing. This CONTEXT_MAP helps agents orient to where knowledge lives and where to refresh context mid-session.

---

## 2. Primary References

| Resource | Path | What It Contains | When to Read |
|----------|------|------------------|--------------|
| CLAUDE.md | `CLAUDE.md` | Context refresh protocol, 6 anti-patterns, recovery sources, architecture summary | Every session start. Mandatory before writing any code. |
| PLAN_UNIFIED_SHELL | `PLAN_UNIFIED_SHELL.md` | Full unified shell architecture: tab strip, Vite aliases, command prefixing, view component contract, deployment | When you need to understand the Yggdrasil host or how apps compose |
| DISPLAY_AND_FILTERING | `DISPLAY_AND_FILTERING.md` | Exchange diff display design: datagram payload, feed rendering, filter bar, session management | When working on Hlidskjalf's Bifrost integration |
| NEURODIVERGENT_MODALITIES | `NEURODIVERGENT_MODALITIES.md` | Multi-modal alert system: 5 modalities, profiles, geiger counter, ambient awareness | When working on the alert/notification system |

---

## 3. Documentation References

### Architecture & Design

| Document | Path | Relevance | Freshness |
|----------|------|-----------|-----------|
| PLAN_UNIFIED_SHELL | `PLAN_UNIFIED_SHELL.md` | Authoritative architecture for the unified shell, command prefixing, Vite aliases, deployment | Current |
| DATAGRAM_SPECIFICATION | `DATAGRAM_SPECIFICATION.md` | Datagram format definition, field semantics, priority levels | Current |
| HLIDSKJALF_DATAGRAM | `HLIDSKJALF_DATAGRAM.md` | Socket protocol, event emission, Hlidskjalf-specific datagram handling | Current |
| DISPLAY_AND_FILTERING | `DISPLAY_AND_FILTERING.md` | Exchange diff display, session chips, filter bar design | Current — design doc, not yet implemented |
| NEURODIVERGENT_MODALITIES | `NEURODIVERGENT_MODALITIES.md` | Alert modalities, profiles, ambient awareness system | Current — design doc, not yet implemented |

### Per-App Outlines

| Document | Path | Relevance | Freshness |
|----------|------|-----------|-----------|
| Svalinn OUTLINE | `svalinn/OUTLINE.md` | Feature outline for code quality viewer | Current |
| Svalinn README | `svalinn/README.md` | Svalinn overview and setup | Current |

---

## 4. Schema References

| Schema | Path | What It Defines |
|--------|------|----------------|
| Datagram | `schemas/datagram.schema.json` | Source of truth for the datagram protocol — fields, types, enums for priority and type |

**Note:** The datagram schema needs updating to include `"exchange"` in the type enum once the Bifrost integration work begins.

---

## 5. Source Code Map

### Workspace Root

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace manifest — 9 crates (4 core + 5 Tauri), shared dependency versions |
| `deploy_apps.sh` | Build all 5 apps, move .app bundles to /Applications, clean build artifacts |
| `.cargo/config.toml` | Build target configuration (aarch64-apple-darwin) |

### Core Crates (`core/`) — Pure Rust, No Tauri

| Crate | Path | Purpose | Key Types/Functions |
|-------|------|---------|---------------------|
| `hlidskjalf_core` | `core/hlidskjalf_core/` | Socket listener, datagram parsing, lockfile monitor, log rotation, voice | `Datagram`, `HookEvent`, `listen_and_emit()`, `speak()`, lockfile scanning |
| `svalinn_core` | `core/svalinn_core/` | QA sidecar scanning, saga report types | `scan_directory()`, `QaReport` |
| `kvasir_core` | `core/kvasir_core/` | File browsing, format conversion (JSON/YAML/TOML/TOON) | `list_directory()`, `read_file()`, `convert_format()` |
| `ratatoskr_core` | `core/ratatoskr_core/` | Graph loading, JSON-LD parsing, merge config | `load_graph()`, `parse_jsonld()` |

### Tauri Shells — Thin Command Wrappers

| App | Tauri Crate Path | Commands Registered | Standalone Port |
|-----|-----------------|--------------------|----|
| Hlidskjalf | `hlidskjalf/src-tauri/` | `start_listener`, `speak` | Standalone app |
| Svalinn | `svalinn/src-tauri/` | `scan`, `read_report` | Standalone app |
| Kvasir | `kvasir/src-tauri/` | `list_dir`, `read_file`, `convert` | Standalone app |
| Ratatoskr | `ratatoskr/src-tauri/` | `load_graph` | Standalone app |
| Yggdrasil | `yggdrasil/src-tauri/` | All above with prefixes (hlid_, sval_, kvas_, rata_) | Unified host |

### Svelte Frontends — View Components

| App | View Component | Path | Supporting Components |
|-----|---------------|------|-----------------------|
| Hlidskjalf | `HlidskjalfView.svelte` | `hlidskjalf/src/lib/` | `GleipnirReport.svelte` |
| Svalinn | `SvalinnView.svelte` | `svalinn/src/lib/` | — |
| Kvasir | `KvasirView.svelte` | `kvasir/src/lib/` | `SchemaInspector.svelte` |
| Ratatoskr | `RatatoskrView.svelte` | `ratatoskr/src/lib/` | — |

**View component contract:** Each accepts a `commands` prop mapping bare command names to (potentially prefixed) names. Internal imports use `./` (not `$lib/`). This allows Yggdrasil to import them via Vite aliases and supply prefixed command names.

### Shared UI Library (`ui/`)

12 components: `SidebarLayout`, `Button`, `Badge`, `Input`, `Select`, `Panel`, `StatCard`, `TreeNode`, `Collapsible`, `ListItem`, `SearchInput`, `FilterBanner`

CSS design tokens in `ui/tokens.css`. All apps import this for consistent theming.

### Yggdrasil Unified Shell

| File | Purpose |
|------|---------|
| `yggdrasil/src/routes/+page.svelte` | Tab strip, view switching, imports all 4 views |
| `yggdrasil/vite.config.js` | Vite aliases: `$hlidskjalf`, `$svalinn`, `$kvasir`, `$ratatoskr` |
| `yggdrasil/src-tauri/src/lib.rs` | Registers all commands with 4-letter prefixes |

---

## 6. Upstream Dependencies

| Dependency | Source | What It Provides |
|------------|--------|-----------------|
| Nornir binaries | `~/.ai/smidja/nornir/` | `send_alert`, `send_datagram` — CLI tools for emitting datagrams to the Hlidskjalf Unix socket |
| Bifrost | `~/.ai/smidja/bifrost/` | Exchange diff datagrams (planned), compaction alerts (current via `send_alert`) |
| Datagram protocol | `schemas/datagram.schema.json` | Shared contract between all datagram producers and the Hlidskjalf consumer |

---

## 7. Downstream Consumers

| Consumer | What It Reads |
|----------|--------------|
| macOS desktop users | Built .app bundles in /Applications |
| Other apps in the workspace | Shared `ui/` components and CSS tokens |

---

## 8. Context Refresh Guide

| If you need to understand... | Read |
|------------------------------|------|
| **The unified shell architecture** | `PLAN_UNIFIED_SHELL.md` |
| **How command prefixing works** | `PLAN_UNIFIED_SHELL.md` "Tauri Command Prefixing" |
| **How Vite aliases import cross-app views** | `PLAN_UNIFIED_SHELL.md` "Frontend Integration" + `yggdrasil/vite.config.js` |
| **The view component contract (commands prop)** | `PLAN_UNIFIED_SHELL.md` + `hlidskjalf/src/lib/HlidskjalfView.svelte` as reference implementation |
| **The datagram format** | `schemas/datagram.schema.json` + `DATAGRAM_SPECIFICATION.md` |
| **How Hlidskjalf listens for events** | `core/hlidskjalf_core/src/lib.rs` `listen_and_emit()` |
| **The exchange diff display design** | `DISPLAY_AND_FILTERING.md` |
| **The multi-modal alert system** | `NEURODIVERGENT_MODALITIES.md` |
| **How to build and deploy all apps** | `deploy_apps.sh` |
| **What shared UI components exist** | `ui/components/*.svelte` |
| **What anti-patterns to avoid** | `CLAUDE.md` "You Will Get These Things Wrong" |
| **How GleipnirReport renders payloads** | `hlidskjalf/src/lib/GleipnirReport.svelte` — reference for building new payload renderers |

---

## 9. Known Issues / Active Work

### Current Build State

All 5 apps have scaffolded Rust backends and Svelte frontends. Hlidskjalf is the most complete — working event feed with datagram rendering, GleipnirReport payload renderer, priority/type filtering, speech alerts, auto-scroll.

### Uncommitted Work

- `hlidskjalf_core`: lockfile monitor, log rotation, voice alerts (uncommitted in `core/hlidskjalf_core/src/lib.rs`)
- Workspace `Cargo.toml`: added `time` feature to tokio
- `deploy_apps.sh`: enhanced with cleanup logic
- `schemas/`: entire directory is untracked (contains `datagram.schema.json`)

### Active Design — Not Yet Implemented

- **Bifrost integration**: Exchange diff datagrams flowing to Hlidskjalf. Design in `DISPLAY_AND_FILTERING.md`.
- **Session-aware filtering**: Workspace chips, color families, per-session toggles. Design in `DISPLAY_AND_FILTERING.md`.
- **Multi-modal alerts**: LED, click, pulse bar, flash, speech. Profiles (hyperfocus, sensitive, monitoring, active, silent). Design in `NEURODIVERGENT_MODALITIES.md`.
- **Geiger counter**: Ambient activity-rate audio + visual chip pulsing. Design in `NEURODIVERGENT_MODALITIES.md`.
- **ExchangeDiffReport.svelte**: New payload renderer for exchange diffs (parallel to GleipnirReport.svelte). Described in `DISPLAY_AND_FILTERING.md`.

### Schema Update Needed

`schemas/datagram.schema.json` type enum needs `"exchange"` added before exchange diff datagrams can flow.

### Gitignore Gotcha

Global gitignore at `~/.config/git/ignore` line 43 excludes `lib/`. Use `git add -f` when staging any files under `src/lib/` directories.
