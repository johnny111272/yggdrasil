# Invariant & Drift Audit — Yggdrasil

**Date:** 2026-03-11
**Auditor:** Claude Opus 4.6 (automated)
**Scope:** INV-1 through INV-6, DRIFT-1 through DRIFT-6, plus emergent findings

---

## Summary

The codebase is in strong architectural health. The core invariants hold: core crates have zero Tauri dependencies, view components use relative imports, all invoke calls go through the `commands` prop, command prefixes are correct in Yggdrasil, and the datagram protocol uses typed enums in Rust. No catastrophic violations.

There are **2 warnings** and **4 notes**. The warnings involve a nonexistent CSS token referenced across multiple components and a UI control that silently does nothing. Neither breaks compilation or crashes the app, but both create user-facing incorrectness.

| Severity | Count |
|----------|-------|
| Violation | 0 |
| Warning | 2 |
| Note | 4 |

---

## Findings

### W-1: `--severity-info` CSS token used but not defined (INV-6)

**Severity:** warning
**Files and lines:**
- `hlidskjalf/src/lib/HlidskjalfView.svelte` lines 50, 344, 439, 440
- `svalinn/src/lib/SvalinnView.svelte` lines 254, 692, 704
- `kvasir/src/lib/FormatControls.svelte` lines 97, 119, 132
- `hlidskjalf/src/lib/QualityReport.svelte` lines 57, 194, 195

**Description:** The CSS variable `--severity-info` is referenced in 4 components across 3 apps (11 total references), but it does not exist in `ui/css/tokens.css`. The token was renamed to `--severity-success` (green, `#6bcb77`). The affected declarations silently resolve to the initial value (typically transparent or inherited), meaning:

- In HlidskjalfView: "normal" priority events have no visible color instead of green. The `.status.connected` badge background is transparent. The voice button border/text is invisible.
- In SvalinnView: the "info" severity default color and `.issue-file` color are missing.
- In FormatControls: the source format badge border and token stat highlight are invisible.
- In QualityReport: the `.group.severity-info` border is missing.

**Impact:** Visual degradation across multiple views. Not a crash, but a noticeable loss of intended color coding in the UI.

**Fix:** Either add `--severity-info` back as an alias in `tokens.css`, or replace all 11 references with `--severity-success`.

---

### W-2: Kvasir "show dotfiles" toggle does nothing (NEW)

**Severity:** warning
**Files and lines:**
- `kvasir/src/lib/KvasirView.svelte` lines 64, 97, 139, 359
- `kvasir/src-tauri/src/lib.rs` line 11
- `yggdrasil/src-tauri/src/lib.rs` line 57
- `core/kvasir_core/src/lib.rs` lines 73, 88-90

**Description:** KvasirView.svelte has a `showHidden` state variable (line 64) bound to a checkbox toggle (line 359). It passes `showHidden` to `invoke(commands.list_directory, { directory, showHidden })` on lines 97 and 139. However:

1. The Tauri command `list_directory` (and `kvas_list_directory`) accepts only `directory: String` -- no `show_hidden` parameter.
2. The core function `kvasir_core::list_directory(directory: &str)` has a hardcoded `if name.starts_with('.') { continue; }` on line 88.
3. Tauri's invoke system silently ignores extra JSON fields, so the `showHidden` parameter is discarded without error.

The user sees a working checkbox, clicks it, reloads the tree, and gets the same result. The toggle is dead UI.

**Impact:** User-facing feature that silently does nothing. An LLM asked to "fix the dotfile toggle" would likely add the parameter to the Tauri wrapper and core function -- which is the correct fix but needs to be done in the right layers.

**Fix:** Add `show_hidden: bool` parameter to `kvasir_core::list_directory()`, gate the `.starts_with('.')` skip on it, propagate through the Tauri wrappers in both `kvasir/src-tauri/src/lib.rs` and `yggdrasil/src-tauri/src/lib.rs`.

---

### N-1: INV-1 / DRIFT-1 — Yggdrasil thin wrapper: PASS

**Severity:** note

**Description:** `yggdrasil/src/routes/+page.svelte` contains only:
- Tab strip markup and tab switching state (`activeTab`, `mounted`)
- View component imports via Vite aliases (`$hlidskjalf`, `$svalinn`, `$kvasir`, `$ratatoskr`)
- Command name mapping objects with correct 4-letter prefixes
- File-open handling (`get_pending_file`, `open-file` listener) that routes to Kvasir

The file-open handling (lines 34-47) is shell-level plumbing -- deciding which tab to activate when the OS opens a file. This is Yggdrasil's job, not any app's. No app-specific logic has leaked.

The only other files in `yggdrasil/src/` are `app.html`, `+layout.ts`, `+layout.svelte` -- all standard SvelteKit scaffolding.

---

### N-2: INV-2 / DRIFT-2 — Core crates Tauri-free: PASS

**Severity:** note

**Description:** All five `core/*/Cargo.toml` files audited. Dependencies:
- `common_core`: zero deps
- `hlidskjalf_core`: `datagram`, `serde`, `serde_json`, `tokio` -- no Tauri
- `svalinn_core`: `common_core`, `serde`, `serde_json`, `glob`, `dirs` -- no Tauri
- `kvasir_core`: `common_core`, `format_core`, `serde`, `serde_json`, `ron`, `csv`, `parquet`, `arrow` -- no Tauri
- `ratatoskr_core`: `serde`, `serde_json` -- no Tauri

No `#[tauri::command]` annotations in any core crate source. No HTML/CSS/display logic in core crate outputs. The core/shell boundary is clean.

---

### N-3: INV-3 / DRIFT-3 — Relative imports in view components: PASS

**Severity:** note

**Description:** Zero `$lib/` imports found in any `*/src/lib/*.svelte` or `*/src/lib/*.ts` file. All sibling imports use `./` relative paths:
- `HlidskjalfView.svelte` imports `./QualityReport.svelte`
- `KvasirView.svelte` imports `./MarkdownPreview.svelte`, `./SchemaInspector.svelte`, `./JsonlViewer.svelte`, `./TableViewer.svelte`, `./FormatControls.svelte`, `./schema-inspect`, `./kvasir-types`
- `FormatControls.svelte` imports `./kvasir-types`
- `JsonlViewer.svelte` imports `./kvasir-types`
- `TableViewer.svelte` imports `./kvasir-types`
- `SchemaInspector.svelte` imports `./schema-inspect`

The `@yggdrasil/ui` imports (SvalinnView, KvasirView, JsonlViewer, RatatoskrView) use the npm package name, which resolves via `node_modules` -- this is correct and works identically in standalone and Yggdrasil modes.

`$lib/` is used only in `*/src/routes/+page.svelte` files (the standalone app entry points), where it correctly resolves to the app's own `src/lib/`.

---

### N-4: INV-4 / DRIFT-4 — Command prefixing: PASS

**Severity:** note

**Description:** Every `invoke()` call in view components uses `commands.X`:
- HlidskjalfView: `commands.speak`, `commands.start_monitor`
- SvalinnView: `commands.scan_directory`, `commands.list_qa_tree`, `commands.run_saga`, `commands.open_in_editor`
- KvasirView: `commands.list_directory`, `commands.read_file`, `commands.open_in_editor`, `commands.convert_to_all_formats`, `commands.detect_data_format`
- JsonlViewer: `commands.read_jsonl_info`, `commands.read_jsonl_entry`, `commands.export_entry_as`, `commands.convert_to_all_formats`, `commands.open_in_editor`
- TableViewer: `commands.read_table`, `commands.export_table_csv`, `commands.open_in_editor`
- RatatoskrView: `commands.generate_sample_graph`, `commands.get_graph_stats`, `commands.load_graph`, `commands.save_graph`

Zero hardcoded invoke strings in any view component.

Standalone apps use bare command names (defaults in `commands` prop). Yggdrasil overrides with prefixed names:
- `hlid_start_monitor`, `hlid_speak`
- `sval_scan_directory`, `sval_list_qa_tree`, `sval_open_in_editor`, `sval_run_saga`
- `kvas_list_directory`, `kvas_read_file`, `kvas_open_in_editor`, `kvas_convert_to_all_formats`, `kvas_detect_data_format`, `kvas_read_jsonl_info`, `kvas_read_jsonl_entry`, `kvas_export_entry_as`, `kvas_read_table`, `kvas_export_table_csv`
- `rata_load_graph`, `rata_save_graph`, `rata_get_graph_stats`, `rata_generate_sample_graph`

All prefixed commands in Yggdrasil's lib.rs match what +page.svelte passes. All bare commands in standalone libs match what the view defaults expect.

KvasirView also correctly forwards `commands` subsets to child components (JsonlViewer, TableViewer) -- the command map flows through the component tree.

---

### N-5: INV-5 / DRIFT-5 — Datagram protocol: PASS with minor observation

**Severity:** note

**Description:**

**Rust side:** `hlidskjalf_core` re-exports `Datagram`, `DatagramKind`, `Priority` from the `datagram` crate. The `parse_event` function tries `Datagram` first, falls back to `HookEvent` with a `From<HookEvent> for Datagram` conversion. `HookEvent` is private, properly marked as legacy, and only used for backward compatibility in the parser. The Tauri event name is `"datagram"` in both standalone (`hlidskjalf/src-tauri/src/lib.rs` line 9) and Yggdrasil (`yggdrasil/src-tauri/src/lib.rs` line 17). No references to `"hook-event"` remain in any Tauri event emission.

**Svelte side:** HlidskjalfView.svelte listens on `"datagram"` (line 101). Priority and kind comparisons use lowercase string matching (`ev.kind === "quality"`, `ev.priority === "critical"`), which is correct since the Rust enums serialize to lowercase strings via serde. The `Datagram` TypeScript interface (lines 20-31) matches the JSON schema fields.

**Schema consistency:** The JSON schema enums (`["alert", "canary", "notify", "quality", "traffic"]` for kind, `["critical", "high", "normal", "low", "trace"]` for priority) match the Rust enum variants after lowercase transformation. The `classifier` field is optional in the schema (not in `required`), matching the Rust `Option<String>`. The `detail`, `speech`, and `payload` fields are also optional in both schema and Rust struct.

**Observation:** The Svelte `Datagram` interface types `kind` and `priority` as `string` rather than union literal types (`"alert" | "canary" | "notify" | "quality" | "traffic"`). This is technically correct but loses type safety -- a comparison like `ev.kind === "alrt"` (typo) compiles without error. This is not a violation, just a missed opportunity for TypeScript to catch string-based comparison bugs.

---

### N-6: INV-6 — Shared UI consistency: PASS

**Severity:** note

**Description:** The `@yggdrasil/ui` package provides 12 components. Current usage across view components:

| Component | Used By |
|-----------|---------|
| `Button` | SvalinnView, KvasirView, JsonlViewer, RatatoskrView |
| `SidebarLayout` | SvalinnView, KvasirView |
| `TreeNode` | SvalinnView, KvasirView |
| `StatCard` | SvalinnView |
| `SearchInput` | SvalinnView |
| `FilterBanner` | SvalinnView |
| `Badge` | (not used in view components) |
| `Input` | (not used in view components) |
| `Select` | (not used in view components) |
| `Panel` | (not used in view components) |
| `Collapsible` | (not used in view components) |
| `ListItem` | (not used in view components) |

HlidskjalfView does not use any shared UI components -- it uses entirely custom markup. This is intentional (it's a real-time event feed with specialized rendering needs), but it means HlidskjalfView does not benefit from shared component updates.

All five apps import `@yggdrasil/ui/css/tokens.css` and `@yggdrasil/ui/css/base.css` via their `+layout.svelte`. CSS token usage is consistent, with the exception of `--severity-info` (see W-1).

---

## Not Checked (Out of Scope)

- **Build verification:** Did not run `cargo check` or `npm run build`. This is a static code audit.
- **Runtime behavior:** Did not verify that invoke calls actually succeed or that events are received.
- **Nornir/datagram crate source:** The `datagram` crate lives in `~/.ai/smidja/nornir/` -- its internal struct was not re-verified against the schema in this audit.
- **Previous audit findings:** The `audit/NAMING_AUDIT.md`, `audit/RUST_BACKEND_AUDIT.md`, and `audit/SVELTE_FRONTEND_AUDIT.md` contain additional findings from prior audits. This audit focused specifically on architectural invariants and drift patterns.
