<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { homeDir } from "@tauri-apps/api/path";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount, untrack } from "svelte";
  import { Button, ContainerLayout, AppHeader, ErrorBanner, EmptyState } from "@yggdrasil/ui";
  import hljs from "highlight.js";
  import "highlight.js/styles/github-dark.css";
  import { marked } from "marked";
  import MarkdownPreview from "./MarkdownPreview.svelte";
  import SchemaInspector from "./SchemaInspector.svelte";
  import JsonlViewer from "./JsonlViewer.svelte";
  import TableViewer from "./TableViewer.svelte";
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
      read_table: "read_table",
      export_table_csv: "export_table_csv",
    },
    openFile = null,
    openLine = null,
    storagePrefix = "solo",
    appTabs,
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
      read_table: string;
      export_table_csv: string;
    };
    openFile?: string | null;
    openLine?: number | null;
    storagePrefix?: string;
    appTabs?: { (): any };
  } = $props();

  // ── State ──────────────────────────────────────────────────────────────

  let directory = $state("");
  let showHidden = $state(false);
  let treeRoot: KvasTreeNode | null = $state(null);
  let selectedFile: string | null = $state(null);
  let fileContent: FileContent | null = $state(null);
  let loading = $state(false);
  let error = $state("");
  let activeTab: ViewTab = $state("code");
  let dataFormats: AllFormats | null = $state(null);
  let selectedFormat: DataFormat = $state("json");
  let isDataFile = $state(false);
  let isMarkdownFile = $state(false);
  let isSchemaFile = $state(false);
  let isJsonlFile = $state(false);
  let isTableFile = $state(false);
  let inspectedSchema: InspectedSchema | null = $state(null);
  let wrapMode: WrapMode = $state("nowrap");
  let refreshKey = $state(0);

  function refresh() {
    if (!selectedFile) return;
    refreshKey++;
    if (!isJsonlFile && !isTableFile) loadFile(selectedFile);
  }

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

  // ── Mode bar ─────────────────────────────────────────────────────────

  let modeOptions = $derived.by((): { value: string; label: string; icon?: string }[] => {
    if (isJsonlFile) return [{ value: "jsonl", label: "JSONL", icon: "J" }];
    if (isTableFile) return [{ value: "table", label: "Table", icon: "T" }];
    if (!fileContent) return [];

    const opts: { value: string; label: string; icon?: string }[] = [{ value: "code", label: "Code", icon: "C" }];
    if (isMarkdownFile) opts.push({ value: "preview", label: "Preview", icon: "P" });
    if (isDataFile) opts.push({ value: "data", label: "Data", icon: "D" });
    if (isSchemaFile && inspectedSchema) opts.push({ value: "inspect", label: "Inspect", icon: "I" });
    return opts;
  });

  // Sync activeTab when mode bar changes
  function handleModeChange(value: string) {
    activeTab = value as ViewTab;
  }

  // ── Directory navigation ─────────────────────────────────────────────

  function parentDir(path: string): string | null {
    const i = path.lastIndexOf("/");
    return i > 0 ? path.substring(0, i) : null;
  }

  function zoomToDirectory(path: string) {
    directory = path;
    selectedFile = null;
    fileContent = null;
    dataFormats = null;
    isJsonlFile = false;
    isTableFile = false;
    loadTree();
  }

  function handleDirectoryChange(path: string) {
    zoomToDirectory(path);
  }

  async function handleSetCwd() {
    const selected = await open({ directory: true, multiple: false });
    if (selected && typeof selected === "string") {
      zoomToDirectory(selected);
    }
  }

  onMount(async () => {
    if (!openFile && !directory) {
      const home = await homeDir();
      directory = home.replace(/\/?$/, "/.ai");
      await loadTree();
    }
  });

  // ── Tree operations ────────────────────────────────────────────────────

  function getTreeNodes(): KvasTreeNode[] {
    return treeRoot ? treeRoot.children : [];
  }
  let treeNodes = $derived(getTreeNodes());

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

  // Reload tree when showHidden changes
  $effect(() => {
    // Read showHidden to track it
    const _ = showHidden;
    if (directory) loadTree();
  });

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
        } catch (err) {
          error = String(err);
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

  function resetFileFlags() {
    fileContent = null;
    dataFormats = null;
    isDataFile = false;
    isMarkdownFile = false;
    isSchemaFile = false;
    isJsonlFile = false;
    isTableFile = false;
    inspectedSchema = null;
  }

  async function loadRegularFile(path: string, format: string | null) {
    fileContent = await invoke<FileContent>(commands.read_file, { path });

    isMarkdownFile = fileContent.language === "markdown";
    isSchemaFile = path.endsWith(".schema.json");

    if (isSchemaFile) {
      try { inspectedSchema = analyzeSchema(fileContent.content); }
      catch { inspectedSchema = null; }
    }

    isDataFile = format !== null;
    if (format) {
      selectedFormat = format as DataFormat;
      try {
        dataFormats = await invoke<AllFormats>(commands.convert_to_all_formats, {
          content: fileContent.content,
          sourceFormat: format,
        });
      } catch { dataFormats = null; }
    }

    if (isSchemaFile && inspectedSchema) activeTab = "inspect";
    else if (isMarkdownFile) activeTab = "preview";
    else activeTab = "code";
  }

  async function loadFile(path: string) {
    selectedFile = path;
    error = "";
    resetFileFlags();

    try {
      const format = await invoke<string | null>(commands.detect_data_format, { path });

      if (format === "jsonl") {
        isJsonlFile = true;
        activeTab = "jsonl";
      } else if (format === "csv" || format === "tsv" || format === "parquet") {
        isTableFile = true;
        activeTab = "table";
      } else {
        await loadRegularFile(path, format);
      }
    } catch (err) {
      error = String(err);
    }
  }

  $effect(() => {
    if (openFile) {
      const dir = parentDir(openFile);
      const currentDir = untrack(() => directory);
      if (dir && dir !== currentDir) {
        directory = dir;
        loadTree();
      }
      loadFile(openFile).then(() => {
        if (openLine) {
          requestAnimationFrame(() => {
            const lineEl = document.querySelector(`[data-line="${openLine}"]`);
            if (lineEl) lineEl.scrollIntoView({ block: "center" });
          });
        }
      });
    }
  });

  // ── Helpers ────────────────────────────────────────────────────────────

  async function openInEditor(line: number = 1) {
    if (!selectedFile) return;
    await invoke(commands.open_in_editor, { path: selectedFile, line });
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
      "ron": "rust",
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
    const raw = fileContent.content;
    const fmMatch = raw.match(/^---\r?\n([\s\S]*?)\r?\n---\r?\n?/);
    if (fmMatch) {
      const frontmatter = "```yaml\n" + fmMatch[1].trim() + "\n```\n\n";
      return marked(frontmatter + raw.slice(fmMatch[0].length)) as string;
    }
    return marked(raw) as string;
  });
</script>

<ContainerLayout
  appName="kvasir"
  {storagePrefix}
  sidebarMode="tree"
  sidebarTitle="Files"
  bind:directory
  bind:showHidden
  {treeNodes}
  {selectedFile}
  onTreeToggle={handleTreeToggle}
  onTreeSelect={handleTreeSelect}
  onTreeDblClickDir={zoomToDirectory}
  onDirectoryChange={handleDirectoryChange}
  onSetCwd={handleSetCwd}
  getTreeIcon={getFileIcon}
  {modeOptions}
  activeMode={activeTab}
  onModeChange={handleModeChange}
  breadcrumbPath={directory}
  onBreadcrumbNavigate={zoomToDirectory}
  {appTabs}
  fullWidth
  noPadding
>
  {#if !selectedFile && !directory}
    <AppHeader appName="KVASIR" subtitle="Workspace Inspector" />
  {/if}

  {#if error}
    <ErrorBanner onDismiss={() => error = ""}>{error}</ErrorBanner>
  {/if}

  {#if isJsonlFile && selectedFile}
    <section class="file-info">
      <div class="file-path">
        <strong>{relativePath(selectedFile)}</strong>
        <div class="file-actions">
          <Button variant="ghost" onclick={cycleWrap} title="Cycle: no wrap → wrap 79 → wrap to width" active={wrapMode !== "nowrap"}>{wrapLabel()}</Button>
          <Button variant="ghost" onclick={refresh}>Refresh</Button>
        </div>
      </div>
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
      {refreshKey}
    />
  {:else if isTableFile && selectedFile}
    <section class="file-info">
      <div class="file-path">
        <strong>{relativePath(selectedFile)}</strong>
        <div class="file-actions">
          <Button variant="ghost" onclick={refresh}>Refresh</Button>
        </div>
      </div>
    </section>
    <TableViewer
      commands={{
        read_table: commands.read_table,
        export_table_csv: commands.export_table_csv,
        open_in_editor: commands.open_in_editor,
      }}
      path={selectedFile}
      {refreshKey}
    />
  {:else if fileContent}
    <section class="file-info">
      <div class="file-path">
        <strong>{relativePath(fileContent.path)}</strong>
        <div class="file-actions">
          <Button variant="ghost" onclick={refresh}>Refresh</Button>
          <Button variant="ghost" onclick={() => openInEditor()}>Open in Editor</Button>
        </div>
      </div>
      <div class="file-meta">
        <span class="meta-item">Language: <strong>{fileContent.language}</strong></span>
        <span class="meta-item">Lines: <strong>{fileContent.line_count}</strong></span>
        <span class="meta-item">Size: <strong>{formatBytes(fileContent.size_bytes)}</strong></span>
      </div>
    </section>

    <!-- Data view format selector -->
    {#if activeTab === "data" && dataFormats}
      <FormatControls {dataFormats} bind:selectedFormat />
    {/if}

    <div class="content-controls">
      <Button variant="ghost" onclick={cycleWrap} title="Cycle: no wrap → wrap 79 → wrap to width" active={wrapMode !== "nowrap"}>{wrapLabel()}</Button>
    </div>

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
        <pre><code>{#each displayContent.split('\n') as line, i}{@const highlighted = highlightedContent.split('\n')[i] || ''}<span class="line-number" data-line={i + 1}>{i + 1}</span><span class="line-content">{@html highlighted}</span>
{/each}</code></pre>
      </section>
    {/if}
  {:else if !loading && directory}
    <EmptyState message="Select a file from the tree to view its contents" />
  {:else if !directory}
    <EmptyState message="Select a directory to browse files" />
  {/if}
</ContainerLayout>

<style>
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

  .file-actions {
    display: flex;
    gap: var(--space-sm);
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

  .content-controls {
    display: flex;
    gap: var(--space-sm);
    margin-bottom: var(--space-lg);
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
    font-size: var(--content-font-size, 14px);
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
    overflow: auto;
  }
</style>
