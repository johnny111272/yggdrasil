# JSONL Viewer — Executable Plan

## Problem

JSONL (JSON Lines) is everywhere — conversation transcripts, log streams, datagram archives, audit trails. Kvasir handles JSON/YAML/TOML/TOON but treats JSONL as plaintext. A 50,000-line conversation transcript opens as a wall of unformatted text. Useless.

## Design: Single-Entry Record Browser

Not a list. Not a table. One record at a time, keyboard-navigated.

```
┌──────────────────────────────────────────────────────┐
│  transcript.jsonl          47 / 1,203       [Open ▸] │
│  ◀  ▲  ▼  ▶               [JSON] YAML TOML TOON     │
├──────────────────────────────────────────────────────┤
│  {                                                    │
│    "timestamp": 1741523400.0,                         │
│    "source": "bifrost",                               │
│    "type": "exchange",                                │
│    "priority": "normal",                              │
│    ...                                                │
│  }                                                    │
└──────────────────────────────────────────────────────┘
```

- Opens to the **last entry** (most recent = most relevant)
- Position indicator: `47 / 1,203`
- **Up/Down arrows** = previous/next entry
- **Left/Right arrows** = first/last entry (jump to boundaries)
- **Format picker** = view current entry as JSON, YAML, TOML, TOON
- Syntax highlighting via existing hljs infrastructure
- **Open in Editor** = export current entry in selected format to temp file, open in Zed

---

## Rust: kvasir_core

### New data structures

```rust
#[derive(Debug, Clone, Serialize)]
pub struct JsonlInfo {
    pub path: String,
    pub entry_count: usize,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonlEntry {
    pub index: usize,
    pub content: String,        // raw JSON string of this entry
    pub entry_count: usize,     // total entries (avoids separate call)
}
```

### New functions

**`detect_data_format`** — already returns `Option<String>`. Add `"jsonl"` detection:

```rust
"jsonl" => Some("jsonl".to_string()),
```

This is the branch point — when the frontend gets `"jsonl"` back, it enters JSONL mode instead of the standard data view.

**`read_jsonl_info(path: &str) -> Result<JsonlInfo, String>`**

Counts lines in the file without loading everything into memory. Streams through the file counting newlines. Returns path, entry count, file size.

```rust
pub fn read_jsonl_info(path: &str) -> Result<JsonlInfo, String> {
    let file_path = Path::new(path);
    if !file_path.is_file() {
        return Err(format!("Not a file: {}", path));
    }
    let metadata = std::fs::metadata(file_path)
        .map_err(|e| format!("Failed to get metadata: {}", e))?;

    let file = std::fs::File::open(file_path)
        .map_err(|e| format!("Failed to open: {}", e))?;
    let reader = std::io::BufReader::new(file);

    use std::io::BufRead;
    let entry_count = reader.lines().count();

    Ok(JsonlInfo {
        path: path.to_string(),
        entry_count,
        size_bytes: metadata.len(),
    })
}
```

**`read_jsonl_entry(path: &str, index: usize) -> Result<JsonlEntry, String>`**

Reads a single line by index. Streams to the line without loading the full file. Validates that the line is valid JSON (parse and re-serialize pretty-printed).

```rust
pub fn read_jsonl_entry(path: &str, index: usize) -> Result<JsonlEntry, String> {
    let file_path = Path::new(path);
    let file = std::fs::File::open(file_path)
        .map_err(|e| format!("Failed to open: {}", e))?;
    let reader = std::io::BufReader::new(file);

    use std::io::BufRead;
    let mut entry_count = 0;
    let mut target_line = None;

    for (i, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("Read error at line {}: {}", i, e))?;
        entry_count = i + 1;
        if i == index {
            target_line = Some(line);
        }
    }

    let raw = target_line
        .ok_or_else(|| format!("Index {} out of range (file has {} entries)", index, entry_count))?;

    // Parse and pretty-print
    let value: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("Invalid JSON at line {}: {}", index, e))?;
    let content = serde_json::to_string_pretty(&value)
        .map_err(|e| format!("Serialization error: {}", e))?;

    Ok(JsonlEntry {
        index,
        content,
        entry_count,
    })
}
```

Note: this reads through the file on every call. For a 50K-line file this is ~milliseconds (it's sequential I/O, no parsing until the target line). If profiling shows this matters, we can add a line-offset index cache later. Don't optimize prematurely.

**`export_entry_as(content: &str, format: &str, source_name: &str, index: usize) -> Result<String, String>`**

Converts JSON content to the target format, writes to a temp file, returns the temp file path. Does NOT open in editor — the frontend calls `open_in_editor` separately with the returned path.

```rust
pub fn export_entry_as(
    content: &str,
    format: &str,
    source_name: &str,
    index: usize,
) -> Result<String, String> {
    let converted = if format == "json" {
        content.to_string()
    } else {
        let all = convert_to_all_formats(content, "json")?;
        match format {
            "yaml" => all.yaml.content,
            "toml" => all.toml.content,
            "toon" => all.toon.content,
            _ => return Err(format!("Unknown format: {}", format)),
        }
    };

    let ext = format;
    let filename = format!("kvasir-{}-{}.{}", source_name, index, ext);
    let path = std::env::temp_dir().join(filename);

    std::fs::write(&path, &converted)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}
```

### Tauri command wrappers

In `kvasir/src-tauri/src/lib.rs`:

```rust
#[tauri::command]
fn read_jsonl_info(path: String) -> Result<kvasir_core::JsonlInfo, String> {
    kvasir_core::read_jsonl_info(&path)
}

#[tauri::command]
fn read_jsonl_entry(path: String, index: usize) -> Result<kvasir_core::JsonlEntry, String> {
    kvasir_core::read_jsonl_entry(&path, index)
}

#[tauri::command]
fn export_entry_as(content: String, format: String, source_name: String, index: usize) -> Result<String, String> {
    kvasir_core::export_entry_as(&content, &format, &source_name, index)
}
```

Register in `generate_handler![]`. Add to Yggdrasil with `kvas_` prefix.

### Commands map update

Standalone:
```ts
commands = {
  ...existing...,
  read_jsonl_info: "read_jsonl_info",
  read_jsonl_entry: "read_jsonl_entry",
  export_entry_as: "export_entry_as",
}
```

Yggdrasil:
```ts
commands={{
  ...existing...,
  read_jsonl_info: "kvas_read_jsonl_info",
  read_jsonl_entry: "kvas_read_jsonl_entry",
  export_entry_as: "kvas_export_entry_as",
}}
```

---

## Svelte: KvasirView changes

### Detection branch in `loadFile`

Currently `detect_data_format` returns `null` for `.jsonl`. After adding `"jsonl"` detection:

```ts
type ViewTab = "code" | "data" | "preview" | "inspect" | "jsonl";

let isJsonlFile = $state(false);
let jsonlInfo: JsonlInfo | null = $state(null);
let jsonlEntry: JsonlEntry | null = $state(null);
let jsonlFormat: DataFormat = $state("json");
```

In `loadFile`, after `detect_data_format`:

```ts
isJsonlFile = format === "jsonl";

if (isJsonlFile) {
  jsonlInfo = await invoke<JsonlInfo>(commands.read_jsonl_info, { path });
  if (jsonlInfo.entry_count > 0) {
    jsonlEntry = await invoke<JsonlEntry>(commands.read_jsonl_entry, {
      path,
      index: jsonlInfo.entry_count - 1,  // start at last entry
    });
  }
  jsonlFormat = "json";
  activeTab = "jsonl";
  return;  // skip standard data loading
}
```

### JSONL tab rendering

New tab button (alongside code/data/preview/inspect):

```svelte
{#if isJsonlFile}
  <button class="tab" class:active={activeTab === "jsonl"} onclick={() => activeTab = "jsonl"}>
    JSONL
  </button>
{/if}
```

### JSONL navigation controls

```svelte
{#if activeTab === "jsonl" && jsonlInfo && jsonlEntry}
  <section class="jsonl-controls">
    <div class="jsonl-nav">
      <button class="nav-btn" onclick={jsonlFirst} disabled={jsonlEntry.index === 0}
        title="First entry (Left arrow)">◀</button>
      <button class="nav-btn" onclick={jsonlPrev} disabled={jsonlEntry.index === 0}
        title="Previous entry (Up arrow)">▲</button>
      <button class="nav-btn" onclick={jsonlNext} disabled={jsonlEntry.index >= jsonlInfo.entry_count - 1}
        title="Next entry (Down arrow)">▼</button>
      <button class="nav-btn" onclick={jsonlLast} disabled={jsonlEntry.index >= jsonlInfo.entry_count - 1}
        title="Last entry (Right arrow)">▶</button>
      <span class="jsonl-position">
        {jsonlEntry.index + 1} / {jsonlInfo.entry_count.toLocaleString()}
      </span>
    </div>
    <div class="format-selector">
      {#each ["json", "yaml", "toml", "toon"] as fmt}
        <button
          class="format-btn"
          class:active={jsonlFormat === fmt}
          onclick={() => { jsonlFormat = fmt as DataFormat; convertJsonlEntry(); }}
        >
          {fmt.toUpperCase()}
        </button>
      {/each}
    </div>
    <Button variant="ghost" onclick={exportJsonlEntry}>Open in Editor</Button>
  </section>
{/if}
```

### Navigation functions

```ts
async function jsonlNavigate(index: number) {
  if (!selectedFile || !jsonlInfo) return;
  if (index < 0 || index >= jsonlInfo.entry_count) return;
  jsonlEntry = await invoke<JsonlEntry>(commands.read_jsonl_entry, {
    path: selectedFile,
    index,
  });
  jsonlFormat = "json";  // reset to JSON on navigation
  jsonlConverted = null;  // clear converted content
}

function jsonlFirst() { jsonlNavigate(0); }
function jsonlPrev() { if (jsonlEntry) jsonlNavigate(jsonlEntry.index - 1); }
function jsonlNext() { if (jsonlEntry) jsonlNavigate(jsonlEntry.index + 1); }
function jsonlLast() { if (jsonlInfo) jsonlNavigate(jsonlInfo.entry_count - 1); }
```

### Scrubber for large files

A range slider spanning the full entry count. Dragging updates the position indicator in real-time but doesn't fetch the entry on every tick — that would hammer the backend. Instead, debounce: only load the entry when the user pauses on a position for >1000ms.

```svelte
{#if jsonlInfo && jsonlInfo.entry_count > 1}
  <input
    type="range"
    class="jsonl-scrubber"
    min={0}
    max={jsonlInfo.entry_count - 1}
    value={scrubberIndex}
    oninput={handleScrub}
  />
{/if}
```

State and logic:

```ts
let scrubberIndex = $state(0);
let scrubTimer: ReturnType<typeof setTimeout> | null = null;

function handleScrub(e: Event) {
  const target = e.target as HTMLInputElement;
  scrubberIndex = parseInt(target.value);

  // Update position indicator immediately (cheap — no backend call)
  // But don't fetch the entry yet

  // Clear previous timer
  if (scrubTimer) clearTimeout(scrubTimer);

  // Fetch entry after 1000ms pause
  scrubTimer = setTimeout(() => {
    jsonlNavigate(scrubberIndex);
  }, 1000);
}
```

Keep `scrubberIndex` in sync with actual navigation:

```ts
async function jsonlNavigate(index: number) {
  // ...existing...
  scrubberIndex = index;  // sync scrubber position after fetch
}
```

The position indicator shows `scrubberIndex + 1` while scrubbing (instant feedback) but the code viewer only updates after the pause (debounced fetch). This gives the feel of scrubbing through a timeline — the number flies by, you stop, the content appears.

**CSS:** The scrubber spans the full width of the nav controls. Styled to match the app (dark track, accent thumb). On files with <20 entries, the scrubber is optional — arrow keys are sufficient. Could hide it below a threshold, but showing it always is simpler and consistent.

### Keyboard handling

Scoped to the JSONL view — only active when `activeTab === "jsonl"`:

```ts
function handleJsonlKeydown(e: KeyboardEvent) {
  if (activeTab !== "jsonl") return;
  switch (e.key) {
    case "ArrowUp": e.preventDefault(); jsonlPrev(); break;
    case "ArrowDown": e.preventDefault(); jsonlNext(); break;
    case "ArrowLeft": e.preventDefault(); jsonlFirst(); break;
    case "ArrowRight": e.preventDefault(); jsonlLast(); break;
  }
}
```

Attach in `onMount`:

```ts
onMount(() => {
  window.addEventListener("keydown", handleJsonlKeydown);
  return () => window.removeEventListener("keydown", handleJsonlKeydown);
});
```

### Format conversion for JSONL entries

When the user selects a non-JSON format, convert the current entry:

```ts
let jsonlConverted: AllFormats | null = $state(null);

async function convertJsonlEntry() {
  if (!jsonlEntry || jsonlFormat === "json") {
    jsonlConverted = null;
    return;
  }
  jsonlConverted = await invoke<AllFormats>(commands.convert_to_all_formats, {
    content: jsonlEntry.content,
    sourceFormat: "json",
  });
}
```

Display content derived:

```ts
let jsonlDisplayContent = $derived.by(() => {
  if (!jsonlEntry) return "";
  if (jsonlFormat === "json") return jsonlEntry.content;
  if (jsonlConverted) return jsonlConverted[jsonlFormat].content;
  return jsonlEntry.content;  // fallback while converting
});
```

### Export to editor

```ts
async function exportJsonlEntry() {
  if (!jsonlEntry || !selectedFile) return;
  const sourceName = selectedFile.split("/").pop()?.replace(".jsonl", "") || "entry";
  const content = jsonlFormat === "json"
    ? jsonlEntry.content
    : (jsonlConverted?.[jsonlFormat]?.content || jsonlEntry.content);
  const tempPath = await invoke<string>(commands.export_entry_as, {
    content: jsonlEntry.content,  // always send JSON, let Rust convert
    format: jsonlFormat,
    sourceName,
    index: jsonlEntry.index,
  });
  await invoke(commands.open_in_editor, { path: tempPath, line: 1 });
}
```

### JSONL entry display

Reuses the existing code viewer with syntax highlighting:

```svelte
{#if activeTab === "jsonl" && jsonlEntry}
  <section class="code-viewer">
    <pre><code>{#each jsonlDisplayContent.split('\n') as line, i}{@const highlighted = ...}<span class="line-number">{i + 1}</span><span class="line-content">{@html highlighted}</span>
{/each}</code></pre>
  </section>
{/if}
```

The highlighting logic is identical to the existing code viewer — just pointing at `jsonlDisplayContent` instead of `displayContent`. This should be extracted into a shared snippet or component to avoid duplication (the existing `highlightedContent` derived can be generalized).

---

## Cross-App Navigation (FR-1 synergy)

When the `openFile` prop includes a `.jsonl` path with a line number:

```ts
openFile?: string | null;
openFileLine?: number | null;  // entry index for JSONL files
```

In the `$effect` that reacts to `openFile`:

```ts
$effect(() => {
  if (openFile) {
    loadFile(openFile);
    // If a line/entry was specified and this is a JSONL file, navigate to it
    if (openFileLine != null && isJsonlFile) {
      jsonlNavigate(openFileLine);
    }
  }
});
```

This enables the datagram-to-Kvasir flow: click a JSONL reference in Hlidskjalf → Kvasir opens the file at the specific entry.

---

## File Changes Summary

| File | Change |
|------|--------|
| `core/kvasir_core/src/lib.rs` | Add `JsonlInfo`, `JsonlEntry` structs. Add `read_jsonl_info`, `read_jsonl_entry`, `export_entry_as` functions. Add `"jsonl"` to `detect_data_format`. |
| `kvasir/src-tauri/src/lib.rs` | Add 3 new Tauri commands, register in `generate_handler![]` |
| `yggdrasil/src-tauri/src/lib.rs` | Add 3 prefixed commands (`kvas_read_jsonl_info`, etc.), register |
| `kvasir/src/lib/KvasirView.svelte` | Add `"jsonl"` to ViewTab. JSONL state variables. Detection in `loadFile`. Nav functions. Keyboard handler. JSONL controls + display in template. |
| `yggdrasil/src/routes/+page.svelte` | Add new commands to KvasirView command map |

---

## Verification

1. `cargo check` — workspace compiles with new structs and functions
2. Open a `.jsonl` file in Kvasir → should show last entry in JSON, position indicator shows `N / N`
3. Up/Down arrows navigate entries, position updates
4. Left/Right arrows jump to first/last
5. Format selector switches between JSON/YAML/TOML/TOON for current entry
6. "Open in Editor" creates temp file in selected format, opens in Zed
7. Open same `.jsonl` in Yggdrasil Kvasir tab — same behavior
8. Large file (50K+ lines) — navigation stays responsive (no full-file parse per entry)
9. Invalid JSON line — shows error, doesn't crash
10. Empty `.jsonl` file — shows empty state, nav disabled
