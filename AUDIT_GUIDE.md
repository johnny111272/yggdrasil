# AUDIT_GUIDE — Yggdrasil

This document exists for audit agents. It lists the critical invariants that LLMs routinely violate in this codebase. These violations compile, look correct, and silently break the architecture. A naive audit will miss them because the code appears to work.

Read this before auditing. Check every invariant. Report violations by invariant ID.

---

## Architectural Invariants

### INV-1: Yggdrasil is a thin wrapper with zero app logic

Yggdrasil imports four views, maps command names, and renders a tab strip. Nothing else. Running Hlidskjalf inside Yggdrasil and running Hlidskjalf standalone must produce identical behavior.

**What to look for:**
- App-specific state, event handlers, or UI logic in `yggdrasil/src/`
- Any file in `yggdrasil/src/` other than `+page.svelte` and `+layout.svelte`
- Conditional behavior in a view component that checks "am I in Yggdrasil?"
- Bug fixes or features added to `yggdrasil/src/routes/+page.svelte` that should be in the app's own `src/lib/`

**The test:** Delete `yggdrasil/` entirely. Do all four apps still work standalone with full functionality? If not, something leaked into the wrapper.

### INV-2: Core crates have zero Tauri dependencies

Core crates (`core/hlidskjalf_core/`, `core/svalinn_core/`, `core/kvasir_core/`, `core/ratatoskr_core/`, `core/common_core/`) are pure Rust. No Tauri, no frontend concepts, no display logic.

**What to look for:**
- `tauri` in any `core/*/Cargo.toml`
- HTML, CSS, or component references in `core/*/src/`
- Frontend formatting (colors, emojis for display, layout strings) in core crate output
- `#[tauri::command]` in any core crate

**The test:** `cargo check -p hlidskjalf_core` must succeed without Tauri installed. Core crates must compile as a standalone Rust library.

### INV-3: View components use relative imports, not $lib/

Every `.svelte` file inside an app's `src/lib/` must use `./` relative imports for sibling components. Never `$lib/`.

**What to look for:**
- `$lib/` in import paths inside `hlidskjalf/src/lib/`, `svalinn/src/lib/`, `kvasir/src/lib/`, `ratatoskr/src/lib/`
- `$lib/` resolves differently in standalone mode vs. inside Yggdrasil — this is a silent break

**The test:** Grep for `from "\$lib/` in all `*/src/lib/` directories. Zero matches is correct.

### INV-4: Command names are prefixed in Yggdrasil, bare in standalone apps

Each standalone app registers commands with bare names. Yggdrasil prefixes them: `hlid_`, `sval_`, `kvas_`, `rata_`. View components receive a `commands` prop mapping bare names to actual names.

**What to look for:**
- Bare (unprefixed) command names in `yggdrasil/src-tauri/src/lib.rs`
- Prefixed command names in standalone app shells (`hlidskjalf/src-tauri/src/lib.rs`, etc.)
- `invoke("start_monitor")` hardcoded in a view component instead of `invoke(commands.start_monitor)`
- Mismatch between the `commands` prop default values and the actual registered command names

**The test:** Every `invoke()` call in a view component must use `commands.X`, never a string literal.

### INV-5: Datagram protocol uses typed enums, not strings

`DatagramKind` and `Priority` are Rust enums in nornir's `datagram` crate, re-exported by `hlidskjalf_core`. Wire format uses lowercase strings via `#[serde(rename_all = "lowercase")]`.

**What to look for:**
- String comparisons for datagram type or priority instead of enum matching
- `HookEvent` usage in new code (legacy, deprecated)
- Tauri event name `"hook-event"` instead of `"datagram"`
- Missing variants in `DatagramKind` or `Priority` enums that exist in `schemas/datagram.schema.json`

**The test:** The Rust enum variants must match the JSON schema enum values exactly (after lowercase transformation).

### INV-6: Shared UI changes affect all consumers

The `ui/` directory (`@yggdrasil/ui`) is imported by all 5 apps. Any change propagates everywhere.

**What to look for:**
- Changes to `ui/components/*.svelte` without checking all importers
- CSS token changes in `ui/css/tokens.css`
- Removed or renamed props on shared components
- New required props that break existing consumers

**The test:** Grep for the component name across all `*/src/lib/` directories before approving any change to `ui/`.

---

## Drift Patterns

These are the specific ways LLMs drift in this codebase. Each has been observed in real sessions.

### DRIFT-1: Fixing app X by editing Yggdrasil

**How it happens:** LLM greps from workspace root, finds Hlidskjalf view imported in `yggdrasil/src/routes/+page.svelte`, adds fix there. The standalone app never gets the fix.

**Where to check:** `yggdrasil/src/routes/+page.svelte` should contain ONLY: tab strip markup, view component imports (via Vite aliases), and command name mapping objects. No event handlers, no state beyond active tab, no conditional rendering logic.

### DRIFT-2: Core crate absorbs shell concerns

**How it happens:** A feature needs "just one Tauri call" so the LLM adds it to the core crate instead of the shell. Tauri dep creeps into `core/*/Cargo.toml`.

**Where to check:** Every `core/*/Cargo.toml` dependency list. Zero Tauri references.

### DRIFT-3: $lib/ imports in view components

**How it happens:** LLM follows standard SvelteKit conventions. `$lib/` is the normal import path. It works in standalone mode. It breaks silently in Yggdrasil because `$lib` resolves to a different directory.

**Where to check:** Every `.svelte` file in `*/src/lib/`. Only `*/src/routes/` files may use `$lib/`.

### DRIFT-4: Hardcoded invoke strings

**How it happens:** LLM writes `invoke("start_monitor")` instead of `invoke(commands.start_monitor)`. Works in standalone. Fails in Yggdrasil where the command is `hlid_start_monitor`.

**Where to check:** Every `invoke()` call in `*/src/lib/*.svelte`.

### DRIFT-5: String-based datagram handling

**How it happens:** LLM pattern-matches from old code or training data. Writes `if (event.type === "alert")` instead of matching on `DatagramKind::Alert`. Or uses the old `"hook-event"` Tauri event name.

**Where to check:** All datagram handling code in both Rust and Svelte.

### DRIFT-6: Display logic in core crates

**How it happens:** LLM wants to return "formatted" data — adds HTML snippets, color codes, emoji for display, or layout strings in core crate functions. Core crates return structured data; frontends decide presentation.

**Where to check:** Return types of all core crate public functions. Should be typed structs/enums, never strings containing markup.

---

## How to Audit This Codebase

Do not start with grep. Do not write a checklist and execute it mechanically. The violations that matter most are the ones nobody thought to check for.

Read the invariants above. Understand *why* each one exists — what breaks when it's violated, and why the violation looks correct on the surface. Then read the code with fresh eyes and ask yourself these questions:

**About boundaries:**
- Where does data cross from one layer to another? Is the boundary clean or is something leaking through?
- If I deleted the Yggdrasil app entirely, would anything break in the four standalone apps? Why or why not?
- Are the core crates truly independent of their runtime environment, or do they make assumptions about who's calling them?

**About intent:**
- Does this code belong where it lives? Not "does it compile here" — does it *belong* here? Would a developer looking for this behavior find it in this location?
- Is this function doing one thing, or is it doing its thing plus something that belongs to its caller or its consumer?
- Are there decisions being made in the wrong layer? Formatting in core crates, business logic in views, app behavior in the wrapper?

**About drift:**
- What would a well-meaning but context-free LLM get wrong if it tried to modify this code? Where are the traps?
- Is there code that works today but encodes a false assumption about the architecture? What happens when that assumption breaks?
- Are there patterns that started correct but have been copied into contexts where they don't apply?

**About what you're not seeing:**
- What error paths don't exist? What happens when the happy path fails?
- What's implicit that should be explicit? What relies on convention rather than enforcement?
- Where would you add a feature, and would you add it in the right place without reading this guide?

Report findings with the invariant or drift IDs where they apply. But the most valuable findings will be the ones that don't map to any existing ID — those are the blind spots.
