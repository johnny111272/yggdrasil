<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Button } from "@yggdrasil/ui";
  import hljs from "highlight.js";
  import type { AllFormats, DataFormat, JsonlInfo, JsonlEntry, WrapMode } from "./kvasir-types";

  let {
    commands,
    path,
    wrapMode = "nowrap",
    getHljsLanguage,
    refreshKey = 0,
  }: {
    commands: {
      read_jsonl_info: string;
      read_jsonl_entry: string;
      export_entry_as: string;
      convert_to_all_formats: string;
      open_in_editor: string;
    };
    path: string;
    wrapMode?: WrapMode;
    getHljsLanguage: (lang: string) => string;
    refreshKey?: number;
  } = $props();

  let jsonlInfo: JsonlInfo | null = $state(null);
  let jsonlEntry: JsonlEntry | null = $state(null);
  let jsonlFormat: DataFormat = $state("json");
  let jsonlConverted: AllFormats | null = $state(null);
  let scrubberIndex = $state(0);
  let scrubTimer: ReturnType<typeof setTimeout> | null = null;

  // Load info + last entry when path changes
  $effect(() => {
    if (path) loadJsonl(path);
  });

  async function loadJsonl(filePath: string) {
    jsonlInfo = await invoke<JsonlInfo>(commands.read_jsonl_info, { path: filePath });
    if (jsonlInfo.entry_count > 0) {
      jsonlEntry = await invoke<JsonlEntry>(commands.read_jsonl_entry, {
        path: filePath,
        index: jsonlInfo.entry_count - 1,
      });
      scrubberIndex = jsonlInfo.entry_count - 1;
    } else {
      jsonlEntry = null;
      scrubberIndex = 0;
    }
    jsonlFormat = "json";
    jsonlConverted = null;
  }

  async function reloadCurrent() {
    if (!path) return;
    jsonlInfo = await invoke<JsonlInfo>(commands.read_jsonl_info, { path });
    if (jsonlInfo.entry_count === 0) {
      jsonlEntry = null;
      scrubberIndex = 0;
      return;
    }
    const idx = Math.min(scrubberIndex, jsonlInfo.entry_count - 1);
    jsonlEntry = await invoke<JsonlEntry>(commands.read_jsonl_entry, { path, index: idx });
    scrubberIndex = idx;
    if (jsonlFormat !== "json") {
      await convertEntry();
    } else {
      jsonlConverted = null;
    }
  }

  $effect(() => {
    void refreshKey;
    if (refreshKey > 0) reloadCurrent();
  });

  async function navigate(index: number) {
    if (!path || !jsonlInfo) return;
    if (index < 0 || index >= jsonlInfo.entry_count) return;
    jsonlEntry = await invoke<JsonlEntry>(commands.read_jsonl_entry, {
      path,
      index,
    });
    scrubberIndex = index;
    jsonlConverted = null;
    if (jsonlFormat !== "json") {
      await convertEntry();
    }
  }

  function first() { navigate(0); }
  function prev() { if (jsonlEntry) navigate(jsonlEntry.index - 1); }
  function next() { if (jsonlEntry) navigate(jsonlEntry.index + 1); }
  function last() { if (jsonlInfo) navigate(jsonlInfo.entry_count - 1); }

  function handleScrub(event: Event) {
    const target = event.target as HTMLInputElement;
    scrubberIndex = parseInt(target.value);
    if (scrubTimer) clearTimeout(scrubTimer);
    scrubTimer = setTimeout(() => {
      navigate(scrubberIndex);
    }, 1000);
  }

  async function convertEntry() {
    if (!jsonlEntry || jsonlFormat === "json") {
      jsonlConverted = null;
      return;
    }
    jsonlConverted = await invoke<AllFormats>(commands.convert_to_all_formats, {
      content: jsonlEntry.content,
      sourceFormat: "json",
    });
  }

  async function exportEntry() {
    if (!jsonlEntry || !path) return;
    const sourceName = path.split("/").pop()?.replace(".jsonl", "") || "entry";
    const tempPath = await invoke<string>(commands.export_entry_as, {
      content: jsonlEntry.content,
      format: jsonlFormat,
      sourceName,
      index: jsonlEntry.index,
    });
    await invoke(commands.open_in_editor, { path: tempPath, line: 1 });
  }

  let displayContent = $derived.by(() => {
    if (!jsonlEntry) return "";
    if (jsonlFormat === "json") return jsonlEntry.content;
    if (jsonlConverted) {
      const fmt = jsonlConverted[jsonlFormat as keyof Pick<AllFormats, "json" | "yaml" | "toml" | "toon">];
      return fmt?.content || jsonlEntry.content;
    }
    return jsonlEntry.content;
  });

  let highlighted = $derived.by(() => {
    if (!displayContent) return "";
    const lang = jsonlFormat === "toon" ? "yaml" : jsonlFormat;
    const hljsLang = getHljsLanguage(lang);
    try {
      return hljs.highlight(displayContent, { language: hljsLang }).value;
    } catch {
      return hljs.highlightAuto(displayContent).value;
    }
  });

  function handleKeydown(event: KeyboardEvent) {
    switch (event.key) {
      case "ArrowUp": event.preventDefault(); prev(); break;
      case "ArrowDown": event.preventDefault(); next(); break;
      case "ArrowLeft": event.preventDefault(); first(); break;
      case "ArrowRight": event.preventDefault(); last(); break;
    }
  }

  $effect(() => {
    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });
</script>

{#if jsonlInfo}
  <section class="jsonl-controls">
    <div class="jsonl-nav">
      <button class="nav-btn" onclick={first} disabled={!jsonlEntry || jsonlEntry.index === 0}
        title="First entry (Left arrow)">&#9664;</button>
      <button class="nav-btn" onclick={prev} disabled={!jsonlEntry || jsonlEntry.index === 0}
        title="Previous entry (Up arrow)">&#9650;</button>
      <button class="nav-btn" onclick={next} disabled={!jsonlEntry || jsonlEntry.index >= jsonlInfo.entry_count - 1}
        title="Next entry (Down arrow)">&#9660;</button>
      <button class="nav-btn" onclick={last} disabled={!jsonlEntry || jsonlEntry.index >= jsonlInfo.entry_count - 1}
        title="Last entry (Right arrow)">&#9654;</button>
      <span class="jsonl-position">
        {scrubberIndex + 1} / {jsonlInfo.entry_count.toLocaleString()}
      </span>
    </div>
    {#if jsonlInfo.entry_count > 1}
      <input
        type="range"
        class="jsonl-scrubber"
        min={0}
        max={jsonlInfo.entry_count - 1}
        value={scrubberIndex}
        oninput={handleScrub}
      />
    {/if}
    <div class="jsonl-right">
      <div class="format-selector">
        {#each ["json", "yaml", "toml", "toon", "ron"] as fmt}
          <button
            class="format-btn"
            class:active={jsonlFormat === fmt}
            onclick={() => { jsonlFormat = fmt as DataFormat; convertEntry(); }}
          >
            {fmt.toUpperCase()}
          </button>
        {/each}
      </div>
      <Button variant="ghost" onclick={exportEntry}>Open in Editor</Button>
    </div>
  </section>

  {#if jsonlEntry}
    <section class="code-viewer" class:wrap79={wrapMode === "wrap79"} class:wrapwidth={wrapMode === "wrapwidth"}>
      <pre><code>{#each displayContent.split('\n') as line, i}{@const hl = highlighted.split('\n')[i] || ''}<span class="line-number">{i + 1}</span><span class="line-content">{@html hl}</span>
{/each}</code></pre>
    </section>
  {:else}
    <section class="empty-state">
      <p>Empty JSONL file</p>
    </section>
  {/if}
{/if}

<style>
  .jsonl-controls {
    background: var(--bg-secondary);
    padding: var(--space-lg);
    border-radius: var(--radius-md);
    margin-bottom: var(--space-lg);
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .jsonl-nav {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .nav-btn {
    padding: var(--space-xs) var(--space-md);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--action-neutral);
    color: var(--text-primary);
    cursor: pointer;
    font-size: var(--text-sm);
    line-height: 1;
  }

  .nav-btn:hover:not(:disabled) {
    background: var(--action-neutral-hover);
  }

  .nav-btn:disabled {
    opacity: 0.3;
    cursor: default;
  }

  .jsonl-position {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    color: var(--text-secondary);
    margin-left: var(--space-md);
    font-variant-numeric: tabular-nums;
  }

  .jsonl-scrubber {
    width: 100%;
    height: 4px;
    appearance: none;
    background: var(--bg-tertiary);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
  }

  .jsonl-scrubber::-webkit-slider-thumb {
    appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--action-primary);
    cursor: pointer;
  }

  .jsonl-right {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-lg);
  }

  .format-selector {
    display: flex;
    gap: var(--space-sm);
  }

  .format-btn {
    padding: var(--space-sm) var(--space-lg);
    border: none;
    border-radius: var(--radius-sm);
    background: var(--action-neutral);
    color: var(--text-primary);
    cursor: pointer;
    font-size: var(--text-sm);
    font-weight: 500;
  }

  .format-btn:hover {
    background: var(--action-neutral-hover);
  }

  .format-btn.active {
    background: var(--action-primary);
  }

  .code-viewer {
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .code-viewer pre {
    margin: 0;
    padding: var(--space-xl);
    overflow-x: auto;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    line-height: 1.6;
  }

  .code-viewer code {
    display: block;
  }

  .line-number {
    display: inline-block;
    width: 4rem;
    color: var(--text-secondary);
    text-align: right;
    padding-right: var(--space-xl);
    user-select: none;
  }

  .line-content {
    white-space: pre;
  }

  .code-viewer.wrap79 .line-content,
  .code-viewer.wrapwidth .line-content {
    white-space: pre-wrap;
    word-break: break-word;
    display: inline-block;
    vertical-align: top;
  }

  .code-viewer.wrap79 .line-content {
    max-width: 79ch;
  }

  .empty-state {
    text-align: center;
    padding: 4rem var(--space-3xl);
    color: var(--text-secondary);
  }
</style>
