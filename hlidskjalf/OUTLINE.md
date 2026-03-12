# Hlidskjalf - Agent Monitor

**Hlidskjalf** (Norse: Odin's high seat from which he could see all worlds) is a real-time agent activity monitor. It listens on a Unix socket for datagrams from hooks, scanners, canaries, and other infrastructure components, displays them in a filterable event feed, and provides voice alerts via macOS `say`.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Hlidskjalf (Tauri App)                    │
├─────────────────────────────────────────────────────────────┤
│  Frontend (Svelte 5)           │  Backend (Rust)            │
│  ─────────────────────         │  ────────────────          │
│  • Real-time event feed        │  • Unix socket listener    │
│  • Priority/kind filtering     │  • Datagram parsing        │
│  • Voice alert controls        │  • Lockfile monitoring     │
│  • QualityReport renderer     │  • Log rotation            │
│  • Auto-scroll                 │  • Speech (macOS say)      │
│  • Datagram payload display    │  • start_all() orchestr.   │
└─────────────────────────────────────────────────────────────┘
```

## Tech Stack

- **Tauri 2.x**: Rust backend + system webview
- **Svelte 5**: Frontend with runes ($state, $derived)
- **@yggdrasil/ui**: Shared component library

## Key Files

```
hlidskjalf/
├── src/
│   └── lib/
│       ├── HlidskjalfView.svelte   # Event feed UI (git add -f required)
│       └── QualityReport.svelte   # Quality datagram payload renderer
│   └── routes/
│       └── +page.svelte            # Thin wrapper: <HlidskjalfView />
├── src-tauri/
│   ├── src/
│   │   ├── main.rs                 # hlidskjalf_lib::run()
│   │   └── lib.rs                  # Tauri wrappers: start_monitor, speak
│   ├── Cargo.toml                  # deps: hlidskjalf_core, tauri
│   └── capabilities/
│       └── default.json            # Tauri permissions
├── package.json
└── OUTLINE.md
```

## Core Crate

`core/hlidskjalf_core/` — pure Rust, no Tauri dependency.

Key exports:
- `Datagram`, `DatagramKind`, `Priority` — re-exported from `datagram` (nornir)
- `HookEvent` — legacy format, backward compatible
- `start_all(sender)` — orchestration: rotate log, init lockfiles, start listener, start lockfile monitor
- `speak(text)` — macOS `say` voice synthesis

## Commands

| Standalone | Yggdrasil (prefixed) | Description |
|-----------|---------------------|-------------|
| `start_monitor` | `hlid_start_monitor` | Start socket listener + lockfile monitor |
| `speak` | `hlid_speak` | Speak text via macOS say |

## Related Apps

- **Svalinn**: Code quality viewer
- **Kvasir**: Workspace inspector
- **Ratatoskr**: Graph viewer
- **@yggdrasil/ui**: Shared components
