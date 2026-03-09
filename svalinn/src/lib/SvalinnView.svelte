<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { Button, SidebarLayout, TreeNode, StatCard, SearchInput, FilterBanner } from "@yggdrasil/ui";

  let {
    commands = {
      scan_directory: "scan_directory",
      list_qa_tree: "list_qa_tree",
      open_in_editor: "open_in_editor",
      run_saga: "run_saga",
    },
  }: {
    commands?: {
      scan_directory: string;
      list_qa_tree: string;
      open_in_editor: string;
      run_saga: string;
    };
  } = $props();

  interface Issue {
    tool: string;
    code: string;
    severity: string;
    file: string;
    line: number;
    column: number | null;
    message: string;
    signal: string;
    direction: string;
    canary: string;
  }

  interface ScanResult {
    directory: string;
    files_scanned: number;
    issues: Issue[];
    by_tool: Record<string, number>;
    by_severity: Record<string, number>;
    by_file: Record<string, number>;
    by_code: Record<string, number>;
  }

  interface SagaResult {
    success: boolean;
    files_analyzed: number;
    total_issues: number;
    output: string;
  }

  interface FileTreeEntry {
    name: string;
    path: string;
    is_dir: boolean;
    has_sidecar: boolean;
    issue_count: number;
  }

  interface SvalTreeNode extends FileTreeEntry {
    expanded: boolean;
    children: SvalTreeNode[];
    loading: boolean;
  }

  let directory = $state("");
  let includeTests = $state(false);
  let scanResult: ScanResult | null = $state(null);
  let loading = $state(false);
  let sagaRunning = $state(false);
  let viewMode: "by_file" | "by_code" | "by_tool" = $state("by_file");
  let severityFilter = $state("All");
  let toolFilter = $state("All");
  let searchQuery = $state("");
  let showTree = $state(true);
  let treeRoot: SvalTreeNode | null = $state(null);
  let selectedFile: string | null = $state(null);

  async function selectDirectory() {
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (selected && typeof selected === "string") {
      directory = selected;
      await refresh();
      await loadTree();
    }
  }

  async function refresh() {
    if (!directory) return;
    loading = true;
    try {
      scanResult = await invoke<ScanResult>(commands.scan_directory, {
        directory,
        includeTests,
      });
    } catch (e) {
      console.error("Scan failed:", e);
    }
    loading = false;
  }

  async function loadTree() {
    if (!directory) return;
    try {
      const entries = await invoke<FileTreeEntry[]>(commands.list_qa_tree, { directory });
      treeRoot = {
        name: directory.split("/").pop() || directory,
        path: directory,
        is_dir: true,
        has_sidecar: false,
        issue_count: entries.reduce((sum, e) => sum + e.issue_count, 0),
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
      console.error("Failed to load tree:", e);
    }
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
          const entries = await invoke<FileTreeEntry[]>(commands.list_qa_tree, { directory: node.path });
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
    // Trigger reactivity
    treeRoot = treeRoot;
  }

  function handleTreeSelect(path: string) {
    selectedFile = path;
  }

  function findNode(node: SvalTreeNode, path: string): SvalTreeNode | null {
    if (node.path === path) return node;
    for (const child of node.children) {
      const found = findNode(child, path);
      if (found) return found;
    }
    return null;
  }

  async function runSaga() {
    if (!directory) return;
    sagaRunning = true;
    try {
      const result = await invoke<SagaResult>(commands.run_saga, { directory });
      console.log("Saga result:", result);
      await refresh();
      await loadTree();
    } catch (e) {
      console.error("Saga failed:", e);
    }
    sagaRunning = false;
  }

  async function openInEditor(file: string, line: number) {
    try {
      await invoke(commands.open_in_editor, { path: file, line });
    } catch (e) {
      console.error("Failed to open in editor:", e);
    }
  }

  let filteredIssues = $derived.by(() => {
    if (!scanResult) return [];
    let issues = scanResult.issues;

    if (selectedFile) {
      issues = issues.filter((i) => i.file === selectedFile);
    }
    if (severityFilter !== "All") {
      issues = issues.filter((i) => i.severity === severityFilter);
    }
    if (toolFilter !== "All") {
      issues = issues.filter((i) => i.tool === toolFilter);
    }
    if (searchQuery) {
      const q = searchQuery.toLowerCase();
      issues = issues.filter(
        (i) =>
          i.file.toLowerCase().includes(q) ||
          i.message.toLowerCase().includes(q) ||
          i.code.toLowerCase().includes(q)
      );
    }
    return issues;
  });

  let groupedIssues = $derived.by(() => {
    const issues = filteredIssues;
    const groups: Record<string, Issue[]> = {};

    for (const issue of issues) {
      let key: string;
      switch (viewMode) {
        case "by_file":
          key = issue.file;
          break;
        case "by_code":
          key = `${issue.tool}:${issue.code}`;
          break;
        case "by_tool":
          key = issue.tool;
          break;
      }
      if (!groups[key]) groups[key] = [];
      groups[key].push(issue);
    }

    return Object.entries(groups).sort((a, b) => b[1].length - a[1].length);
  });

  function relativePath(fullPath: string): string {
    if (directory && fullPath.startsWith(directory)) {
      return fullPath.slice(directory.length + 1);
    }
    return fullPath;
  }

  function severityColor(severity: string): string {
    switch (severity) {
      case "blocked":
        return "var(--severity-blocked)";
      case "error":
        return "var(--severity-error)";
      case "warning":
        return "var(--severity-warning)";
      default:
        return "var(--severity-info)";
    }
  }

  function clearFileFilter() {
    selectedFile = null;
  }

  const severityPriority: Record<string, number> = {
    blocked: 4,
    error: 3,
    warning: 2,
    info: 1,
  };

  let filteredDataByPath = $derived.by(() => {
    if (!scanResult) return { counts: new Map<string, number>(), severities: new Map<string, string>() };

    let issues = scanResult.issues;
    if (severityFilter !== "All") {
      issues = issues.filter((i) => i.severity === severityFilter);
    }
    if (toolFilter !== "All") {
      issues = issues.filter((i) => i.tool === toolFilter);
    }
    if (searchQuery) {
      const q = searchQuery.toLowerCase();
      issues = issues.filter(
        (i) =>
          i.file.toLowerCase().includes(q) ||
          i.message.toLowerCase().includes(q) ||
          i.code.toLowerCase().includes(q)
      );
    }

    const countsByFile = new Map<string, number>();
    const maxSeverityByFile = new Map<string, string>();
    for (const issue of issues) {
      countsByFile.set(issue.file, (countsByFile.get(issue.file) || 0) + 1);
      const current = maxSeverityByFile.get(issue.file);
      if (!current || severityPriority[issue.severity] > severityPriority[current]) {
        maxSeverityByFile.set(issue.file, issue.severity);
      }
    }
    return { counts: countsByFile, severities: maxSeverityByFile };
  });

  function getFilteredCount(path: string, isDir: boolean): number {
    if (!isDir) {
      return filteredDataByPath.counts.get(path) || 0;
    }
    let sum = 0;
    for (const [filePath, count] of filteredDataByPath.counts) {
      if (filePath.startsWith(path + "/")) {
        sum += count;
      }
    }
    return sum;
  }

  function getMaxSeverity(path: string, isDir: boolean): string {
    if (!isDir) {
      return filteredDataByPath.severities.get(path) || "info";
    }
    let maxPriority = 0;
    let maxSev = "info";
    for (const [filePath, sev] of filteredDataByPath.severities) {
      if (filePath.startsWith(path + "/")) {
        const p = severityPriority[sev] || 0;
        if (p > maxPriority) {
          maxPriority = p;
          maxSev = sev;
        }
      }
    }
    return maxSev;
  }

  let filteredStats = $derived.by(() => {
    if (!scanResult) return { total: 0, blocked: 0, error: 0, warning: 0 };

    let issues = scanResult.issues;
    if (toolFilter !== "All") {
      issues = issues.filter((i) => i.tool === toolFilter);
    }
    if (severityFilter !== "All") {
      issues = issues.filter((i) => i.severity === severityFilter);
    }
    if (searchQuery) {
      const q = searchQuery.toLowerCase();
      issues = issues.filter(
        (i) =>
          i.file.toLowerCase().includes(q) ||
          i.message.toLowerCase().includes(q) ||
          i.code.toLowerCase().includes(q)
      );
    }

    const bySeverity: Record<string, number> = {};
    for (const issue of issues) {
      bySeverity[issue.severity] = (bySeverity[issue.severity] || 0) + 1;
    }

    return {
      total: issues.length,
      blocked: bySeverity["blocked"] || 0,
      error: bySeverity["error"] || 0,
      warning: bySeverity["warning"] || 0,
    };
  });
</script>

<SidebarLayout
  showSidebar={showTree && treeRoot !== null}
  sidebarTitle="Files"
  onCloseSidebar={() => showTree = false}
>
  {#snippet sidebar()}
    {#if treeRoot}
      <TreeNode
        node={treeRoot}
        selected={selectedFile}
        onToggle={handleTreeToggle}
        onSelect={handleTreeSelect}
        getBadgeCount={getFilteredCount}
        getBadgeSeverity={getMaxSeverity}
      />
    {/if}
  {/snippet}

  <header>
    <h1>Svalinn</h1>
    <p class="subtitle">Code Quality Viewer</p>
  </header>

  <section class="controls">
    <div class="directory-row">
      {#if !showTree && treeRoot}
        <Button variant="ghost" onclick={() => showTree = true}>&#128450;</Button>
      {/if}
      <Button onclick={selectDirectory}>Select Directory</Button>
      <input
        type="text"
        bind:value={directory}
        placeholder="Or paste path here..."
        class="directory-input"
      />
      <Button variant="primary" onclick={refresh} disabled={!directory}>
        {loading ? "Scanning..." : "Refresh"}
      </Button>
      <Button variant="special" onclick={runSaga} disabled={!directory}>
        {sagaRunning ? "Running..." : "Run Saga"}
      </Button>
    </div>

    <div class="options-row">
      <label>
        <input type="checkbox" bind:checked={includeTests} />
        Include tests/
      </label>
    </div>
  </section>

  {#if scanResult}
    <section class="stats">
      <StatCard value={scanResult.files_scanned} label="sidecars read" />
      <StatCard value={filteredStats.total} label="total issues" />
      <StatCard value={filteredStats.blocked} label="blocked" severity="blocked" />
      <StatCard value={filteredStats.error} label="errors" severity="error" />
      <StatCard value={filteredStats.warning} label="warnings" severity="warning" />
    </section>

    <section class="filters">
      <div class="view-modes">
        <span>View:</span>
        <button class="view-btn" class:active={viewMode === "by_file"} onclick={() => (viewMode = "by_file")}>By File</button>
        <button class="view-btn" class:active={viewMode === "by_code"} onclick={() => (viewMode = "by_code")}>By Error Type</button>
        <button class="view-btn" class:active={viewMode === "by_tool"} onclick={() => (viewMode = "by_tool")}>By Tool</button>
      </div>

      <div class="filter-selects">
        <label>
          Severity:
          <select bind:value={severityFilter}>
            <option>All</option>
            <option value="blocked">Blocked</option>
            <option value="error">Error</option>
            <option value="warning">Warning</option>
            <option value="info">Info</option>
          </select>
        </label>

        <label>
          Tool:
          <select bind:value={toolFilter}>
            <option>All</option>
            {#each Object.entries(scanResult.by_tool) as [tool, count]}
              <option value={tool}>{tool} ({count})</option>
            {/each}
          </select>
        </label>
      </div>
    </section>

    {#if selectedFile}
      <FilterBanner label="Showing issues for" value={relativePath(selectedFile)} onClear={clearFileFilter} />
    {/if}

    <SearchInput bind:value={searchQuery} placeholder="Search files, messages, codes..." />

    <section class="results">
      <p class="results-count">{filteredIssues.length} issues shown</p>

      {#each groupedIssues as [group, issues]}
        <details class="group" open={groupedIssues.length <= 10}>
          <summary>
            <span class="group-name">{viewMode === "by_file" ? relativePath(group) : group}</span>
            <span class="group-count">{issues.length}</span>
          </summary>
          <ul class="issues">
            {#each issues as issue}
              <li class="issue-row">
                <div
                  class="issue"
                  onclick={() => openInEditor(issue.file, issue.line)}
                  onkeydown={(e) => e.key === 'Enter' && openInEditor(issue.file, issue.line)}
                  role="button"
                  tabindex="0"
                >
                  <span class="issue-location">
                    {#if viewMode !== "by_file"}
                      <span class="issue-file">{relativePath(issue.file)}</span>
                    {/if}
                    <span class="issue-line">:{issue.line}</span>
                  </span>
                  <span class="issue-code" style="color: {severityColor(issue.severity)}">
                    [{issue.tool}:{issue.code}]
                  </span>
                  <span class="issue-message">{issue.message}</span>
                </div>
                {#if issue.signal || issue.direction || issue.canary}
                  <div class="issue-detail">
                    {#if issue.signal}
                      <p class="issue-signal"><span class="detail-label">Signal</span> {issue.signal}</p>
                    {/if}
                    {#if issue.direction}
                      <p class="issue-direction"><span class="detail-label">Direction</span> {issue.direction}</p>
                    {/if}
                    {#if issue.canary}
                      <p class="issue-canary"><span class="detail-label">Canary</span> {issue.canary}</p>
                    {/if}
                  </div>
                {/if}
              </li>
            {/each}
          </ul>
        </details>
      {/each}
    </section>
  {:else if !loading}
    <section class="empty-state">
      <p>Select a directory to view .qa sidecars generated by Saga</p>
    </section>
  {/if}
</SidebarLayout>

<style>
  header {
    text-align: center;
    margin-bottom: var(--space-2xl);
  }

  h1 {
    margin: 0;
    font-size: var(--text-3xl);
    color: var(--text-primary);
  }

  .subtitle {
    margin: var(--space-sm) 0 0;
    color: var(--text-secondary);
  }

  .controls {
    background: var(--bg-secondary);
    padding: var(--space-xl);
    border-radius: var(--radius-md);
    margin-bottom: var(--space-xl);
  }

  .directory-row {
    display: flex;
    gap: var(--space-sm);
    margin-bottom: var(--space-lg);
  }

  .directory-input {
    flex: 1;
    padding: var(--space-sm) var(--space-lg);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family: var(--font-body);
  }

  .options-row {
    display: flex;
    gap: var(--space-lg);
  }

  .options-row label {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    cursor: pointer;
  }

  .stats {
    display: flex;
    gap: var(--space-lg);
    margin-bottom: var(--space-xl);
  }

  .filters {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-lg);
    flex-wrap: wrap;
    gap: var(--space-lg);
  }

  .view-modes {
    display: flex;
    gap: var(--space-sm);
    align-items: center;
  }

  .view-btn {
    padding: var(--space-sm) var(--space-lg);
    border: none;
    border-radius: var(--radius-sm);
    background: var(--action-neutral);
    color: var(--text-primary);
    cursor: pointer;
  }

  .view-btn:hover {
    background: var(--action-neutral-hover);
  }

  .view-btn.active {
    background: var(--action-primary);
  }

  .filter-selects {
    display: flex;
    gap: var(--space-lg);
  }

  .filter-selects label {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  select {
    padding: var(--space-sm);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .results-count {
    color: var(--text-secondary);
    margin-bottom: var(--space-lg);
  }

  .group {
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    margin-bottom: var(--space-sm);
  }

  .group summary {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-md) var(--space-lg);
    cursor: pointer;
    user-select: none;
  }

  .group summary:hover {
    background: var(--bg-hover);
  }

  .group-name {
    font-family: var(--font-mono);
  }

  .group-count {
    background: var(--action-primary);
    padding: var(--space-xs) var(--space-md);
    border-radius: var(--radius-full);
    font-size: var(--text-sm);
  }

  .issues {
    list-style: none;
    margin: 0;
    padding: 0;
    border-top: 1px solid var(--border-default);
  }

  .issue-row {
    border-bottom: 1px solid var(--border-subtle);
  }

  .issue-row:last-child {
    border-bottom: none;
  }

  .issue {
    padding: var(--space-sm) var(--space-lg);
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    display: flex;
    gap: var(--space-sm);
    flex-wrap: wrap;
  }

  .issue:hover {
    background: var(--bg-hover);
  }

  .issue-detail {
    padding: var(--space-xs) var(--space-lg) var(--space-md) var(--space-2xl);
    font-size: var(--text-sm);
    font-family: var(--font-body);
    border-top: 1px solid var(--border-subtle);
    background: var(--bg-primary);
  }

  .issue-detail p {
    margin: var(--space-xs) 0;
    line-height: 1.5;
  }

  .detail-label {
    font-weight: bold;
    margin-right: var(--space-sm);
  }

  .issue-signal .detail-label {
    color: var(--severity-error);
  }

  .issue-direction .detail-label {
    color: var(--severity-info);
  }

  .issue-canary .detail-label {
    color: var(--severity-warning);
  }

  .issue-location {
    color: var(--text-secondary);
  }

  .issue-file {
    color: var(--severity-info);
  }

  .issue-line {
    color: var(--text-secondary);
  }

  .issue-code {
    font-weight: bold;
  }

  .issue-message {
    color: var(--text-muted);
  }

  .empty-state {
    text-align: center;
    padding: 4rem var(--space-2xl);
    color: var(--text-secondary);
  }
</style>
