# Naming Audit -- Yggdrasil

## Verdict

The naming is substantially good. The Norse mythology names are consistent and meaningful, the core/shell separation is reflected in crate naming, and the command prefix convention (`hlid_`, `sval_`, `kvas_`, `rata_`) is clean and applied uniformly. The `commands` prop pattern uses bare names that map to prefixed names -- the naming makes this obvious. A fresh LLM would correctly infer most architecture from names alone.

However, there are real naming failures. The worst ones create ambiguity between crates that do different things, use the same name for different concepts in different apps, or rely on prefixes (`Sval`, `Kvas`) that abbreviate already-opaque Norse names into near-meaninglessness. There are also several names that will mislead an LLM into wrong behavior -- particularly around what `list_directory` does in svalinn_core vs kvasir_core, what the `event` variable refers to in Hlidskjalf, and what `SanityReport` has to do with Gleipnir.

---

## Critical Naming Failures

### 1. `list_directory` means different things in different crates

**Current name:** `list_directory` in both `svalinn_core` and `kvasir_core`

**What an LLM infers:** "These are the same function -- they list a directory." An agent working in Yggdrasil's unified shell might call the wrong one, or assume they're interchangeable.

**What they actually do:**
- `svalinn_core::list_directory` returns `SvalFileTreeEntry` with `has_sidecar` and `issue_count` -- it is a QA-aware file tree.
- `kvasir_core::list_directory` returns `KvasFileTreeEntry` with `extension` and `size_bytes` -- it is a general-purpose file browser.

These do fundamentally different things. The names are identical. In the Yggdrasil shell, they are disambiguated only by prefix (`sval_list_directory` vs `kvas_list_directory`), but the bare function names in the core crates are indistinguishable.

**Suggested rename:**
- `svalinn_core::list_directory` -> `svalinn_core::list_directory_with_qa` or `svalinn_core::list_qa_tree`
- Keep `kvasir_core::list_directory` as-is (it is the generic one)

### 2. `SanityReport` struct name hides its origin

**Current name:** `SanityReport` in `svalinn_core/src/lib.rs` (line 60)

**What an LLM infers:** "This is some kind of sanity check report." There is zero connection to Gleipnir, Saga, or the `.qa` sidecar format.

**What it actually is:** The deserialization target for `.qa` sidecar files produced by Saga/Gleipnir. The struct is the internal representation of a Gleipnir quality report.

**Why this matters:** An agent asked to "add a new field from a Gleipnir report" would grep for `gleipnir` or `qa_report` and never find this struct. The name `SanityReport` comes from an older naming in the upstream tool and has drifted from the current vocabulary.

**Suggested rename:** `QaSidecarReport` or `SidecarReport` -- names that match the file format they parse (`.qa` sidecars).

### 3. `SvalFileTreeEntry` and `KvasFileTreeEntry` -- prefixed struct names are cryptic

**Current names:** `SvalFileTreeEntry` (svalinn_core), `KvasFileTreeEntry` (kvasir_core)

**What an LLM infers:** "Sval" and "Kvas" are type prefixes to disambiguate, but an LLM with no context cannot parse `Sval` as "Svalinn" or `Kvas` as "Kvasir". These read as arbitrary 4-character prefixes.

**What they actually are:** File tree entries with app-specific metadata -- QA info for Svalinn, file metadata for Kvasir.

**Suggested rename:**
- `SvalFileTreeEntry` -> `QaFileEntry` (describes what makes it special)
- `KvasFileTreeEntry` -> `BrowsedFileEntry` or just `FileEntry` (it is the generic one)

Alternatively, keep the full app name: `SvalinnFileEntry`, `KvasirFileEntry`. The 4-letter abbreviations save nothing in a struct name.

### 4. `hook-event` Tauri event name is a legacy lie

**Current name:** `"hook-event"` (emitted in `hlidskjalf/src-tauri/src/lib.rs` line 9, listened for in `HlidskjalfView.svelte` line 101)

**What an LLM infers:** "This carries hook events." The name `hook-event` strongly implies the `HookEvent` struct.

**What it actually carries:** `Datagram` structs. The `HookEvent` format is explicitly deprecated in the code comments. The event name is a holdover from before the datagram protocol existed.

**Why this matters:** An agent seeing `listen("hook-event", ...)` would reasonably assume the payload is a `HookEvent` and add `HookEvent` fields to the handler. The code comments say datagrams, but the name says hooks.

**Suggested rename:** `"datagram"` or `"datagram-event"`. This matches the type being emitted.

### 5. `@introspection/ui` -- package name references the pre-rename project

**Current name:** `@introspection/ui` (in `ui/package.json`, imported everywhere)

**What an LLM infers:** "This project is called 'introspection'."

**What is true:** The project was renamed from `introspection/` to `yggdrasil/`. The package name was never updated. The tokens.css header comment still says "Hlidskjalf (file viewer)" which is wrong -- Hlidskjalf is the agent monitor, not the file viewer. And it says "Graph viewer (future)" even though Ratatoskr exists.

**Suggested rename:** `@yggdrasil/ui`. This matches the project name and workspace.

### 6. `start_listener` understates what it does

**Current name:** `start_listener` (Tauri command in hlidskjalf shell, core function `start_all`)

**What an LLM infers:** "This starts a socket listener." That is partially true.

**What it actually does:** The Tauri command calls `start_all()` which: (1) rotates logs, (2) initializes lockfiles, (3) starts the Unix socket listener, AND (4) starts the lockfile monitor. The name `start_listener` covers only step 3.

**Why this matters:** An agent debugging lockfile issues would not look at `start_listener` because lockfile monitoring is not "listening."

The core function `start_all` is better named. The Tauri command should match.

**Suggested rename for the Tauri command:** `start_all` or `start_hlidskjalf` or `start_monitor`. The `start_all` name from core is already accurate.

---

## Structural Naming Debt

### Inconsistent function names across the core/shell boundary

The core crates and their Tauri shells do not use a consistent naming convention for the same operation:

| Core function | Standalone Tauri command | Yggdrasil command |
|---|---|---|
| `start_all` | `start_listener` | `hlid_start_listener` |
| `speak` | `speak` | `hlid_speak` |
| `scan_directory` | `scan_directory` | `sval_scan_directory` |
| `list_directory` | `list_directory` | `sval_list_directory` |
| `open_in_editor` | `open_in_editor` | `sval_open_in_editor` |
| `run_saga` | `run_saga` | `sval_run_saga` |
| `list_directory` | `list_directory` | `kvas_list_directory` |
| `read_file` | `read_file` | `kvas_read_file` |
| `convert_all_formats` | `convert_all_formats` | `kvas_convert_all_formats` |
| `is_data_file` | `is_data_file` | `kvas_is_data_file` |
| `load_graph` | `load_graph` | `rata_load_graph` |
| `save_graph` | `save_graph` | `rata_save_graph` |
| `get_graph_stats` | `get_graph_stats` | `rata_get_graph_stats` |
| `generate_sample_graph` | `generate_sample_graph` | `rata_generate_sample_graph` |

The `start_all` -> `start_listener` mismatch is the only real problem. Everything else is consistent. But the pattern shows that standalone shell commands use bare names identical to core functions, except for Hlidskjalf where `start_all` becomes `start_listener`. This inconsistency is a trap.

### `open_in_editor` lives in `yggdrasil_shared` but its name is app-agnostic

The function `open_in_editor` is defined in `yggdrasil_shared` and re-exported by both `svalinn_core` and `kvasir_core`. This is fine architecturally. But the crate name `yggdrasil_shared` is misleading -- it is not shared Yggdrasil-specific code. It is a utility crate shared by the core crates. The name implies it is specific to the Yggdrasil unified shell.

**Suggested rename:** `yggdrasil_shared` -> `core_shared` or `workspace_utils`. The crate lives in `core/` and serves core crates, not the Yggdrasil app.

### `events` variable name collision across layers

In `HlidskjalfView.svelte`:
- `events` is the state array of datagrams
- `event` is the Tauri event wrapper in the listener callback (`event.payload` extracts the datagram)
- `ev` is the extracted datagram
- The TS interface is called `Datagram`

The variable progression `event.payload` -> `ev` creates a naming chain where `event` refers to a Tauri event wrapper, not a datagram, but `events` (the array) contains datagrams, not Tauri events. An LLM reasoning about "events" would conflate the two.

**Suggested rename:** `events` -> `datagrams` (or `feed`). This matches the type name `Datagram` and avoids collision with the Tauri `event` wrapper.

---

## Per-Layer Findings

### Crate & Directory Names

**Good:**
- `hlidskjalf_core`, `svalinn_core`, `kvasir_core`, `ratatoskr_core` -- the `_core` suffix clearly communicates "pure logic, no Tauri."
- `core/` directory containing all core crates -- self-documenting.
- `schemas/` for the datagram schema -- clear.

**Problems:**
- `yggdrasil_shared` (discussed above) -- lives in `core/` but name implies Yggdrasil-specific.
- `hlidskjalf_lib`, `svalinn_lib`, `kvasir_lib`, `ratatoskr_lib` -- these are the `[lib]` names in Tauri shell Cargo.toml. The `_lib` suffix is inert. It does not communicate "thin Tauri command wrapper." A name like `hlidskjalf_tauri` would be more precise, but this is a Tauri convention and changing it may not be worth the churn.
- `ui/` -- too generic. Every app has UI. This is specifically the shared component library. `shared-ui/` or `ui-kit/` would be more precise. But `ui/` is established convention in many monorepos, so the risk is low.

### Rust Structs & Types

**Good:**
- `Datagram` -- clear, matches the protocol name.
- `HookEvent` -- marked as legacy with a comment. The `From<HookEvent> for Datagram` impl makes the relationship explicit.
- `GraphNode`, `GraphEdge`, `GraphData`, `GraphStats` -- clean, consistent, self-documenting graph vocabulary.
- `Issue`, `ScanResult`, `SagaResult` -- clear in context.
- `FileContent`, `FormatConversion`, `AllFormats` -- descriptive.
- `MergeConfig` -- accurately describes what it configures.

**Problems:**
- `SanityReport` (discussed above) -- misleading name.
- `SvalFileTreeEntry`, `KvasFileTreeEntry` (discussed above) -- cryptic prefixes.
- `datagram_type` field on `Datagram` -- the field is named `type` in JSON (via `#[serde(rename = "type")]`) but `datagram_type` in Rust to avoid keyword collision. This is fine Rust practice. But an LLM reading the Rust code might not realize this maps to the `type` field in the schema. The rename attribute is the correct mechanism but the discrepancy is worth noting.

### Rust Functions & Parameters

**Good:**
- `parse_event` -- accurately describes parsing a line into either format.
- `start_listener`, `start_lockfile_monitor` -- clear what each starts.
- `start_all` -- clear orchestrator name.
- `rotate_log` -- verb + noun, clear action.
- `init_lockfiles` -- clear.
- `speak` -- minimal, correct.
- `scan_directory`, `read_file`, `load_graph`, `save_graph` -- standard CRUD verbs.
- `find_sidecars`, `read_sidecar` -- clear internal operations.
- `jsonld_to_graph` -- clear transformation name.
- `prefix_graph_ids`, `deduplicate_nodes`, `apply_colors` -- all action verbs.

**Problems:**
- `now()` in `hlidskjalf_core` -- too generic. Every crate could have a `now()`. Name should be `unix_timestamp_now()` or `epoch_secs()` to communicate the return type.
- `hlidskjalf_dir()` -- returns a `PathBuf` but the name sounds like a directory, not a path. This is a minor style issue; `hlidskjalf_data_dir()` would be slightly more precise.
- `is_data_file` -- returns `Option<String>` (the format name), not `bool`. This is a naming lie. Functions starting with `is_` should return booleans. An LLM would write `if is_data_file(path) { ... }` which works in Rust (Option is truthy in `if let`) but the intent is wrong.
  **Suggested rename:** `detect_data_format` (which already exists as the internal helper it delegates to!) -- just expose the internal name.
- `count_tokens` -- divides `content.len()` by 4. This is not token counting. It is a byte-based approximation. The name implies precision that does not exist. `estimate_token_count` or `approximate_tokens` would be honest.
- `convert_all_formats` -- the `all` is ambiguous. Does it convert to all formats? From all formats? The name should be `convert_to_all_formats` to clarify direction.
- `generate_sample_graph` -- accurate but will become a lie if/when the sample data changes. However, as a development tool, this is acceptable.
- `parse_saga_output` -- parses the stdout of the `saga` CLI tool. The name is accurate but an agent unfamiliar with the external `saga` tool would not know what "saga output" looks like.
- `extract_merge_config`, `extract_references`, `extract_label`, `extract_type` -- good consistent `extract_` prefix pattern.
- `resolve_reference` -- singular, but could return multiple paths in theory. Currently correct but fragile.

### Svelte Components & Props

**Good:**
- `HlidskjalfView`, `SvalinnView`, `KvasirView`, `RatatoskrView` -- `*View` suffix clearly communicates "top-level view component for this app."
- `QualityReport` -- accurately describes what it renders (renamed from `GleipnirReport`).
- `SchemaInspector` -- clear purpose.
- `commands` prop -- the key architectural prop is well-named. It communicates "a map of command names" without implying what the values are.
- `SidebarLayout`, `Button`, `Badge`, `Input`, `Select`, `Panel`, `StatCard`, `TreeNode`, `Collapsible`, `ListItem`, `SearchInput`, `FilterBanner` -- all clear, standard UI vocabulary.

**Problems:**
- `filter_type` vs `filterType` naming inconsistency in `HlidskjalfView.svelte` -- the state variable is `filter_type` (snake_case) while other state variables in other views use camelCase (`viewMode`, `severityFilter`, `scanResult`). This is a cross-component inconsistency.
  - In HlidskjalfView: `filter_type`, `filter_priority_min`, `speech_threshold`, `auto_scroll`, `feed_element` -- all snake_case.
  - In SvalinnView: `includeTests`, `scanResult`, `sagaRunning`, `viewMode`, `severityFilter` -- all camelCase.
  - In KvasirView: `showTree`, `activeTab`, `dataFormats`, `selectedFormat`, `isDataFile` -- all camelCase.

  The inconsistency is Hlidskjalf-specific. All other apps use camelCase. An LLM would adopt whichever convention it saw first, potentially mixing them.

- `priority_numeric`, `priority_color`, `type_icon` -- snake_case function names in HlidskjalfView. The other views use camelCase (`severityColor`, `relativePath`, `formatBytes`). Same inconsistency.

- `speech_threshold` -- stores a numeric priority level but the name says "threshold." The value 3 means "high and above," 4 means "critical only," 5 means "silent." These are magic numbers with no connection to the name. `speech_min_priority` would be clearer.

- `speech_labels` -- defined as a const array but never used. Dead code with a misleading name (it looks like it should be used in `speech_label()` but isn't).

- `filteredDataByPath` -- verbose and unclear. This is a derived value containing issue counts and max severities per file path. The name does not communicate what data or what the filtering produces. `issueCountsByPath` or `pathIssueStats` would be more precise.

- `filteredStats` -- "filtered stats" of what? In context, it is the issue breakdown (total, blocked, error, warning) after applying current filters. `filteredIssueSummary` would be clearer.

- `groupedIssues` -- returns `[string, Issue[]][]`, an array of tuples. The name suggests a record/object. `issueGroups` would match the return type better.

### CSS Classes & Design Tokens

**Good:**
- Token naming follows a clear hierarchy: `--bg-primary/secondary/tertiary`, `--text-primary/secondary/muted`, `--severity-blocked/error/warning/info`, `--action-primary/neutral/special`.
- Spacing scale: `--space-xs/sm/md/lg/xl/2xl/3xl` -- standard.
- Class names are descriptive: `.watchtower`, `.quality-report`, `.event-row`, `.event-meta`, `.event-priority`.

**Problems:**
- `--severity-info` is green (`#6bcb77`). An LLM would infer "info" means blue (the universal convention). Green is the success/good color. This will cause confusion if anyone adds a blue "informational" severity. **Suggested rename:** `--severity-success` or `--severity-good`.
- `.canary-row` vs `.event-row.canary` -- two different CSS approaches for the same concept (canary events). The class `.canary-row` is on the meta div, while `.event-row.canary` is on the row wrapper. Not wrong, but the dual naming for one concept creates ambiguity.
- tokens.css header comment lists "Hlidskjalf (file viewer)" -- Hlidskjalf is the agent monitor, Kvasir is the file/workspace inspector. This comment is actively wrong and would mislead any reader.
- tokens.css header says "Graph viewer (future)" -- Ratatoskr is the graph viewer and it exists. The comment is stale.

### Infrastructure & Scripts

**Good:**
- `deploy_apps.sh` -- clear purpose.
- `datagram.schema.json` -- matches the protocol name.
- Workspace member paths in Cargo.toml are clean and systematic.

**Problems:**
- `deploy_apps.sh` line 5: `BUNDLE_DIR` hardcodes `release/bundle/macos`. The name `BUNDLE_DIR` is fine but the variable is macOS-specific without indicating that. This is acceptable for a macOS-only project.
- Schema `$id` is `"datagram-schema-v1"` -- this is a version-suffixed ID. The `v1` will become a lie when fields are added without bumping the version. JSON Schema `$id` is typically a URI, not a versioned slug. However, since this is a local schema, the practical impact is low.

---

## Recommendations

Prioritized by impact on LLM navigability:

### Priority 1 -- Rename to prevent misnavigation

1. **`hook-event` -> `datagram`** (Tauri event name in hlidskjalf shell and yggdrasil shell). This is the highest-impact change: the name actively directs agents toward the deprecated `HookEvent` type.

2. **`is_data_file` -> `detect_data_format`** (kvasir_core public function + Tauri command + command map entry). The current name lies about its return type. It already delegates to `detect_data_format` internally. Just expose that name.

3. **`SanityReport` -> `QaSidecarReport`** (svalinn_core internal struct). Agents searching for Gleipnir/QA sidecar handling will never find `SanityReport`.

4. **tokens.css header comment** -- fix "Hlidskjalf (file viewer)" to "Hlidskjalf (agent monitor)" and remove "(future)" from graph viewer. Comments are context for agents.

### Priority 2 -- Rename to reduce ambiguity

5. **`yggdrasil_shared` -> `core_shared`** (crate name + Cargo.toml path). The current name confuses the shared utility crate with the Yggdrasil unified shell app.

6. **`@introspection/ui` -> `@yggdrasil/ui`** (package.json name). The project was renamed; the package was not.

7. **`start_listener` Tauri command -> `start_all` or `start_monitor`** (hlidskjalf/src-tauri and yggdrasil/src-tauri). Align with the core function name.

8. **`SvalFileTreeEntry` / `KvasFileTreeEntry`** -- use full names or descriptive names. At minimum: `SvalinnFileEntry` / `KvasirFileEntry`. Better: `QaFileEntry` / `FileEntry`.

### Priority 3 -- Consistency cleanup

9. **Standardize on camelCase for all Svelte state variables.** HlidskjalfView uses snake_case (`filter_type`, `auto_scroll`, `speech_threshold`). Every other view uses camelCase. Pick one. camelCase is the Svelte community standard.

10. **`events` -> `datagrams`** in HlidskjalfView.svelte. Matches the type name, avoids collision with Tauri `event` wrapper.

11. **`now()` -> `epoch_secs_now()`** in hlidskjalf_core. Too generic for a function that returns a specific representation.

12. **`count_tokens` -> `estimate_token_count`** in kvasir_core. Honest about what it does.

13. **`convert_all_formats` -> `convert_to_all_formats`** in kvasir_core. Clarifies direction.

14. **Remove dead `speech_labels`** const in HlidskjalfView.svelte. Dead code with a name that suggests it should be used.

### Priority 4 -- Nice-to-have

15. **`--severity-info` -> `--severity-success`** in tokens.css. Green is not "info" in any standard palette.

16. **`list_directory` in svalinn_core -> `list_qa_tree`**. Disambiguates from kvasir_core's `list_directory`.
