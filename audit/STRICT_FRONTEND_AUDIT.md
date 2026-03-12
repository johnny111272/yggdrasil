# Strict Frontend Audit — 2026-03-11

Auditor: Claude Opus 4.6 (secondary audit — verifying completeness of first round fixes)

---

## Per-Component Results

### HlidskjalfView.svelte
**Path:** `/Users/johnny/.ai/smidja/yggdrasil/hlidskjalf/src/lib/HlidskjalfView.svelte`

- **Imports:** PASS — Only import from `src/lib/` is `./QualityReport.svelte` (relative). Other imports are `@tauri-apps/api/core`, `@tauri-apps/api/event`, and `svelte` (all correct).
- **invoke() calls:** PASS — Two invoke calls, both use `commands.X` pattern:
  - Line 108: `invoke(commands.speak, { text: ev.speech })`
  - Line 119: `invoke(commands.start_monitor)`
- **CSS variables:** PASS — All CSS variables used are defined in `tokens.css`:
  `--bg-primary`, `--bg-secondary`, `--bg-tertiary`, `--bg-hover`, `--text-primary`, `--text-secondary`, `--text-muted`, `--font-mono`, `--space-sm`, `--space-md`, `--space-lg`, `--space-xs`, `--text-lg`, `--text-sm`, `--text-xs`, `--text-xl`, `--radius-full`, `--radius-sm`, `--severity-error`, `--severity-success`, `--severity-warning`, `--action-primary`
- **Command prop completeness:** PASS — Prop type declares `{ start_monitor: string; speak: string }`. Defaults match standalone app's registered commands: `start_monitor`, `speak`. Both match `hlidskjalf/src-tauri/src/lib.rs` handler registration.

### QualityReport.svelte
**Path:** `/Users/johnny/.ai/smidja/yggdrasil/hlidskjalf/src/lib/QualityReport.svelte`

- **Imports:** PASS — No imports from `$lib/`. No external module imports at all. Self-contained component.
- **invoke() calls:** PASS — No invoke calls. Pure display component receiving data via props.
- **CSS variables:** PASS — All CSS variables used are defined in `tokens.css`:
  `--font-mono`, `--text-sm`, `--space-md`, `--space-lg`, `--space-sm`, `--space-xs`, `--severity-error`, `--severity-warning`, `--severity-success`, `--bg-tertiary`, `--bg-primary`, `--text-primary`, `--text-secondary`, `--text-muted`, `--text-xs`, `--radius-sm`, `--action-primary`
- **Command prop completeness:** N/A — Not a view component. Receives `payload`, `workspace`, `timestamp` props. No commands prop.

### SvalinnView.svelte
**Path:** `/Users/johnny/.ai/smidja/yggdrasil/svalinn/src/lib/SvalinnView.svelte`

- **Imports:** PASS — Imports from `@tauri-apps/api/core`, `@tauri-apps/plugin-dialog`, `@yggdrasil/ui`. No `$lib/` imports. Shared UI components imported from package correctly.
- **invoke() calls:** PASS — All five invoke calls use `commands.X` pattern:
  - Line 95: `invoke<ScanResult>(commands.scan_directory, ...)`
  - Line 108: `invoke<FileTreeEntry[]>(commands.list_qa_tree, ...)`
  - Line 140: `invoke<FileTreeEntry[]>(commands.list_qa_tree, ...)`
  - Line 175: `invoke<SagaResult>(commands.run_saga, ...)`
  - Line 187: `invoke(commands.open_in_editor, ...)`
- **CSS variables:** PASS — All CSS variables used are defined in `tokens.css`:
  `--space-2xl`, `--space-md`, `--space-lg`, `--space-xl`, `--space-sm`, `--space-xs`, `--text-primary`, `--text-secondary`, `--text-lg`, `--text-sm`, `--bg-secondary`, `--bg-primary`, `--bg-hover`, `--border-default`, `--border-subtle`, `--radius-sm`, `--radius-md`, `--radius-full`, `--font-mono`, `--font-body`, `--action-neutral`, `--action-neutral-hover`, `--action-primary`, `--severity-blocked`, `--severity-error`, `--severity-warning`, `--severity-success`
- **Command prop completeness:** PASS — Prop type declares `{ scan_directory, list_qa_tree, open_in_editor, run_saga }`. All four match `svalinn/src-tauri/src/lib.rs` registered commands.

### KvasirView.svelte
**Path:** `/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/lib/KvasirView.svelte`

- **Imports:** PASS — All sibling imports use `./` relative paths:
  - `./MarkdownPreview.svelte`
  - `./SchemaInspector.svelte`
  - `./JsonlViewer.svelte`
  - `./TableViewer.svelte`
  - `./FormatControls.svelte`
  - `./schema-inspect`
  - `./kvasir-types`
  - External imports: `@tauri-apps/api/core`, `@tauri-apps/plugin-dialog`, `@yggdrasil/ui`, `highlight.js`, `marked`
  - No `$lib/` imports.
- **invoke() calls:** PASS — All eight invoke calls use `commands.X` pattern:
  - Line 97: `invoke<FileTreeEntry[]>(commands.list_directory, ...)`
  - Line 139: `invoke<FileTreeEntry[]>(commands.list_directory, ...)`
  - Line 169: `invoke<string | null>(commands.detect_data_format, ...)`
  - Line 196: `invoke<FileContent>(commands.read_file, ...)`
  - Line 215: `invoke<AllFormats>(commands.convert_to_all_formats, ...)`
  - Line 248: `invoke(commands.open_in_editor, ...)`
- **CSS variables:** PASS — All CSS variables used are defined in `tokens.css`:
  `--space-xs`, `--space-sm`, `--space-md`, `--space-lg`, `--space-xl`, `--space-2xl`, `--space-3xl`, `--text-primary`, `--text-secondary`, `--text-sm`, `--text-xs`, `--text-lg`, `--bg-primary`, `--bg-secondary`, `--bg-hover`, `--border-default`, `--radius-sm`, `--radius-md`, `--font-body`, `--font-mono`, `--action-neutral`, `--action-neutral-hover`, `--action-primary`, `--severity-error`
- **Command prop completeness:** PASS — Prop type declares `{ list_directory, read_file, open_in_editor, convert_to_all_formats, detect_data_format, read_jsonl_info, read_jsonl_entry, export_entry_as, read_table, export_table_csv }`. All 10 match `kvasir/src-tauri/src/lib.rs` registered commands (plus `get_pending_file` which is shell-level, not in the commands prop).

### FormatControls.svelte
**Path:** `/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/lib/FormatControls.svelte`

- **Imports:** PASS — Only import is `./kvasir-types` (relative). No `$lib/` imports.
- **invoke() calls:** PASS — No invoke calls. Pure UI component.
- **CSS variables:** PASS — All CSS variables used are defined in `tokens.css`:
  `--bg-secondary`, `--bg-hover`, `--space-lg`, `--space-sm`, `--space-xs`, `--radius-sm`, `--radius-md`, `--text-sm`, `--text-xs`, `--text-primary`, `--text-secondary`, `--action-neutral`, `--action-neutral-hover`, `--action-primary`, `--severity-success`, `--severity-warning`
- **Command prop completeness:** N/A — Not a view component. Receives `dataFormats` and `selectedFormat` as bindable props.

### JsonlViewer.svelte
**Path:** `/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/lib/JsonlViewer.svelte`

- **Imports:** PASS — Imports from `@tauri-apps/api/core`, `@yggdrasil/ui`, `highlight.js`, `./kvasir-types`. No `$lib/` imports.
- **invoke() calls:** PASS — All five invoke calls use `commands.X` pattern:
  - Line 38: `invoke<JsonlInfo>(commands.read_jsonl_info, ...)`
  - Line 40: `invoke<JsonlEntry>(commands.read_jsonl_entry, ...)`
  - Line 56: `invoke<JsonlEntry>(commands.read_jsonl_entry, ...)`
  - Line 86: `invoke<AllFormats>(commands.convert_to_all_formats, ...)`
  - Line 95: `invoke<string>(commands.export_entry_as, ...)`
  - Line 101: `invoke(commands.open_in_editor, ...)`
- **CSS variables:** PASS — All CSS variables used are defined in `tokens.css`:
  `--bg-secondary`, `--bg-tertiary`, `--space-lg`, `--space-md`, `--space-sm`, `--space-xs`, `--space-xl`, `--space-3xl`, `--radius-sm`, `--radius-md`, `--text-sm`, `--text-primary`, `--text-secondary`, `--border-default`, `--font-mono`, `--action-neutral`, `--action-neutral-hover`, `--action-primary`
- **Command prop completeness:** PASS — Commands prop declares `{ read_jsonl_info, read_jsonl_entry, export_entry_as, convert_to_all_formats, open_in_editor }`. All are subset of KvasirView's command map, passed down correctly at lines 421-427 of KvasirView.svelte.

### TableViewer.svelte
**Path:** `/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/lib/TableViewer.svelte`

- **Imports:** PASS — Imports from `@tauri-apps/api/core` and `./kvasir-types`. No `$lib/` imports.
- **invoke() calls:** PASS — All three invoke calls use `commands.X` pattern:
  - Line 65: `invoke<TableData>(commands.read_table, ...)`
  - Line 96: `invoke<string>(commands.export_table_csv, ...)`
  - Line 101: `invoke(commands.open_in_editor, ...)`
- **CSS variables:** FAIL — Multiple undefined CSS variables with hardcoded fallbacks:
  - `--border-color` (lines 173, 180, 211, 257) — NOT in tokens.css. Should use `--border-default`.
  - `--bg-input` (line 179) — NOT in tokens.css. Should use `--bg-primary`.
  - `--bg-badge` (line 201) — NOT in tokens.css.
  - `--bg-button` (line 210) — NOT in tokens.css. Should use `--action-neutral`.
  - `--bg-button-hover` (line 218) — NOT in tokens.css. Should use `--action-neutral-hover`.
  - `--bg-header` (line 256) — NOT in tokens.css. Should use `--bg-primary`.
  - `--bg-header-hover` (line 268) — NOT in tokens.css. Should use `--bg-hover`.
  - `--border-subtle` used on line 273 with fallback `#222` — IS in tokens.css (value: `#262640`). Fallback is inconsistent.
  - `--text-muted` used on lines 187, 193, 229 with fallback `#666`/`#888` — IS in tokens.css (value: `#ccc`). Fallback values are inconsistent.
  - `--text-primary` used with fallback `#e0e0e0` — tokens define `#eee`. Minor inconsistency.
  - `--text-secondary` used with fallback `#aaa`/`#ccc` — tokens define `#888`. Inconsistent fallbacks.
  - `--severity-error` used with fallback `#f44` — tokens define `#ff3333`. Minor inconsistency.
  - `--font-mono` used with fallback `"SF Mono", "Fira Code", monospace` — tokens define a different stack.
  - `--bg-alt-row` (line 279) — NOT in tokens.css.
  - `--bg-row-hover` (line 283) — NOT in tokens.css.

  **Impact:** The component works because it provides CSS fallback values, but it does not use the shared design tokens. If a theme change is applied to `tokens.css`, TableViewer will NOT respond — it will use its hardcoded fallbacks for the 9 undefined variables.

- **Command prop completeness:** PASS — Commands prop declares `{ read_table, export_table_csv, open_in_editor }`. All are subset of KvasirView's command map.

### SchemaInspector.svelte
**Path:** `/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/lib/SchemaInspector.svelte`

- **Imports:** PASS — Only import is `./schema-inspect` (relative). No `$lib/` imports.
- **invoke() calls:** PASS — No invoke calls. Pure display component.
- **CSS variables:** PASS (with note) — Structural tokens all correctly reference `tokens.css` variables (`--bg-secondary`, `--bg-hover`, `--border-default`, `--border-subtle`, `--text-primary`, `--text-secondary`, `--space-*`, `--radius-*`, `--font-mono`, `--action-primary`). Hardcoded hex colors (`#7aa2f7`, `#7dcfff`, `#9ece6a`, `#bb9af7`, `#e0af68`, `#f7768e`, `#73daca`) are intentional Tokyo Night syntax-highlighting colors used for schema field visualization — these are semantic syntax colors that should NOT be design tokens.
- **Command prop completeness:** N/A — Not a view component. Receives `schema` prop only.

### MarkdownPreview.svelte
**Path:** `/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/lib/MarkdownPreview.svelte`

- **Imports:** PASS — No imports at all. Self-contained rendering component.
- **invoke() calls:** PASS — No invoke calls. Pure display component.
- **CSS variables:** PASS — All CSS variables used are defined in `tokens.css`:
  `--bg-secondary`, `--bg-primary`, `--radius-md`, `--space-2xl`, `--space-lg`, `--space-md`, `--space-sm`, `--space-xs`, `--space-xl`, `--text-primary`, `--text-secondary`, `--text-muted`, `--text-3xl`, `--text-2xl`, `--text-xl`, `--border-default`, `--border-subtle`, `--radius-sm`, `--font-mono`, `--action-primary`
- **Command prop completeness:** N/A — Not a view component. Receives `content` prop only.

### RatatoskrView.svelte
**Path:** `/Users/johnny/.ai/smidja/yggdrasil/ratatoskr/src/lib/RatatoskrView.svelte`

- **Imports:** PASS — Imports from `@tauri-apps/api/core`, `@tauri-apps/plugin-dialog`, `@yggdrasil/ui`, `svelte`, `d3`. No `$lib/` imports.
- **invoke() calls:** PASS — All five invoke calls use `commands.X` pattern:
  - Line 91: `invoke<GraphData>(commands.generate_sample_graph)`
  - Line 92: `invoke<GraphStats>(commands.get_graph_stats, ...)`
  - Line 108: `invoke<GraphData>(commands.load_graph, ...)`
  - Line 109: `invoke<GraphStats>(commands.get_graph_stats, ...)`
  - Line 126: `invoke(commands.save_graph, ...)`
- **CSS variables:** PASS — All CSS variables used are defined in `tokens.css`:
  `--space-2xl`, `--space-xl`, `--space-lg`, `--space-md`, `--space-sm`, `--space-xs`, `--text-primary`, `--text-secondary`, `--text-muted`, `--text-lg`, `--text-sm`, `--text-xs`, `--text-base`, `--bg-primary`, `--bg-secondary`, `--bg-hover`, `--border-default`, `--radius-sm`, `--radius-md`, `--font-mono`, `--action-primary`, `--severity-error`
- **Command prop completeness:** PASS — Prop type declares `{ load_graph, save_graph, get_graph_stats, generate_sample_graph }`. All four match `ratatoskr/src-tauri/src/lib.rs` registered commands.

---

## Command Map Verification

### Hlidskjalf standalone vs Yggdrasil

| Command (standalone) | View prop default | Yggdrasil `lib.rs` registered | Yggdrasil `+page.svelte` mapping |
|---|---|---|---|
| `start_monitor` | `start_monitor` | `hlid_start_monitor` | `start_monitor: "hlid_start_monitor"` |
| `speak` | `speak` | `hlid_speak` | `speak: "hlid_speak"` |

**Result: PASS** — All 2 commands mapped correctly.

### Svalinn standalone vs Yggdrasil

| Command (standalone) | View prop default | Yggdrasil `lib.rs` registered | Yggdrasil `+page.svelte` mapping |
|---|---|---|---|
| `scan_directory` | `scan_directory` | `sval_scan_directory` | `scan_directory: "sval_scan_directory"` |
| `list_qa_tree` | `list_qa_tree` | `sval_list_qa_tree` | `list_qa_tree: "sval_list_qa_tree"` |
| `open_in_editor` | `open_in_editor` | `sval_open_in_editor` | `open_in_editor: "sval_open_in_editor"` |
| `run_saga` | `run_saga` | `sval_run_saga` | `run_saga: "sval_run_saga"` |

**Result: PASS** — All 4 commands mapped correctly.

### Kvasir standalone vs Yggdrasil

| Command (standalone) | View prop default | Yggdrasil `lib.rs` registered | Yggdrasil `+page.svelte` mapping |
|---|---|---|---|
| `list_directory` | `list_directory` | `kvas_list_directory` | `list_directory: "kvas_list_directory"` |
| `read_file` | `read_file` | `kvas_read_file` | `read_file: "kvas_read_file"` |
| `open_in_editor` | `open_in_editor` | `kvas_open_in_editor` | `open_in_editor: "kvas_open_in_editor"` |
| `convert_to_all_formats` | `convert_to_all_formats` | `kvas_convert_to_all_formats` | `convert_to_all_formats: "kvas_convert_to_all_formats"` |
| `detect_data_format` | `detect_data_format` | `kvas_detect_data_format` | `detect_data_format: "kvas_detect_data_format"` |
| `read_jsonl_info` | `read_jsonl_info` | `kvas_read_jsonl_info` | `read_jsonl_info: "kvas_read_jsonl_info"` |
| `read_jsonl_entry` | `read_jsonl_entry` | `kvas_read_jsonl_entry` | `read_jsonl_entry: "kvas_read_jsonl_entry"` |
| `export_entry_as` | `export_entry_as` | `kvas_export_entry_as` | `export_entry_as: "kvas_export_entry_as"` |
| `read_table` | `read_table` | `kvas_read_table` | `read_table: "kvas_read_table"` |
| `export_table_csv` | `export_table_csv` | `kvas_export_table_csv` | `export_table_csv: "kvas_export_table_csv"` |

**Note:** `get_pending_file` is a shell-level command, not app-specific. It is correctly registered in `yggdrasil/src-tauri/src/lib.rs` without a prefix and invoked directly in `+page.svelte` (not via commands prop). This is architecturally correct.

**Result: PASS** — All 10 commands mapped correctly.

### Ratatoskr standalone vs Yggdrasil

| Command (standalone) | View prop default | Yggdrasil `lib.rs` registered | Yggdrasil `+page.svelte` mapping |
|---|---|---|---|
| `load_graph` | `load_graph` | `rata_load_graph` | `load_graph: "rata_load_graph"` |
| `save_graph` | `save_graph` | `rata_save_graph` | `save_graph: "rata_save_graph"` |
| `get_graph_stats` | `get_graph_stats` | `rata_get_graph_stats` | `get_graph_stats: "rata_get_graph_stats"` |
| `generate_sample_graph` | `generate_sample_graph` | `rata_generate_sample_graph` | `generate_sample_graph: "rata_generate_sample_graph"` |

**Result: PASS** — All 4 commands mapped correctly.

---

## Yggdrasil +page.svelte Additional Checks

- **Shell-level invoke:** `get_pending_file` on line 35 uses a hardcoded string. This is correct — it is a shell-level command (not app-specific), registered without prefix in `yggdrasil/src-tauri/src/lib.rs` line 135. No architectural violation.
- **Tab strip imports:** Uses Vite aliases (`$hlidskjalf`, `$svalinn`, `$kvasir`, `$ratatoskr`). Correct.
- **CSS variables in +page.svelte:** PASS — All variables used (`--bg-primary`, `--bg-secondary`, `--bg-hover`, `--text-primary`, `--text-secondary`, `--border-default`, `--font-mono`, `--space-md`, `--space-xs`, `--radius-sm`, `--action-primary`) are defined in `tokens.css`.

---

## Defects Found

### FAIL-1: TableViewer.svelte — 9 undefined CSS variables

**Severity:** Medium — component functions due to CSS fallback values, but does not participate in the shared design system.

**Undefined variables used (not in `tokens.css`):**
1. `--border-color` (4 occurrences) — should be `--border-default`
2. `--bg-input` (1 occurrence) — should be `--bg-primary`
3. `--bg-badge` (1 occurrence) — no equivalent token, needs decision
4. `--bg-button` (1 occurrence) — should be `--action-neutral`
5. `--bg-button-hover` (1 occurrence) — should be `--action-neutral-hover`
6. `--bg-header` (1 occurrence) — should be `--bg-primary`
7. `--bg-header-hover` (1 occurrence) — should be `--bg-hover`
8. `--bg-alt-row` (1 occurrence) — no equivalent token, uses `rgba(255,255,255,0.02)`
9. `--bg-row-hover` (1 occurrence) — no equivalent token, uses `rgba(255,255,255,0.05)`

**Additional issue:** Fallback values for defined tokens are inconsistent with token values:
- `--text-muted` fallback `#666`/`#888` vs token value `#ccc`
- `--text-primary` fallback `#e0e0e0` vs token value `#eee`
- `--text-secondary` fallback `#aaa`/`#ccc` vs token value `#888`
- `--font-mono` fallback `"SF Mono", "Fira Code", monospace` vs token value `ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace`

**Root cause:** TableViewer appears to have been authored as a standalone component before the design token system was adopted, using its own ad-hoc CSS variable names with hardcoded fallbacks.

**Fix:** Replace all undefined variables with their `tokens.css` equivalents and remove hardcoded fallback values (since all apps import `tokens.css`). For `--bg-alt-row` and `--bg-row-hover`, use inline `rgba()` values directly or add new tokens.

---

## Summary

| Category | Count |
|----------|-------|
| Components audited | 10 (.svelte files) + 1 (+page.svelte) |
| Tauri lib.rs files audited | 5 |
| Import checks ($lib/ violations) | 0 found — **ALL PASS** |
| invoke() pattern checks | 0 hardcoded strings in view components — **ALL PASS** |
| CSS variable checks | 1 component with 9 undefined variables — **1 FAIL** |
| Command prop completeness | All 20 commands across 4 apps verified — **ALL PASS** |
| Yggdrasil command map completeness | All 20 prefixed + 1 shell-level verified — **ALL PASS** |

**Final tally: 9 PASS, 1 FAIL**

The single failure is `TableViewer.svelte` using 9 CSS variable names that do not exist in `tokens.css`. All other checks pass across all components: no `$lib/` imports in `src/lib/` files, all invoke calls use the `commands.X` pattern, all command props have complete mappings in Yggdrasil, and all Rust backend registrations match.
