<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { Button, SidebarLayout, TreeNode } from "@yggdrasil/ui";
  import hljs from "highlight.js";
  import "highlight.js/styles/github-dark.css";
  import { marked } from "marked";
  import MarkdownPreview from "./MarkdownPreview.svelte";
  import SchemaInspector from "./SchemaInspector.svelte";
  import JsonlViewer from "./JsonlViewer.svelte";
  import FormatControls from "./FormatControls.svelte";
  import { analyzeSchema, type InspectedSchema } from "./schema-inspect";
  import type { FileTreeEntry, FileContent, AllFormats, KvasTreeNode, ViewTab, DataFormat, WrapMode } from "./kvasir-types";

  let {
    commands = {
      list_directory: "list_directory",
      read_file: "read_file",
      open_in_editor: "open_in_editor",
      convert_to_all_formats: "convert_to_all_formats",
      detect_data_format: "detect_data_format",
      read_jsonl_info: "read_jsonl_info",
      read_jsonl_entry: "read_jsonl_entry",
      export_entry_as: "export_entry_as",
    },
    openFile = null,
  }: {
    commands?: {
      list_directory: string;
      read_file: string;
      open_in_editor: string;
      convert_to_all_formats: string;
      detect_data_format: string;
      read_jsonl_info: string;
      read_jsonl_entry: string;
      export_entry_as: string;
    };
    openFile?: string | null;
  } = $props();

  // ── State ──────────────────────────────────────────────────────────────

  let directory = $state("");
  let treeRoot: KvasTreeNode | null = $state(null);
  let selectedFile: string | null = $state(null);
  let fileContent: FileContent | null = $state(null);
  let loading = $state(false);
  let error = $state("");
  let showTree = $derived(treeRoot !== null);
  let activeTab: ViewTab = $state("code");
  let dataFormats: AllFormats | null = $state(null);
  let selectedFormat: DataFormat = $state("json");
  let isDataFile = $state(false);
  let isMarkdownFile = $state(false);
  let isSchemaFile = $state(false);
  let isJsonlFile = $state(false);
  let inspectedSchema: InspectedSchema | null = $state(null);
  let showHidden = $state(false);
  let wrapMode: WrapMode = $state("nowrap");

  function cycleWrap() {
    if (wrapMode === "nowrap") wrapMode = "wrap79";
    else if (wrapMode === "wrap79") wrapMode = "wrapwidth";
    else wrapMode = "nowrap";
  }

  function wrapLabel(): string {
    if (wrapMode === "nowrap") return "no wrap";
    if (wrapMode === "wrap79") return "wrap 79";
    return "wrap fit";
  }

  // ── Tree operations ────────────────────────────────────────────────────

  async function selectDirectory() {
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (selected && typeof selected === "string") {
      directory = selected;
      await loadTree();
    }
  }

  async function loadTree() {
    if (!directory) return;
    loading = true;
    error = "";
    try {
      const entries = await invoke<FileTreeEntry[]>(commands.list_directory, { directory, showHidden });
      treeRoot = {
        name: directory.split("/").pop() || directory,
        path: directory,
        is_dir: true,
        extension: null,
        size_bytes: 0,
        expanded: true,
        children: entries.map(e => ({
          ...e,
          expanded: false,
          children: [],
          loading: false,
        })),
        loading: false,
      };
    } catch (e) {
      error = String(e);
    }
    loading = false;
  }

  function findNode(root: KvasTreeNode, path: string): KvasTreeNode | null {
    if (root.path === path) return root;
    for (const child of root.children) {
      const found = findNode(child, path);
      if (found) return found;
    }
    return null;
  }

  async function handleTreeToggle(path: string) {
    if (!treeRoot) return;
    const node = findNode(treeRoot, path);
    if (!node) return;

    if (node.expanded) {
      node.expanded = false;
    } else {
      if (node.children.length === 0) {
        node.loading = true;
        try {
          const entries = await invoke<FileTreeEntry[]>(commands.list_directory, { directory: node.path, showHidden });
          node.children = entries.map(e => ({
            ...e,
            expanded: false,
            children: [],
            loading: false,
          }));
        } catch (e) {
          console.error("Failed to load children:", e);
        }
        node.loading = false;
      }
      node.expanded = true;
    }
    treeRoot = treeRoot;
  }

  async function handleTreeSelect(path: string) {
    await loadFile(path);
  }

  // ── File loading ───────────────────────────────────────────────────────

  async function loadFile(path: string) {
    selectedFile = path;
    error = "";
    dataFormats = null;

    try {
      // Detect format first — JSONL skips read_file entirely
      const format = await invoke<string | null>(commands.detect_data_format, { path });

      isJsonlFile = format === "jsonl";
      if (isJsonlFile) {
        fileContent = null;
        isDataFile = false;
        isMarkdownFile = false;
        isSchemaFile = false;
        inspectedSchema = null;
        activeTab = "jsonl";
        return;
      }

      fileContent = await invoke<FileContent>(commands.read_file, { path });

      isMarkdownFile = fileContent.language === "markdown";

      isSchemaFile = path.endsWith(".schema.json");
      inspectedSchema = null;
      if (isSchemaFile && fileContent) {
        try {
          inspectedSchema = analyzeSchema(fileContent.content);
        } catch (e) {
          console.error("Schema inspection failed:", e);
        }
      }

      isDataFile = format !== null;

      if (format && fileContent) {
        selectedFormat = format as DataFormat;
        try {
          dataFormats = await invoke<AllFormats>(commands.convert_to_all_formats, {
            content: fileContent.content,
            sourceFormat: format,
          });
        } catch (e) {
          console.error("Format conversion failed:", e);
        }
      }

      if (isSchemaFile && inspectedSchema) {
        activeTab = "inspect";
      } else if (isMarkdownFile) {
        activeTab = "preview";
      } else {
        activeTab = "code";
      }
    } catch (e) {
      error = String(e);
      fileContent = null;
    }
  }

  $effect(() => {
    if (openFile) {
      loadFile(openFile);
    }
  });

  // ── Helpers ────────────────────────────────────────────────────────────

  async function openInEditor(line: number = 1) {
    if (!selectedFile) return;
    try {
      await invoke(commands.open_in_editor, { path: selectedFile, line });
    } catch (e) {
      console.error("Failed to open in editor:", e);
    }
  }

  function relativePath(fullPath: string): string {
    if (directory && fullPath.startsWith(directory)) {
      return fullPath.slice(directory.length + 1);
    }
    return fullPath;
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function getFileIcon(path: string): string {
    const ext = path.split(".").pop()?.toLowerCase();
    switch (ext) {
      case "py": return "\uD83D\uDC0D";
      case "rs": return "\uD83E\uDD80";
      case "js":
      case "ts":
      case "jsx":
      case "tsx": return "\uD83D\uDCDC";
      case "svelte": return "\uD83D\uDD36";
      case "json":
      case "jsonld":
      case "yaml":
      case "yml":
      case "toml":
      case "toon":
      case "qa": return "\uD83D\uDCCB";
      case "md": return "\uD83D\uDCDD";
      case "html":
      case "css": return "\uD83C\uDFA8";
      default: return "\uD83D\uDCC4";
    }
  }

  function getHljsLanguage(lang: string): string {
    const mapping: Record<string, string> = {
      "python": "python",
      "rust": "rust",
      "javascript": "javascript",
      "typescript": "typescript",
      "jsx": "javascript",
      "tsx": "typescript",
      "svelte": "xml",
      "html": "html",
      "css": "css",
      "scss": "scss",
      "json": "json",
      "yaml": "yaml",
      "toml": "ini",
      "markdown": "markdown",
      "sql": "sql",
      "bash": "bash",
      "c": "c",
      "cpp": "cpp",
      "go": "go",
      "java": "java",
      "ruby": "ruby",
      "php": "php",
      "swift": "swift",
      "kotlin": "kotlin",
      "scala": "scala",
      "xml": "xml",
      "dockerfile": "dockerfile",
      "makefile": "makefile",
    };
    return mapping[lang] || "plaintext";
  }

  // ── Derived state ──────────────────────────────────────────────────────

  let displayContent = $derived.by(() => {
    if (activeTab === "data" && dataFormats) {
      return dataFormats[selectedFormat].content;
    }
    return fileContent?.content || "";
  });

  let highlightedContent = $derived.by(() => {
    if (!displayContent) return "";
    const lang = activeTab === "data"
      ? (selectedFormat === "toon" ? "yaml" : selectedFormat)
      : (fileContent?.language || "plaintext");
    const hljsLang = getHljsLanguage(lang);

    try {
      const result = hljs.highlight(displayContent, { language: hljsLang });
      return result.value;
    } catch {
      return hljs.highlightAuto(displayContent).value;
    }
  });

  let renderedMarkdown = $derived.by(() => {
    if (!fileContent?.content || !isMarkdownFile) return "";
    return marked(fileContent.content) as string;
  });
</script>

<SidebarLayout showSidebar={showTree} sidebarTitle="Files">
  {#snippet headerExtra()}
    <label class="dotfile-toggle" title="Show dotfiles">
      <input type="checkbox" bind:checked={showHidden} onchange={() => loadTree()} />
      <span class="dotfile-label">.*</span>
    </label>
  {/snippet}

  {#snippet sidebar()}
    {#if treeRoot}
      <TreeNode
        node={treeRoot}
        selected={selectedFile}
        onToggle={handleTreeToggle}
        onSelect={handleTreeSelect}
        getIcon={getFileIcon}
      />
    {/if}
  {/snippet}

  <header>
    <h1><span class="app-name">KVASIR</span> <span class="separator">::</span> <span class="subtitle">Workspace Inspector</span></h1>
  </header>

    <section class="controls">
      <div class="directory-row">
        <Button onclick={selectDirectory}>Select Directory</Button>
        <input
          type="text"
          bind:value={directory}
          placeholder="Or paste path here..."
          class="directory-input"
          onkeydown={(e) => e.key === 'Enter' && loadTree()}
        />
        <Button variant="primary" onclick={loadTree} disabled={!directory}>
          {loading ? "Loading..." : "Refresh"}
        </Button>
      </div>
    </section>

    {#if error}
      <section class="error-banner">
        {error}
      </section>
    {/if}

    {#if isJsonlFile && selectedFile}
      <!-- JSONL: independent path, no fileContent needed -->
      <section class="file-info">
        <div class="file-path">
          <strong>{relativePath(selectedFile)}</strong>
        </div>
      </section>
      <section class="tabs">
        <button class="tab active">JSONL</button>
        <button
          class="tab wrap-toggle"
          class:active={wrapMode !== "nowrap"}
          onclick={cycleWrap}
          title="Cycle: no wrap → wrap 79 → wrap to width"
        >
          {wrapLabel()}
        </button>
      </section>
      <JsonlViewer
        commands={{
          read_jsonl_info: commands.read_jsonl_info,
          read_jsonl_entry: commands.read_jsonl_entry,
          export_entry_as: commands.export_entry_as,
          convert_to_all_formats: commands.convert_to_all_formats,
          open_in_editor: commands.open_in_editor,
        }}
        path={selectedFile}
        {wrapMode}
        {getHljsLanguage}
      />
    {:else if fileContent}
      <section class="file-info">
        <div class="file-path">
          <strong>{relativePath(fileContent.path)}</strong>
          <Button variant="ghost" onclick={() => openInEditor()}>Open in Editor</Button>
        </div>
        <div class="file-meta">
          <span class="meta-item">Language: <strong>{fileContent.language}</strong></span>
          <span class="meta-item">Lines: <strong>{fileContent.line_count}</strong></span>
          <span class="meta-item">Size: <strong>{formatBytes(fileContent.size_bytes)}</strong></span>
        </div>
      </section>

      <!-- Tabs -->
      <section class="tabs">
        <button
          class="tab"
          class:active={activeTab === "code"}
          onclick={() => activeTab = "code"}
        >
          Code
        </button>
        {#if isMarkdownFile}
          <button
            class="tab"
            class:active={activeTab === "preview"}
            onclick={() => activeTab = "preview"}
          >
            Preview
          </button>
        {/if}
        {#if isDataFile}
          <button
            class="tab"
            class:active={activeTab === "data"}
            onclick={() => activeTab = "data"}
          >
            Data
          </button>
        {/if}
        {#if isSchemaFile && inspectedSchema}
          <button
            class="tab"
            class:active={activeTab === "inspect"}
            onclick={() => activeTab = "inspect"}
          >
            Inspect
          </button>
        {/if}
        <button
          class="tab wrap-toggle"
          class:active={wrapMode !== "nowrap"}
          onclick={cycleWrap}
          title="Cycle: no wrap → wrap 79 → wrap to width"
        >
          {wrapLabel()}
        </button>
      </section>

      <!-- Data view format selector and token stats -->
      {#if activeTab === "data" && dataFormats}
        <FormatControls {dataFormats} bind:selectedFormat />
      {/if}

      <!-- Schema Inspector -->
      {#if activeTab === "inspect" && inspectedSchema}
        <section class="inspector-view">
          <SchemaInspector schema={inspectedSchema} />
        </section>
      <!-- Markdown Preview -->
      {:else if activeTab === "preview" && isMarkdownFile}
        <MarkdownPreview content={renderedMarkdown} />
      {:else}
        <section class="code-viewer" class:wrap79={wrapMode === "wrap79"} class:wrapwidth={wrapMode === "wrapwidth"}>
          <pre><code>{#each displayContent.split('\n') as line, i}{@const highlighted = highlightedContent.split('\n')[i] || ''}<span class="line-number">{i + 1}</span><span class="line-content">{@html highlighted}</span>
{/each}</code></pre>
        </section>
      {/if}
    {:else if !loading && directory}
      <section class="empty-state">
        <p>Select a file from the tree to view its contents</p>
      </section>
    {:else if !directory}
      <section class="empty-state">
        <p>Select a directory to browse files</p>
      </section>
    {/if}
</SidebarLayout>

<style>
  .dotfile-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    cursor: pointer;
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .dotfile-toggle:hover {
    color: var(--text-primary);
  }

  .dotfile-toggle input {
    cursor: pointer;
  }

  .dotfile-label {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  header {
    margin-bottom: var(--space-xl);
  }

  h1 {
    margin: 0;
    font-size: var(--text-lg);
    display: flex;
    align-items: baseline;
    gap: var(--space-md);
  }

  .app-name {
    font-weight: 800;
    letter-spacing: 0.12em;
    color: var(--text-primary);
  }

  .separator {
    color: var(--text-secondary);
    font-weight: 300;
    opacity: 0.5;
  }

  .subtitle {
    font-weight: 300;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    letter-spacing: 0.04em;
  }

  .controls {
    background: var(--bg-secondary);
    padding: var(--space-2xl);
    border-radius: var(--radius-md);
    margin-bottom: var(--space-2xl);
  }

  .directory-row {
    display: flex;
    gap: var(--space-md);
  }

  .directory-input {
    flex: 1;
    padding: var(--space-md) var(--space-xl);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family: var(--font-body);
  }

  .error-banner {
    background: var(--severity-error);
    color: var(--text-primary);
    padding: var(--space-lg) var(--space-xl);
    border-radius: var(--radius-sm);
    margin-bottom: var(--space-xl);
  }

  .file-info {
    background: var(--bg-secondary);
    padding: var(--space-xl);
    border-radius: var(--radius-md);
    margin-bottom: var(--space-xl);
  }

  .file-path {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-md);
    font-family: var(--font-mono);
  }

  .file-meta {
    display: flex;
    gap: var(--space-2xl);
    color: var(--text-secondary);
    font-size: var(--text-sm);
  }

  .meta-item strong {
    color: var(--text-primary);
  }

  .tabs {
    display: flex;
    gap: var(--space-sm);
    margin-bottom: var(--space-lg);
  }

  .tab {
    padding: var(--space-sm) var(--space-xl);
    border: none;
    border-radius: var(--radius-sm) var(--radius-sm) 0 0;
    background: var(--action-neutral);
    color: var(--text-primary);
    cursor: pointer;
    font-size: var(--text-sm);
  }

  .tab:hover {
    background: var(--action-neutral-hover);
  }

  .tab.active {
    background: var(--bg-secondary);
    border-bottom: 2px solid var(--action-primary);
  }

  .wrap-toggle {
    margin-left: auto;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
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

  .inspector-view {
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .empty-state {
    text-align: center;
    padding: 4rem var(--space-3xl);
    color: var(--text-secondary);
  }
</style>
