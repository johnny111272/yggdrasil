<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { Button, SidebarLayout } from "@yggdrasil/ui";
  import hljs from "highlight.js";
  import "highlight.js/styles/github-dark.css";
  import { marked } from "marked";
  import SchemaInspector from "./SchemaInspector.svelte";
  import { analyzeSchema, type InspectedSchema } from "./schema-inspect";

  let {
    commands = {
      list_directory: "list_directory",
      read_file: "read_file",
      open_in_editor: "open_in_editor",
      convert_to_all_formats: "convert_to_all_formats",
      detect_data_format: "detect_data_format",
    },
  }: {
    commands?: {
      list_directory: string;
      read_file: string;
      open_in_editor: string;
      convert_to_all_formats: string;
      detect_data_format: string;
    };
  } = $props();

  interface FileTreeEntry {
    name: string;
    path: string;
    is_dir: boolean;
    extension: string | null;
    size_bytes: number;
  }

  interface FileContent {
    path: string;
    content: string;
    language: string;
    line_count: number;
    size_bytes: number;
  }

  interface FormatConversion {
    content: string;
    token_count: number;
  }

  interface AllFormats {
    json: FormatConversion;
    yaml: FormatConversion;
    toml: FormatConversion;
    toon: FormatConversion;
    source_format: string;
  }

  interface TreeNode extends FileTreeEntry {
    expanded: boolean;
    children: TreeNode[];
    loading: boolean;
  }

  type ViewTab = "code" | "data" | "preview" | "inspect";
  type DataFormat = "json" | "yaml" | "toml" | "toon";

  let directory = $state("");
  let treeRoot: TreeNode | null = $state(null);
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
  let inspectedSchema: InspectedSchema | null = $state(null);
  let showHidden = $state(false);

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

  async function toggleNode(node: TreeNode) {
    if (!node.is_dir) {
      await loadFile(node.path);
      return;
    }

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
  }

  async function loadFile(path: string) {
    selectedFile = path;
    error = "";
    dataFormats = null;

    try {
      fileContent = await invoke<FileContent>(commands.read_file, { path });

      // Check if it's a markdown file
      isMarkdownFile = fileContent.language === "markdown";

      // Check if it's a JSON Schema file (.schema.json)
      isSchemaFile = path.endsWith(".schema.json");
      inspectedSchema = null;
      if (isSchemaFile && fileContent) {
        try {
          inspectedSchema = analyzeSchema(fileContent.content);
        } catch (e) {
          console.error("Schema inspection failed:", e);
        }
      }

      // Check if it's a data file and load conversions
      const format = await invoke<string | null>(commands.detect_data_format, { path });
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

      // Default tab selection
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

  function getIcon(node: TreeNode): string {
    if (node.is_dir) {
      if (node.loading) return "\u23F3";
      return node.expanded ? "\uD83D\uDCC2" : "\uD83D\uDCC1";
    }
    switch (node.extension) {
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

  // Map language names to highlight.js language identifiers
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

  // Get current display content based on active tab and format
  let displayContent = $derived.by(() => {
    if (activeTab === "data" && dataFormats) {
      return dataFormats[selectedFormat].content;
    }
    return fileContent?.content || "";
  });

  // Syntax highlighted content
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

  // Rendered markdown
  let renderedMarkdown = $derived.by(() => {
    if (!fileContent?.content || !isMarkdownFile) return "";
    return marked(fileContent.content) as string;
  });

  let displayLineCount = $derived.by(() => {
    return displayContent.split('\n').length;
  });

  // Token savings calculation
  let tokenStats = $derived.by(() => {
    if (!dataFormats) return null;
    const source = dataFormats.source_format as DataFormat;
    const baseline = dataFormats[source].token_count;
    return {
      json: { count: dataFormats.json.token_count, savings: baseline - dataFormats.json.token_count },
      yaml: { count: dataFormats.yaml.token_count, savings: baseline - dataFormats.yaml.token_count },
      toml: { count: dataFormats.toml.token_count, savings: baseline - dataFormats.toml.token_count },
      toon: { count: dataFormats.toon.token_count, savings: baseline - dataFormats.toon.token_count },
      source,
      baseline,
    };
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
    {#snippet renderNode(node: TreeNode, depth: number)}
      <div
        class="tree-node"
        class:selected={selectedFile === node.path}
        style="padding-left: {depth * 16 + 8}px"
        onclick={() => toggleNode(node)}
        onkeydown={(e) => e.key === 'Enter' && toggleNode(node)}
        role="button"
        tabindex="0"
      >
        <span class="tree-icon">{getIcon(node)}</span>
        <span class="tree-name">{node.name}</span>
      </div>
      {#if node.expanded && node.children.length > 0}
        {#each node.children as child}
          {@render renderNode(child, depth + 1)}
        {/each}
      {/if}
    {/snippet}
    {#if treeRoot}
      {@render renderNode(treeRoot, 0)}
    {/if}
  {/snippet}

  <header>
    <h1>Kvasir</h1>
    <p class="subtitle">Workspace Inspector</p>
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

    {#if fileContent}
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
      </section>

      <!-- Data view format selector and token stats -->
      {#if activeTab === "data" && dataFormats && tokenStats}
        <section class="data-controls">
          <div class="format-selector">
            {#each ["json", "yaml", "toml", "toon"] as fmt}
              <button
                class="format-btn"
                class:active={selectedFormat === fmt}
                class:source={tokenStats.source === fmt}
                onclick={() => selectedFormat = fmt as DataFormat}
              >
                {fmt.toUpperCase()}
              </button>
            {/each}
          </div>
          <div class="token-stats">
            <span class="token-label">Tokens:</span>
            {#each ["json", "yaml", "toml", "toon"] as fmt}
              {@const stat = tokenStats[fmt as DataFormat]}
              <span
                class="token-item"
                class:source={tokenStats.source === fmt}
                class:active={selectedFormat === fmt}
              >
                {fmt.toUpperCase()}: {stat.count.toLocaleString()}
                {#if tokenStats.source !== fmt}
                  <span class="savings" class:positive={stat.savings > 0} class:negative={stat.savings < 0}>
                    ({stat.savings > 0 ? '-' : '+'}{Math.abs(Math.round(stat.savings / tokenStats.baseline * 100))}%)
                  </span>
                {/if}
              </span>
            {/each}
          </div>
        </section>
      {/if}

      <!-- Schema Inspector -->
      {#if activeTab === "inspect" && inspectedSchema}
        <section class="inspector-view">
          <SchemaInspector schema={inspectedSchema} />
        </section>
      <!-- Markdown Preview -->
      {:else if activeTab === "preview" && isMarkdownFile}
        <section class="markdown-preview">
          {@html renderedMarkdown}
        </section>
      {:else}
        <section class="code-viewer">
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

  .tree-node {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-sm) var(--space-md);
    cursor: pointer;
    user-select: none;
  }

  .tree-node:hover {
    background: var(--bg-hover);
  }

  .tree-node.selected {
    background: var(--bg-selected);
  }

  .tree-icon {
    font-size: var(--text-sm);
    width: 1.25rem;
    text-align: center;
  }

  .tree-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: var(--text-sm);
  }

  header {
    text-align: center;
    margin-bottom: var(--space-3xl);
  }

  h1 {
    margin: 0;
    font-size: var(--text-3xl);
    color: var(--text-primary);
  }

  .subtitle {
    margin: var(--space-md) 0 0;
    color: var(--text-secondary);
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

  /* Tabs */
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

  /* Data controls */
  .data-controls {
    background: var(--bg-secondary);
    padding: var(--space-lg);
    border-radius: var(--radius-md);
    margin-bottom: var(--space-lg);
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
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

  .format-btn.source {
    border: 2px solid var(--severity-info);
  }

  .token-stats {
    display: flex;
    gap: var(--space-lg);
    align-items: center;
    font-size: var(--text-sm);
    color: var(--text-secondary);
    flex-wrap: wrap;
  }

  .token-label {
    font-weight: 500;
  }

  .token-item {
    padding: var(--space-xs) var(--space-sm);
    border-radius: var(--radius-sm);
  }

  .token-item.source {
    color: var(--severity-info);
  }

  .token-item.active {
    background: var(--bg-hover);
  }

  .savings {
    font-size: var(--text-xs);
    margin-left: var(--space-xs);
  }

  .savings.positive {
    color: var(--severity-info);
  }

  .savings.negative {
    color: var(--severity-warning);
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

  /* Markdown preview */
  .markdown-preview {
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    padding: var(--space-2xl);
    overflow-y: auto;
    line-height: 1.7;
  }

  .markdown-preview :global(h1),
  .markdown-preview :global(h2),
  .markdown-preview :global(h3),
  .markdown-preview :global(h4) {
    color: var(--text-primary);
    margin-top: var(--space-2xl);
    margin-bottom: var(--space-lg);
  }

  .markdown-preview :global(h1) {
    font-size: var(--text-3xl);
    border-bottom: 1px solid var(--border-default);
    padding-bottom: var(--space-md);
  }

  .markdown-preview :global(h2) {
    font-size: var(--text-2xl);
    border-bottom: 1px solid var(--border-subtle);
    padding-bottom: var(--space-sm);
  }

  .markdown-preview :global(h3) {
    font-size: var(--text-xl);
  }

  .markdown-preview :global(p) {
    margin-bottom: var(--space-lg);
    color: var(--text-muted);
  }

  .markdown-preview :global(code) {
    background: var(--bg-primary);
    padding: var(--space-xs) var(--space-sm);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: 0.9em;
  }

  .markdown-preview :global(pre) {
    background: var(--bg-primary);
    padding: var(--space-lg);
    border-radius: var(--radius-md);
    overflow-x: auto;
    margin-bottom: var(--space-lg);
  }

  .markdown-preview :global(pre code) {
    background: none;
    padding: 0;
  }

  .markdown-preview :global(ul),
  .markdown-preview :global(ol) {
    margin-bottom: var(--space-lg);
    padding-left: var(--space-2xl);
    color: var(--text-muted);
  }

  .markdown-preview :global(li) {
    margin-bottom: var(--space-sm);
  }

  .markdown-preview :global(blockquote) {
    border-left: 4px solid var(--action-primary);
    margin: var(--space-lg) 0;
    padding: var(--space-md) var(--space-xl);
    background: var(--bg-primary);
    color: var(--text-secondary);
  }

  .markdown-preview :global(a) {
    color: var(--action-primary);
    text-decoration: none;
  }

  .markdown-preview :global(a:hover) {
    text-decoration: underline;
  }

  .markdown-preview :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: var(--space-lg);
  }

  .markdown-preview :global(th),
  .markdown-preview :global(td) {
    border: 1px solid var(--border-default);
    padding: var(--space-md);
    text-align: left;
  }

  .markdown-preview :global(th) {
    background: var(--bg-primary);
    font-weight: bold;
  }

  .markdown-preview :global(hr) {
    border: none;
    border-top: 1px solid var(--border-default);
    margin: var(--space-2xl) 0;
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
