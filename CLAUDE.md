# Yggdrasil — Context Refresh Protocol

## STOP. Read this before writing any code.

You are working on a multi-app Tauri 2.x desktop platform with shared Rust core crates and Svelte 5 frontends. You have context from a compaction summary or from earlier in this session. **You are almost certainly wrong about the details.**

After compaction, you have app names and a workspace shape but lose the core/shell boundary, the command prefix system, the vite alias conventions, and the ways this project's architecture diverges from standard Tauri/Svelte patterns. You will produce output that looks right, compiles, and silently breaks the unified shell because the summary told you WHAT but not HOW or WHY NOT.

**This has happened before.** Previous sessions in this project have:

- Used `$lib/` imports inside view components — breaks when the component is imported via Vite alias from another app's source tree
- Added Tauri dependencies to core crates — core crates are pure Rust logic with zero Tauri dependency
- Forgot command prefixes in the unified shell — each app's commands need 4-letter prefixes (hlid_, sval_, kvas_, rata_) when hosted in Yggdrasil
- Tried to `git add` files under `src/lib/` without `-f` — the global gitignore at `~/.config/git/ignore` excludes `lib/`

None of them felt uncertain while doing it.

**The test:** Can you name the four command prefixes, explain why view components use `./` imports instead of `$lib/`, and state what the `commands` prop does on each view? If not, read PLAN_UNIFIED_SHELL.md before proceeding.

---
<!-- ═══════════════════════════════════════════════════════════════════════ -->
<!-- EVERYTHING ABOVE THIS LINE IS STANDARDIZED — DO NOT MODIFY            -->
<!-- (except the failure examples, which must be real)                      -->
<!-- EVERYTHING BELOW THIS LINE IS PROJECT-SPECIFIC                        -->
<!-- ═══════════════════════════════════════════════════════════════════════ -->
---

## You Will Get These Things Wrong

### Using $lib/ Imports in View Components
**Detection:** If you find `$lib/` in an import path inside a component that lives in an app's `src/lib/` directory...
**Why it's wrong:** View components are consumed by two different apps: their standalone app (where `$lib` resolves to their own `src/lib/`) and the Yggdrasil unified shell (where `$lib` resolves to Yggdrasil's `src/lib/`). Using `$lib/` means the import works standalone but breaks in the unified shell.
**Recovery:** Use `./` relative imports for sibling components. The view component must be self-contained within its own directory.

### Adding Tauri Dependencies to Core Crates
**Detection:** If you find `tauri` in any `core/*/Cargo.toml` dependency list...
**Why it's wrong:** Core crates (`hlidskjalf_core`, `svalinn_core`, `kvasir_core`, `ratatoskr_core`) are pure Rust logic — parsing, classification, data structures, business rules. They must compile without Tauri. The Tauri crate (e.g., `hlidskjalf/src-tauri/`) is a thin shell that wraps core functions as `#[tauri::command]` handlers. This separation lets core logic be tested, reused, and reasoned about independently.
**Recovery:** Put the logic in the core crate, put the `#[tauri::command]` wrapper in the Tauri crate's `lib.rs`.

### Forgetting Command Prefixes in Unified Shell
**Detection:** If you find a Tauri command registration in Yggdrasil's `lib.rs` without a 4-letter prefix...
**Why it's wrong:** Each standalone app registers commands with bare names (`start_listener`, `speak`). Yggdrasil hosts all four apps in one process — command names must be globally unique. The convention is 4-letter prefixes: `hlid_start_listener`, `sval_scan`, `kvas_list_dir`, `rata_load_graph`. View components receive a `commands` prop that maps bare names to prefixed names.
**Recovery:** Read PLAN_UNIFIED_SHELL.md "Tauri Command Prefixing" section.

### Confusing the Datagram Schema with HookEvent
**Detection:** If you find `HookEvent` fields on a datagram struct, or datagram fields on a hook event handler...
**Why it's wrong:** The datagram format (`schemas/datagram.schema.json`) is the forward-looking protocol. The legacy `HookEvent` format is backward-compatible but deprecated. New code should emit and consume datagrams. Hlidskjalf accepts both via the `hook-event` Tauri event (which deserializes both shapes), but new payload types should use datagram fields.
**Recovery:** Read HLIDSKJALF_DATAGRAM.md and `schemas/datagram.schema.json`.

### Putting Display Logic in Core Crates
**Detection:** If you find HTML, CSS, component references, or frontend formatting in a `*_core` crate...
**Why it's wrong:** Core crates produce typed data. The Svelte frontend decides how to display it. The boundary is: core returns structured data, frontend renders it. If you need to format something for display, that's a Svelte concern.
**Recovery:** Return the raw data from the core crate. Write a Svelte component to render it.

### Editing Shared UI Components Without Checking All Consumers
**Detection:** If you modify a component in `ui/` without checking which apps import it...
**Why it's wrong:** The `ui/` directory is a shared component library consumed by all 5 apps. A change to `SidebarLayout.svelte` affects every app that uses it. CSS token changes propagate everywhere.
**Recovery:** Grep for the component name across all app `src/` directories before modifying.

---

## Recovery Sources

| Document | Path | What it tells you |
|----------|------|-------------------|
| **PLAN_UNIFIED_SHELL** | `PLAN_UNIFIED_SHELL.md` | The unified shell architecture: tab strip, command prefixing, vite aliases, view component contracts, deployment strategy |
| **DATAGRAM_SPECIFICATION** | `DATAGRAM_SPECIFICATION.md` | Datagram format, field definitions, priority levels, type enum, backward compatibility with HookEvent |
| **HLIDSKJALF_DATAGRAM** | `HLIDSKJALF_DATAGRAM.md` | Hlidskjalf-specific datagram handling, socket protocol, event emission |
| **DISPLAY_AND_FILTERING** | `DISPLAY_AND_FILTERING.md` | Exchange diff display design, filter bar, session management, workspace chips |
| **NEURODIVERGENT_MODALITIES** | `NEURODIVERGENT_MODALITIES.md` | Multi-modal alert system: LED, click, pulse, flash, speech. Profiles, geiger counter, ambient awareness |
| **Datagram Schema** | `schemas/datagram.schema.json` | Source of truth for datagram structure |
| **Deploy Script** | `deploy_apps.sh` | Build and deploy all 5 apps to /Applications |

---

## Quick Orientation

**What is this?** A unified introspection platform — 5 Tauri 2.x desktop apps sharing one Cargo workspace, all with Svelte 5 frontends. Norse mythology naming throughout.

**What pattern does it follow?** Core/shell separation: pure Rust logic in `core/*_core/` crates, thin Tauri command wrappers in `*/src-tauri/`, Svelte 5 views in `*/src/lib/`. Yggdrasil unifies all four views via Vite aliases, command prefixing, and a `commands` prop contract.

**Where is the plan?** `PLAN_UNIFIED_SHELL.md` for architecture. `DISPLAY_AND_FILTERING.md` and `NEURODIVERGENT_MODALITIES.md` for the Bifrost integration and alert system design.

**What is the current phase?** Core infrastructure exists for all 5 apps. Hlidskjalf has a working event feed with datagram rendering. Active work: integrating Bifrost exchange diffs into Hlidskjalf display, building the multi-modal alert system, and adding session-aware filtering.

---
<!-- ═══════════════════════════════════════════════════════════════════════ -->
<!-- EVERYTHING ABOVE THIS LINE IS BEHAVIORAL (human-maintained, empirical) -->
<!-- EVERYTHING BELOW THIS LINE IS DYNAMIC ORIENTATION (agent-regenerated)  -->
<!-- ═══════════════════════════════════════════════════════════════════════ -->
---

## Dynamic Orientation

@./CONTEXT_MAP.md
