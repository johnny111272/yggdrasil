# Rust Backend Audit -- Yggdrasil

**Auditor:** Claude Opus 4.6 (automated)
**Date:** 2026-03-08
**Scope:** All Rust source files in the workspace (5 core crates, 5 Tauri shells)
**Verdict:** Structurally sound architecture with serious gaps in error handling, test coverage, type safety, and code duplication. Not production-ready.

---

## Critical (must fix before any new feature work)

### C1. Zero test coverage across the entire workspace

There is not a single `#[test]`, `#[cfg(test)]`, or test module anywhere in the codebase. Zero. Across 5 core crates and 5 Tauri shells.

This is not a style issue. This is a structural defect. Every function in every core crate is untested. The JSON-LD parser in `ratatoskr_core` is ~400 lines of recursive graph traversal with zero verification. The datagram parser in `hlidskjalf_core` does two-format fallback parsing with zero verification. The format converter in `kvasir_core` does round-trip conversion across 4 serialization formats with zero verification.

There is no way to know if any of this code works correctly except by running the full application and manually testing. Refactoring is dangerous. Regression is undetectable.

**Files:** Every `core/*/src/lib.rs`

### C2. Datagram struct uses stringly-typed fields where the schema defines enums

`core/hlidskjalf_core/src/lib.rs`, lines 8-22:

```rust
pub struct Datagram {
    pub datagram_type: String,  // Schema says: enum ["alert", "report", "canary", "notify"]
    pub priority: String,       // Schema says: enum ["critical", "high", "normal", "low", "trace"]
    pub source: String,         // Schema says: pattern "^[a-z0-9_]{1,64}$"
    pub workspace: String,      // Schema says: maxLength 128
}
```

The schema at `schemas/datagram.schema.json` defines `type` as an enum of 4 values and `priority` as an enum of 5 values. The Rust struct uses `String` for both. This means:
- No compile-time validation of datagram types or priorities
- The `From<HookEvent>` implementation (line 51-63) constructs priority strings inline ("high", "normal", "low") -- if someone typos one, it compiles fine and silently breaks filtering
- The lockfile monitor (lines 227-251) writes `"critical"` as a string literal -- there is no guarantee this matches the schema enum
- The schema says the `type` enum is `["alert", "report", "canary", "notify"]` but the lockfile monitor writes `"alert"` -- fine today, but nothing enforces this

This should be an enum. The schema is the source of truth; the Rust types should mirror it.

### C3. `open_in_editor` in `yggdrasil_shared` passes unsanitized user input to a shell command

`core/yggdrasil_shared/src/lib.rs`, lines 1-8:

```rust
pub fn open_in_editor(path: &str, line: usize) -> Result<(), String> {
    std::process::Command::new("code")
        .args(["--goto", &format!("{}:{}", path, line)])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

The `path` parameter comes directly from the frontend via Tauri commands (`sval_open_in_editor`, `kvas_open_in_editor`). There is no validation that:
- The path is a real file
- The path does not contain shell metacharacters
- The path is within an allowed directory

`Command::new` with `.args()` does not invoke a shell, so shell injection is not the risk here -- but arbitrary file paths can still be passed. A malicious or buggy frontend could open any file on the system. For a desktop app with IPC from a webview, this is a real attack surface.

Additionally, this function spawns a process and immediately drops the `Child` handle. The spawned process becomes orphaned. This is fire-and-forget by design, but the function returns `Ok(())` before `code` has even started -- the caller cannot know if the editor actually opened.

### C4. `speak()` passes unsanitized user input to an OS command

`core/hlidskjalf_core/src/lib.rs`, lines 282-291:

```rust
pub fn speak(text: &str) {
    if text.is_empty() {
        return;
    }
    let _ = std::process::Command::new("say")
        .args(["-v", "Fiona", text])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}
```

The `text` parameter flows from the frontend via `hlid_speak`. Same issue as C3: `.args()` prevents shell injection, but there is zero validation of the input content. The `say` command on macOS will speak whatever text is passed. In the context of this application, `text` comes from datagram `speech` fields, which are received over a Unix socket from external processes. A datagram producer can make the computer say anything.

The result of `spawn()` is discarded with `let _ =`. If `say` does not exist on the system, this silently does nothing.

### C5. Hardcoded socket path with no configurability

`core/hlidskjalf_core/src/lib.rs`, line 96:

```rust
let socket_path = "/tmp/hlidskjalf.sock";
```

This path is:
- Hardcoded, not configurable
- In `/tmp/`, which is world-writable on macOS
- Not namespaced to the user (another user on the same machine could create this path first)
- Deleted unconditionally on startup (line 99: `let _ = std::fs::remove_file(socket_path)`) -- if another instance is running, this silently destroys its socket

There is no check for an existing running instance. Two Hlidskjalf processes will fight over the socket file.

---

## Structural (architectural debt that compounds)

### S1. Duplicated `list_directory` implementations across svalinn_core and kvasir_core

`core/svalinn_core/src/lib.rs` lines 227-292 and `core/kvasir_core/src/lib.rs` lines 45-95 both implement directory listing with the same pattern:

1. Check if path is a directory
2. Read entries, skip dotfiles
3. Sort directories-first then alphabetical

The differences:
- Svalinn adds sidecar detection and issue counting
- Kvasir adds extension detection and file size

The shared pattern (read dir, skip dotfiles, sort dirs-first) is the same 20+ lines of code in both. This should be in `yggdrasil_shared` or a shared utility, with per-crate extensions.

Both also define their own `FileTreeEntry` struct (`SvalFileTreeEntry`, `KvasFileTreeEntry`) with overlapping fields (`name`, `path`, `is_dir`). There is an existing `yggdrasil_shared` crate that currently contains only one function. It should hold this shared logic.

### S2. The `SanityReport` / `SidecarIssue` types duplicate the public `Issue` type

`core/svalinn_core/src/lib.rs` defines both:
- `SidecarIssue` (lines 72-85): private struct for deserialization
- `Issue` (lines 10-25): public struct for the API

These have identical fields. The `read_sidecar` function (lines 102-125) manually maps every field from `SidecarIssue` to `Issue`, adding only the `file` field from the parent `SanityReport`. This is 13 lines of boilerplate that could be eliminated by either:
- Adding `file` to `SidecarIssue` after deserialization, or
- Using `#[serde(flatten)]` or a wrapper type

The `SanityReport` struct also has 3 fields annotated `#[allow(dead_code)]` (`relative_path`, `content_hash`, `generated_at` -- lines 62-67). These are deserialized and immediately thrown away. If these fields are not needed, the struct should not include them -- `serde` with `#[serde(deny_unknown_fields)]` off (the default) will simply ignore them.

### S3. Every Tauri shell `run()` function panics on startup failure

All 5 shells end with:

```rust
.run(tauri::generate_context!())
.expect("error while running tauri application");
```

Locations:
- `hlidskjalf/src-tauri/src/lib.rs:26`
- `svalinn/src-tauri/src/lib.rs:30`
- `kvasir/src-tauri/src/lib.rs:35`
- `ratatoskr/src-tauri/src/lib.rs:30`
- `yggdrasil/src-tauri/src/lib.rs:132`

This is a standard Tauri pattern and the `run()` method returning `Err` means the application cannot start at all, so panicking is arguably acceptable here. However, the generic message "error while running tauri application" is useless for debugging. The error should be formatted into the message.

### S4. `hlidskjalf_core` silently swallows all file I/O errors in logging and lockfile operations

`core/hlidskjalf_core/src/lib.rs` has 10 instances of `let _ =` discarding `Result` values:

- Line 99: `let _ = std::fs::remove_file(socket_path)` -- stale socket removal, acceptable
- Line 119: `let _ = tx.send(datagram)` -- if receiver is dropped, events are lost silently
- Line 152: `let _ = std::fs::create_dir_all(&dir)` -- if log directory creation fails, logging silently stops
- Line 160: `let _ = writeln!(f, "{}", json)` -- if log write fails, event is lost
- Line 191: `let _ = std::fs::write(...)` -- if log rotation write fails, entire log is lost
- Line 207: `let _ = std::fs::create_dir_all(&dir)` -- lockfile dir creation failure silently ignored
- Line 210: `let _ = std::fs::File::create(&path)` -- lockfile creation failure silently ignored
- Lines 237, 251: `let _ = sender.send(dg)` -- critical lockfile alerts can be silently lost

Line 191 is the worst: `rotate_log()` reads the entire log file, filters lines, then writes back. If the write fails, the log contents are gone -- they were already read into memory and the old file will be overwritten (or not, but the `let _ =` means the caller does not know).

### S5. `run_saga` constructs a PATH with hardcoded directories

`core/svalinn_core/src/lib.rs`, lines 183-186:

```rust
let extended_path = format!(
    "{}/.local/bin:/opt/homebrew/bin:/usr/local/bin:{}",
    home, path
);
```

This hardcodes `/opt/homebrew/bin` (Apple Silicon Homebrew) and `~/.local/bin` into the PATH. This is a macOS-specific hack that will break on other platforms and makes implicit assumptions about the user's environment.

The saga binary path is also hardcoded (line 174):

```rust
h.join(".ai/phoenix/quality/saga/.venv/bin/saga")
```

This couples `svalinn_core` to a specific directory layout of a separate project (`phoenix`). If that project restructures, this breaks silently.

### S6. `count_tokens` in kvasir_core is a fake implementation

`core/kvasir_core/src/lib.rs`, line 198:

```rust
fn count_tokens(content: &str) -> usize {
    content.len() / 4
}
```

This divides byte length by 4 and calls the result a "token count." This is not tokenization. For UTF-8 content with non-ASCII characters, `content.len()` returns byte count, not character count. The division by 4 is a rough heuristic for English text at best. The function is used to populate `FormatConversion::token_count`, which is presented to the user. It will produce wrong numbers for non-English content, compact JSON, verbose YAML, etc.

If approximate token counts are the goal, this should at minimum be documented as an approximation. Better: use whitespace splitting or a proper tokenizer.

### S7. The `parse_event` function silently drops malformed events

`core/hlidskjalf_core/src/lib.rs`, lines 79-89:

```rust
fn parse_event(line: &str) -> Option<Datagram> {
    if let Ok(dg) = serde_json::from_str::<Datagram>(line) {
        return Some(dg);
    }
    if let Ok(ev) = serde_json::from_str::<HookEvent>(line) {
        return Some(ev.into());
    }
    None
}
```

The two-pass parsing strategy (try Datagram, fall back to HookEvent) has a subtle problem: `Datagram` has no required fields that `HookEvent` lacks, and `HookEvent` has more required fields than `Datagram`. A valid `HookEvent` will often parse successfully as a `Datagram` (since all of Datagram's fields exist on HookEvent, and Datagram's non-overlapping fields are `Option`). This means the HookEvent-to-Datagram conversion logic (the `From` impl) may never execute for events that happen to have both formats' required fields present.

This is fragile. The behavior depends on field naming coincidences between the two formats.

### S8. `hlidskjalf_core::Datagram` is not in a shared crate

`Datagram` is defined in `core/hlidskjalf_core/src/lib.rs`. If any other core crate ever needs to produce or consume datagrams (which the architecture implies -- Bifrost integration, exchange diffs, etc.), they will need to depend on `hlidskjalf_core` or duplicate the type. The `Datagram` struct should live in `yggdrasil_shared`.

Currently `yggdrasil_shared` contains only `open_in_editor`. It is underused.

---

## Specific Findings by Crate

### yggdrasil_shared

**File:** `core/yggdrasil_shared/src/lib.rs` (8 lines)
**Cargo.toml:** Zero dependencies

**Findings:**

1. **Underused.** This crate contains a single 8-line function. It was created to share `open_in_editor` between svalinn and kvasir. But shared types (`Datagram`, `FileTreeEntry` base type, directory listing utilities) that belong here are instead duplicated across other crates.

2. **`open_in_editor` hardcodes VS Code.** Line 3: `Command::new("code")`. If the user uses a different editor, this does nothing useful. There is no configuration, no `$EDITOR` fallback, no error if `code` is not installed (the spawn error is mapped to `Err` but the process may just fail to start).

3. **No path validation.** Covered in C3 above.

### hlidskjalf_core

**File:** `core/hlidskjalf_core/src/lib.rs` (292 lines)
**Cargo.toml:** serde, serde_json, tokio

**Findings:**

1. **C2 above:** Stringly-typed `Datagram` where enums are warranted.

2. **C5 above:** Hardcoded `/tmp/hlidskjalf.sock`.

3. **S4 above:** Pervasive `let _ =` error swallowing.

4. **S7 above:** Fragile two-pass parsing.

5. **S8 above:** `Datagram` defined here instead of in shared crate.

6. **`rotate_log` is not atomic.** Lines 166-192: the function reads the entire log file into memory, filters it, and writes back. If the process crashes between the read and the write, the log is lost. If another process appends to the log during this window, those events are lost. This should use a temporary file and atomic rename.

7. **`hlidskjalf_dir` uses `$HOME` with a hardcoded fallback to `/tmp`.** Line 141: `unwrap_or_else(|_| "/tmp".to_string())`. If `$HOME` is unset, all lockfiles and logs go to `/tmp/.ai/hlidskjalf/`. This is wrong -- it should fail, not silently use a world-writable directory.

8. **`start_listener` spawns a background task but returns `Ok(())` immediately.** Lines 93-136: The function binds the socket synchronously (good) but then spawns a `tokio::spawn` for the accept loop and returns. There is no way for the caller to know if the listener fails after startup. Errors in the accept loop go to `eprintln!` and are otherwise swallowed. There is no shutdown mechanism.

9. **`start_lockfile_monitor` has no shutdown mechanism.** Lines 218-255: The spawned task runs forever. There is no cancellation token, no way to stop it. If the sender is dropped, `sender.send()` returns `Err` which is discarded with `let _ =`, and the monitor keeps running and checking files every 5 seconds forever.

10. **The `now()` helper (lines 257-262) returns `0.0` on failure.** If `SystemTime::now()` somehow fails (unlikely but theoretically possible on a badly misconfigured system), datagrams get timestamp `0.0`. This will cause `rotate_log` to immediately delete them (they will always be before the cutoff).

11. **`HookEvent.context_injected` is `#[allow(dead_code)]`.** Line 34-35. This field is deserialized but never used. Unlike the svalinn case, this is on a struct that gets consumed by `From<HookEvent>`, so the field is truly dead. Remove it or use `#[serde(default)]` to make it optional and skip it.

### svalinn_core

**File:** `core/svalinn_core/src/lib.rs` (293 lines)
**Cargo.toml:** yggdrasil_shared, serde, serde_json, glob, dirs

**Findings:**

1. **S1 above:** Duplicated directory listing with kvasir_core.

2. **S2 above:** Duplicated issue types.

3. **S5 above:** Hardcoded PATH and saga binary location.

4. **`list_directory` has an `unwrap()` on line 249.** `path.parent().unwrap()` -- this will panic if `path` is a root path. In practice, this is called on files found by `read_dir` so the parent always exists, but it is still an unnecessary `unwrap()` in a core crate that is supposed to be panic-free.

5. **`list_directory` is expensive.** Lines 258-269: for every directory entry, it globs for all `.qa` sidecars recursively and reads + parses every one of them just to get a count. This means listing a directory with 10 subdirectories triggers 10 recursive glob+parse operations. For a workspace with many files, this could take seconds. There is no caching, no lazy evaluation, no depth limit.

6. **`scan_directory` uses `eprintln!` for errors.** Line 143: `Err(e) => eprintln!("Warning: {}", e)`. In a Tauri desktop app, stderr goes nowhere visible to the user. These errors are effectively lost.

7. **`parse_saga_output` does brittle text parsing.** Lines 213-225: Parses `saga` output by looking for a line starting with "Analyzed" and extracting numbers by position. If `saga` changes its output format, this silently returns `(0, 0)`.

8. **`find_sidecars` does not handle symlink loops.** The glob pattern `{}/**/.*.qa` will follow symlinks. A symlink cycle in the directory tree will cause this to hang or exhaust resources.

9. **Three `#[allow(dead_code)]` annotations on `SanityReport` fields (lines 62-67).** These fields are deserialized and discarded. The `#[allow(dead_code)]` hides the warning but the fields still consume memory during deserialization. Since `serde` ignores unknown fields by default, these fields should be removed from the struct entirely.

### kvasir_core

**File:** `core/kvasir_core/src/lib.rs` (253 lines)
**Cargo.toml:** yggdrasil_shared, serde, serde_json, serde_yaml, toml, serde_toon2

**Findings:**

1. **S6 above:** Fake token counting.

2. **`read_file` does two filesystem calls where one suffices.** Lines 115-119: it calls `read_to_string` and then `metadata` on the same path. The file size could be obtained from `read_to_string`'s result (the string length). Or metadata could be fetched first to check size before reading.

3. **`read_file` has no size limit.** It will happily read a 2GB text file into memory. There should be a size check before `read_to_string`.

4. **`convert_all_formats` goes through a `serde_json::Value` intermediate for all conversions.** Lines 136-151: every format is deserialized to `serde_json::Value` first, then serialized to each target format. This means TOML-to-YAML goes through JSON in the middle, losing type information that TOML preserves (e.g., dates become strings). This is a known limitation of the serde-json-as-pivot approach, but it is not documented.

5. **TOML and TOON serialization failures are silently converted to comment strings.** Lines 158-166: if `toml::to_string_pretty` fails, the output is `"# TOML conversion not supported for this structure"`. Same for TOON. The user sees a comment where they expected data. This should be an explicit error or at minimum a separate field indicating failure.

6. **`is_data_file` is a trivial wrapper.** Line 189-191: `pub fn is_data_file(path: &str) -> Option<String> { detect_data_format(path) }`. This function adds zero value over calling `detect_data_format` directly. It exists solely because the Tauri command uses this name.

7. **Binary extension list is incomplete and hardcoded.** Lines 108-109: the list includes common binaries but misses many formats (.mp3, .mp4, .mov, .avi, .ttf, .otf, .sqlite, .db, .class, .o, .a, .lib, etc.). A better approach: attempt to read as UTF-8 and check for decoding errors, or use a magic bytes check.

### ratatoskr_core

**File:** `core/ratatoskr_core/src/lib.rs` (791 lines)
**Cargo.toml:** serde, serde_json

**Findings:**

1. **This is the largest crate and the most complex, with zero tests.** The JSON-LD parser handles embedded graphs, reference resolution with cycle detection, merge configuration, ID prefixing, deduplication, and stylesheet application. All of this is untested.

2. **`generate_sample_graph` is 70+ lines of hardcoded data.** Lines 718-790. This belongs in a test fixture or a data file, not in the core library. It is dead code for production use -- it only serves the UI demo. It clutters the API surface.

3. **`load_jsonld_with_refs` does recursive file I/O with no depth limit.** Lines 520-642: the function follows `@id` references to other JSON-LD files on disk, recursively. Cycle detection via `visited` prevents infinite loops on the same file, but a chain of 1000 unique files would blow the stack. There is no depth limit.

4. **`resolve_reference` has a hardcoded `kerak:patterns/` prefix.** Lines 361-367: this couples the generic JSON-LD loader to a specific project's naming convention. If the purpose of `ratatoskr_core` is to be a general graph viewer, this domain-specific reference resolution does not belong here.

5. **`parse_embedded_graph` silently drops nodes without `@id`.** Line 147: `if !id.is_empty()`. Nodes without an `@id` are silently excluded from the graph. No warning, no count of dropped nodes. The caller has no way to know data was lost.

6. **`save_graph` does not validate the path.** Line 677-685: it writes to any path provided. No directory existence check, no permission check, no extension validation. Combined with the Tauri command `rata_save_graph`, this allows the frontend to write arbitrary JSON files anywhere on the filesystem.

7. **Graph density calculation assumes directed graph.** Line 703-704: `edge_count / (node_count * (node_count - 1))`. This is the density formula for a directed graph. If the graph is undirected, the denominator should be `node_count * (node_count - 1) / 2`. There is no indication in the data model whether edges are directed.

### Tauri Shells (all 5)

**Findings common to all shells:**

1. **S3 above:** `.expect()` panic on startup.

2. **All standalone shells include `#[cfg_attr(mobile, tauri::mobile_entry_point)]`.** This is dead configuration on a macOS-only desktop platform. It does not cause harm but it is misleading -- there is no mobile target for these apps.

3. **The `windows_subsystem = "windows"` comment in main.rs says "DO NOT REMOVE!!"** But the project only targets macOS (`.cargo/config.toml` specifies `aarch64-apple-darwin`). The attribute and the comment are dead code.

**Hlidskjalf shell (`hlidskjalf/src-tauri/src/lib.rs`):**

4. The channel relay (lines 7-11) spawns a task that loops forever. If `app.emit` fails, the error is discarded. If the channel closes, the task exits silently. There is no cleanup.

**Svalinn shell (`svalinn/src-tauri/src/lib.rs`):**

5. `use svalinn_core::*` (line 1) is a glob import. This brings every public symbol from svalinn_core into scope. This is fragile -- adding a new public type to svalinn_core could cause name collisions.

**Kvasir shell (`kvasir/src-tauri/src/lib.rs`):**

6. Same glob import: `use kvasir_core::*` (line 1).

**Ratatoskr shell (`ratatoskr/src-tauri/src/lib.rs`):**

7. Same glob import: `use ratatoskr_core::*` (line 1).

8. `get_graph_stats` takes ownership of the entire `GraphData` by value (line 14: `fn get_graph_stats(graph: GraphData)`). This means the frontend must send the entire graph structure to the backend just to get a stat summary. For large graphs, this is a round-trip of the entire dataset. The function in the core crate takes `&GraphData` (a reference), but the Tauri command signature forces deserialization into an owned value.

**Yggdrasil shell (`yggdrasil/src-tauri/src/lib.rs`):**

9. This file is 134 lines of pure boilerplate. Every function is a one-line delegation to the corresponding core crate function with a prefixed name. There is no way to generate these wrappers automatically in the current architecture, but it is worth noting that adding a new command to any core crate requires:
   - Adding the function to the core crate
   - Adding a wrapper to the standalone Tauri shell
   - Adding a prefixed wrapper to the Yggdrasil shell
   - Registering it in the `generate_handler!` macro in both shells
   - Adding the mapping to the frontend `commands` prop

   That is 5 places to update for one new command. Forgetting any one of them results in a runtime error, not a compile error.

10. The Yggdrasil shell does not re-export `open_in_editor` for Hlidskjalf. Svalinn and Kvasir both expose `open_in_editor` commands (`sval_open_in_editor`, `kvas_open_in_editor`), but there is no `hlid_open_in_editor`. This is either intentional (Hlidskjalf does not need it) or an oversight.

---

## Dependency Hygiene

### Workspace-level Cargo.toml

| Dependency | Used by | Note |
|---|---|---|
| `serde` | All core crates + shells (via core) | Correct |
| `serde_json` | All core crates | Correct |
| `serde_yaml` | kvasir_core only | Could be non-workspace |
| `toml` | kvasir_core only | Could be non-workspace |
| `serde_toon2` | kvasir_core only | Could be non-workspace |
| `glob` | svalinn_core only | Could be non-workspace |
| `dirs` | svalinn_core only | Could be non-workspace |
| `tokio` | hlidskjalf_core + hlidskjalf shell + yggdrasil shell | Correct |
| `tauri` | 5 Tauri shells | Correct |
| `tauri-build` | 5 Tauri shells | Correct |
| `tauri-plugin-opener` | 5 Tauri shells | Correct |
| `tauri-plugin-dialog` | 4 Tauri shells (not hlidskjalf) | Correct |

Several dependencies are workspace-level but only used by one crate. This is not wrong, but it inflates the workspace manifest with single-consumer dependencies.

### Missing dependency: `hlidskjalf_core` does not depend on `yggdrasil_shared`

Both `svalinn_core` and `kvasir_core` depend on `yggdrasil_shared` for `open_in_editor`. `hlidskjalf_core` does not. The `Datagram` type that should live in `yggdrasil_shared` is instead local to `hlidskjalf_core`.

### Core crate purity: PASS

No core crate depends on `tauri`, `tauri-build`, or any Tauri plugin. The core/shell separation is clean at the dependency level.

---

## Recommendations (prioritized)

### 1. Add tests. Now.

Start with `hlidskjalf_core::parse_event` -- it has two code paths and a subtle precedence issue. Then `kvasir_core::convert_all_formats` -- it does round-trip conversion that can lose data. Then `ratatoskr_core::load_graph` and `jsonld_to_graph` -- 400+ lines of untested graph logic.

Minimum viable test coverage: one test per public function per crate. This is not aspirational -- it is the floor.

### 2. Replace stringly-typed Datagram fields with enums

Define `DatagramType` and `Priority` enums in `yggdrasil_shared`. Derive `Serialize`/`Deserialize` on them. Use `#[serde(rename_all = "lowercase")]`. Move `Datagram` to `yggdrasil_shared`. This gives compile-time enforcement of the schema's enum values.

### 3. Move `Datagram` and shared directory listing logic to `yggdrasil_shared`

`yggdrasil_shared` should contain:
- `Datagram`, `DatagramType`, `Priority` types
- A shared `FileTreeEntry` base type or trait
- The dir-listing / dotfile-skipping / dirs-first-sorting logic
- `open_in_editor`

### 4. Add input validation to `open_in_editor` and `save_graph`

At minimum: validate that paths are within expected directories. For `open_in_editor`, check that the file exists. For `save_graph`, validate the output directory exists and the extension is appropriate.

### 5. Make hardcoded paths configurable

- Socket path (`/tmp/hlidskjalf.sock`) should use `dirs::runtime_dir()` or `$XDG_RUNTIME_DIR` or at minimum `$TMPDIR`.
- Saga binary path should be configurable via environment variable.
- Editor in `open_in_editor` should respect `$EDITOR` or `$VISUAL`.

### 6. Fix error handling in `hlidskjalf_core` logging

Replace `let _ =` with logging. If you cannot log to the log file, at minimum use `eprintln!`. The current code silently loses events and log data.

### 7. Add size limits to file reading

`kvasir_core::read_file` should check file size before reading. A reasonable limit (e.g., 10MB) prevents the application from freezing when a user navigates to a large file.

### 8. Remove `generate_sample_graph` from the core crate

Move it to a test fixture or a separate demo module that is not part of the production API surface.

### 9. Document the token counting approximation

Either replace `count_tokens` with a proper implementation, or rename it to `estimate_byte_token_count` and document that it divides byte length by 4.

### 10. Add a shutdown mechanism to hlidskjalf_core

Use a `tokio::sync::CancellationToken` or a `watch` channel to allow graceful shutdown of the socket listener and lockfile monitor.
