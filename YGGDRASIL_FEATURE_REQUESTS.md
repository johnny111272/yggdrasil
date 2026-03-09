# YGGDRASIL_FEATURE_REQUESTS

Feature ideas under discussion. Not yet prioritized or scheduled.

---

## FR-1: Cross-App Navigation via Clickable Paths

**Origin:** Datagram payloads often contain file paths (e.g., compaction summaries, exchange diffs). These should be clickable.

**Behavior inside Yggdrasil:**
- Data files (.json, .yaml, .toml, .toon, .schema.json) — switch to Kvasir tab, load the file
- Code files (.py, .rs, .svelte, .ts) — `open_in_editor`
- Everything else — system default (`open`)

**Behavior standalone:**
- `open_in_editor` for code
- `open -a "kvasir"` for data files if Kvasir.app is installed
- System `open` as fallback

**Mechanism:**
- Views accept an optional `navigate` callback prop (parallel to `commands`)
- Hlidskjalf calls `navigate?.("kvasir", { file: "/path/to/file" })`
- Yggdrasil receives it, switches tab, passes file path to KvasirView via prop
- KvasirView watches the prop and loads the file
- File-type routing is a pure function (extension in, action out) — lives in the view, not the wrapper
- Target views accept typed props for incoming navigation: `openFile?: string` for Kvasir

**Key constraint:** Yggdrasil remains a thin wrapper. It doesn't know how Kvasir loads a file — it just passes a typed payload through. No app logic in the wrapper.

**Synergy with FR-2:** Datagrams can include `{ file: "/path/to.jsonl", line: 47 }`. Click in Hlidskjalf → Kvasir opens the JSONL browser at entry 47. Compaction finishes, datagram arrives, click the link, read the record in YAML.

---

## FR-2: JSONL Viewer in Kvasir

**Origin:** JSONL (JSON Lines) is used everywhere — conversation transcripts, log streams, data exports. Kvasir handles JSON/YAML/TOML/TOON but not JSONL.

**Challenge:** JSONL is a stream of independent objects, not a single document. Files can be very large (conversation transcripts, audit logs). Needs different rendering than a single JSON tree.

**Design: Single-entry record browser.**

Not a list, not a table — a single-record viewer with keyboard navigation:

- Open JSONL → shows the **last entry** (most recent = most relevant)
- Position indicator in header: `47 / 1,203`
- **Up/Down arrows** = previous/next entry
- **Left/Right arrows** = first/last entry (jump to beginning/end)
- **Format picker** = view the current entry as JSON, YAML, TOML, or TOON
- `convert_to_all_formats` already exists in Rust — reuse it per-entry

**Why this works:**
- Only one entry parsed and rendered at a time — memory efficient for huge files
- No pagination, no virtualization, no collapsible lists
- Format conversion applies per-entry, using existing infrastructure
- Keyboard-driven navigation is fast and natural
- Starting at the last record matches the typical use case (inspect the most recent event)

**Export and open in editor:**
- "Open in Editor" button exports the current entry in the currently selected format
- Writes to a temp file (e.g., `/tmp/kvasir-entry-55.toml`) and opens in Zed via `open_in_editor`
- Reuses existing `convert_to_all_formats` + `open_in_editor` — just needs a temp file write between them
- Temp file name encodes source context: `kvasir-{filename}-{entry}-{format}`

**Implementation:**
- Rust: `detect_data_format` returns `"jsonl"`. New function reads JSONL, returns entry count + entry at index N. New function exports entry to temp file in chosen format.
- Svelte: JSONL mode in KvasirView — single entry display with nav controls, format selector, keyboard listener.
- The existing code/syntax highlighting and format conversion UI can be reused as-is for the single entry.

---

## FR-3: "Open as Format" in Data Viewer

**Origin:** Kvasir already converts between JSON/YAML/TOML/TOON and shows the result. But sometimes you want to take that conversion into your editor — view a JSON config, realize TOML is more readable, open the TOML version in Zed.

**Behavior:**
- Viewing any data file in Kvasir with a format selected (JSON, YAML, TOML, TOON)
- "Open in Editor" exports the file in the **currently selected format** to a temp file, opens in Zed
- Not a permanent conversion — a temp file for reading/editing
- Temp file name: `kvasir-{filename}.{format}` in `/tmp/`

**Implementation:**
- Rust: new command takes file content + target format, writes temp file, calls `open_in_editor`
- Svelte: the existing "Open in Editor" button becomes format-aware when in data view — opens the converted version, not the original
- Reuses `convert_to_all_formats` (already exists) + `open_in_editor` (already exists)
- Shares the same temp-file-and-open mechanism as FR-2's JSONL export

---

## FR-4: Shared Format Conversion Crate

**Origin:** TOON files don't convert in Kvasir — `serde_toon2 0.1.0` from crates.io doesn't match the TOON format our files actually use. Format conversion logic has been cannibalized into different places across smidja projects instead of living in one shared crate.

**Problem:**
- `kvasir_core` depends on `serde_toon2 = "0.1"` (crates.io) — may not match our TOON dialect
- Format detection, parsing, and conversion logic exists in `kvasir_core` but could be needed by other projects (nornir, bifrost, etc.)
- No single source of truth for "how do we parse/emit JSON, YAML, TOML, TOON, JSONL"

**Solution:**
- Shared format crate in the smidja workspace (like `socket_emit` in nornir) — or a new nornir capability crate
- Handles: detection by extension, parsing, conversion between formats, serialization
- All projects depend on this one crate instead of rolling their own
- TOON support uses our actual format definition, not a random crates.io crate
- Kvasir becomes a thin consumer: `format_crate::detect()`, `format_crate::convert()`

**Formats to support:**
- JSON, YAML, TOML — standard, serde ecosystem
- RON — Rusty Object Notation (`ron` crate). Type-safe Rust config: named structs, enum variants with data, Option<T>. Zero cost until a .ron file is opened.
- TOON — our dialect, not the crates.io `serde_toon2`
- TOMLX — our own format, no external crate exists
- JSONL — line-delimited JSON (FR-2)

**Scope:** Cross-repo — affects nornir and yggdrasil at minimum. Needs investigation into where format code currently lives across all 6 smidja projects. This is a prerequisite for FR-2 (JSONL) and FR-3 (Open as Format) to work correctly with all formats.

---

## FR-5: Lightweight Table Viewer

**Origin:** Opening a CSV in a full spreadsheet is overkill when you just need to sort a column or check a few values. Kvasir is already the "inspect data" tool — tabular data is a natural addition.

**Scope:** Bare bones. Not a spreadsheet. A viewer with just enough interaction to be useful.

**Core features:**
- Import CSV, TSV (detect delimiter)
- Column headers from first row (or auto-generate A, B, C...)
- Click column header to sort (asc/desc/unsorted cycle)
- Basic text filter/search across all columns
- Column resize by dragging
- Row count in header

**Nice to have:**
- Export re-sorted data (CSV/TSV)
- "Open in Editor" for the source file
- Freeze first column option
- Copy cell / copy row

**What it is NOT:**
- No formulas, no cell editing, no charts, no pivot tables
- No multi-sheet support
- Not Excel — just a fast, clean table viewer

**Additional tabular formats:**
- **Parquet**: columnar data format, common in data engineering. Rust `parquet` crate reads schema + row groups. Same table UI, different reader.
- **SQLite**: embedded database. List tables in sidebar, select one to view. Optionally run basic queries. Rust `rusqlite` crate. Same table UI per result set.

Both produce rows + columns — the table viewer component is format-agnostic. Only the Rust reader layer differs.

**Implementation:**
- Rust: CSV (`csv` crate), Parquet (`parquet` crate), SQLite (`rusqlite`) — each returns headers + rows in a common format
- Svelte: single table component with sort state, virtual scrolling for large files
- Fits as another mode in Kvasir alongside code/data/preview/inspect
- Format detection by extension: `.csv`/`.tsv` → CSV, `.parquet` → Parquet, `.sqlite`/`.db` → SQLite

---

## FR-6: Theme Switching

**Origin:** Eye strain from fixed color/contrast combos. Need to swap themes throughout the day based on lighting conditions, fatigue, and screen time. Not cosmetic — accessibility.

**Foundation already exists:** `ui/css/tokens.css` defines all colors as CSS custom properties. Swapping themes = swapping token values. No component changes needed.

**Core features:**
- Multiple theme files (not just light/dark — low contrast, warm, high contrast, etc.)
- Quick switch via keyboard shortcut or persistent UI toggle
- Theme persists across app restarts (Tauri local storage or config file)
- All 5 apps share the same themes via the shared `ui/` package

**Existing research:** `doctools/docbuild/documents/BUILDING_THEMES.md` and `DETERMINING_APPROPRIATE_COLOR_CONTRAST.md` — proven pattern with automated contrast testing.

**Key insight — polarity-aware contrast:**
- Dark-on-light (light themes): target 14-16:1 contrast ratio
- Light-on-dark (dark themes): target 7-10:1 contrast ratio
- Higher contrast on dark backgrounds causes halation (text bleeding/glowing)
- Lower contrast on dark backgrounds reduces eye strain
- Goes beyond WCAG AAA (7:1) for comfort, not just compliance

**Themes (from prior research):**
- **Light**: bg `#ffffff`, text `#1f1f1f` (15.12:1) — bright environments, daytime
- **Darkly**: bg `#1a1a1a`, text `#b2b8c5` (9.71:1) — neutral dark, current default
- **Warm-dark**: bg `#1a1612`, text `#c8c898` (9.52:1) — eye strain relief, evening work, less blue light
- **Cool-dark**: bg `#0f1419`, text `#b2c9d8` (10.15:1) — focus work, crisp and clear
- **High contrast**: TBD — maximum accessibility

**Synergy with NEURODIVERGENT_MODALITIES:** Alert profiles (hyperfocus, sensitive, monitoring, active, silent) could include a preferred theme. Switch to "sensitive" profile → theme auto-shifts to warm-dark.

**Implementation:**
- Proven pattern: `body[data-theme="warm-dark"]` attribute + CSS selectors. Already built for Sphinx docs, adapts to Yggdrasil's token system.
- Theme files: alternate token values per theme in `ui/css/`, keyed off `data-theme` attribute
- Svelte: theme selector component, persists via `localStorage`
- Keyboard shortcut for quick cycling
- Contrast validation: reuse existing Playwright contrast analysis scripts to verify all themes meet polarity targets
- All 5 apps inherit themes automatically via shared `ui/` package

---

## FR-7: Markdown Reading Experience

**Origin:** No existing markdown viewer is satisfying. Kvasir already renders markdown (MarkdownPreview.svelte) — the goal is to make it a genuinely great reading experience, especially for long and dense documentation.

**Reading controls (always visible toolbar):**
- **Theme quick-switch**: 3-5 favorite color theme buttons (not a dropdown — one click). Ties into FR-6 themes but could also have markdown-specific overrides (reading bg slightly different from app bg).
- **Font size slider**: continuous or stepped (12-24px range). Persists across sessions.
- **Font combination selector**: dropdown or slider for 3-5 curated pairings (e.g., heading font + body font). Examples:
  - System sans / monospace (clean, default)
  - Serif body / sans headings (book-like, easy on eyes for long reads)
  - Monospace everything (technical docs)
- **Heading separators**: selectable styles for visual breaks between sections — horizontal rules, colored bars, extra whitespace. Helps navigation in long dense docs where headings blur together.

**Auto-generated TOC sidebar:**
- Parse headings (h1-h6) from rendered markdown, display as navigable tree
- Click to jump to section
- Highlight current section on scroll (intersection observer)
- Indentation reflects heading depth
- Collapsible for deeply nested docs
- Toggle on/off — some docs are short enough not to need it

**Reading aids for dense docs:**
- Adjustable paragraph spacing (compact vs airy)
- Optional line-height control
- Focus mode: dim everything except the current section

**What it is NOT:**
- Not an editor — no editing, no split pane, no live preview of edits
- Not a full theming engine — curated presets, not infinite customization

**Implementation:**
- MarkdownPreview.svelte already exists — extend it with a toolbar
- Reading preferences stored in `localStorage`, applied as CSS custom properties
- Font combinations as predefined CSS class sets
- Heading separators as CSS `::after` pseudo-elements on heading tags
- The controls are markdown-specific, not app-wide (though theme buttons sync with FR-6)
