<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { homeDir } from "@tauri-apps/api/path";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import { Button, ContainerLayout, StatCard, SearchInput, FilterBanner, EmptyState, ToggleGroup, Select, Checkbox, Collapsible } from "@yggdrasil/ui";

  let {
    commands = {
      scan_directory: "scan_directory",
      list_qa_tree: "list_qa_tree",
      open_in_editor: "open_in_editor",
      run_saga: "run_saga",
    },
    storagePrefix = "solo",
    appTabs,
  }: {
    commands?: {
      scan_directory: string;
      list_qa_tree: string;
      open_in_editor: string;
      run_saga: string;
    };
    storagePrefix?: string;
    appTabs?: { (): any };
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

  // ── State ──────────────────────────────────────────────────────────────

  let directory = $state("");
  let showHidden = $state(false);
  let includeTests = $state(false);
  let scanResult: ScanResult | null = $state(null);
  let loading = $state(false);
  let sagaRunning = $state(false);
  let viewMode: "by_file" | "by_code" | "by_tool" = $state("by_file");
  let severityFilter = $state("All");
  let toolFilter = $state("All");
  let searchQuery = $state("");
  let treeRoot: SvalTreeNode | null = $state(null);
  let selectedFile: string | null = $state(null);

  // ── Mode bar ─────────────────────────────────────────────────────────

  const modeOptions = [
    { value: "by_file", label: "By File", icon: "F" },
    { value: "by_code", label: "By Error", icon: "E" },
    { value: "by_tool", label: "By Tool", icon: "T" },
  ];

  function handleModeChange(value: string) {
    viewMode = value as "by_file" | "by_code" | "by_tool";
  }

  // ── Directory navigation ─────────────────────────────────────────────

  function parentDir(path: string): string | null {
    const i = path.lastIndexOf("/");
    return i > 0 ? path.substring(0, i) : null;
  }

  function zoomToDirectory(path: string) {
    directory = path;
    selectedFile = null;
    scanResult = null;
    refresh();
    loadTree();
  }

  function handleDirectoryChange(path: string) {
    zoomToDirectory(path);
  }

  async function handleSetCwd() {
    const selected = await open({ directory: true, multiple: false });
    if (selected && typeof selected === "string") {
      directory = selected;
      await refresh();
      await loadTree();
    }
  }

  onMount(async () => {
    if (!directory) {
      const home = await homeDir();
      directory = home.replace(/\/?$/, "/.ai");
      await loadTree();
    }
  });

  // ── Tree operations ────────────────────────────────────────────────────

  function getTreeNodes(): SvalTreeNode[] {
    return treeRoot ? treeRoot.children : [];
  }
  let treeNodes = $derived(getTreeNodes());

  async function refresh() {
    if (!directory) return;
    loading = true;
    try {
      scanResult = await invoke<ScanResult>(commands.scan_directory, {
        directory,
        includeTests,
      });
    } catch (_) {
      // scan failure surfaced via empty scanResult
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
        issue_count: entries.reduce((total, entry) => total + entry.issue_count, 0),
        expanded: true,
        children: entries.map(entry => ({
          ...entry,
          expanded: false,
          children: [],
          loading: false,
        })),
        loading: false,
      };
    } catch (_) {
      // tree failure surfaced via null treeRoot
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
          node.children = entries.map(entry => ({
            ...entry,
            expanded: false,
            children: [],
            loading: false,
          }));
        } catch (_) {
          // child load failure leaves node empty
        }
        node.loading = false;
      }
      node.expanded = true;
    }
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

  // ── Actions ────────────────────────────────────────────────────────────

  async function runSaga() {
    if (!directory) return;
    sagaRunning = true;
    try {
      await invoke<SagaResult>(commands.run_saga, { directory });
      await refresh();
      await loadTree();
    } catch (_) {
      // saga failure surfaced via unchanged scanResult
    }
    sagaRunning = false;
  }

  async function openInEditor(file: string, line: number) {
    try {
      await invoke(commands.open_in_editor, { path: file, line });
    } catch (_) {
      // editor open is best-effort
    }
  }

  // ── Derived state ──────────────────────────────────────────────────────

  let baseFilteredIssues = $derived.by(() => {
    if (!scanResult) return [];
    let issues = scanResult.issues;
    if (severityFilter !== "All") issues = issues.filter(i => i.severity === severityFilter);
    if (toolFilter !== "All") issues = issues.filter(i => i.tool === toolFilter);
    if (searchQuery) {
      const needle = searchQuery.toLowerCase();
      issues = issues.filter(issue =>
        issue.file.toLowerCase().includes(needle) ||
        issue.message.toLowerCase().includes(needle) ||
        issue.code.toLowerCase().includes(needle)
      );
    }
    return issues;
  });

  let filteredIssues = $derived.by(() => {
    if (!selectedFile) return baseFilteredIssues;
    return baseFilteredIssues.filter(i => i.file === selectedFile);
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

    return Object.entries(groups).sort((groupA, groupB) => groupB[1].length - groupA[1].length);
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
        return "var(--severity-success)";
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
    const countsByFile = new Map<string, number>();
    const maxSeverityByFile = new Map<string, string>();
    for (const issue of baseFilteredIssues) {
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
    for (const [filePath, severity] of filteredDataByPath.severities) {
      if (filePath.startsWith(path + "/")) {
        const priority = severityPriority[severity] || 0;
        if (priority > maxPriority) {
          maxPriority = priority;
          maxSev = severity;
        }
      }
    }
    return maxSev;
  }

  let filteredStats = $derived.by(() => {
    const bySeverity: Record<string, number> = {};
    for (const issue of baseFilteredIssues) {
      bySeverity[issue.severity] = (bySeverity[issue.severity] || 0) + 1;
    }
    return {
      total: baseFilteredIssues.length,
      blocked: bySeverity["blocked"] || 0,
      error: bySeverity["error"] || 0,
      warning: bySeverity["warning"] || 0,
    };
  });
</script>

<ContainerLayout
  appName="svalinn"
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
  getTreeBadgeCount={getFilteredCount}
  getTreeBadgeSeverity={getMaxSeverity}
  {modeOptions}
  activeMode={viewMode}
  onModeChange={handleModeChange}
  breadcrumbPath={directory}
  onBreadcrumbNavigate={zoomToDirectory}
  {appTabs}
>
  <section class="controls">
    <Button variant="primary" onclick={refresh} disabled={!directory}>
      {loading ? "Scanning..." : "Refresh"}
    </Button>
    <Button variant="special" onclick={runSaga} disabled={!directory}>
      {sagaRunning ? "Running..." : "Run Saga"}
    </Button>
    <Checkbox bind:checked={includeTests} label="Include tests/" />
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
      <div class="filter-selects">
        <label>
          Severity:
          <Select bind:value={severityFilter}>
            <option>All</option>
            <option value="blocked">Blocked</option>
            <option value="error">Error</option>
            <option value="warning">Warning</option>
            <option value="info">Info</option>
          </Select>
        </label>

        <label>
          Tool:
          <Select bind:value={toolFilter}>
            <option>All</option>
            {#each Object.entries(scanResult.by_tool) as [tool, count]}
              <option value={tool}>{tool} ({count})</option>
            {/each}
          </Select>
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
        <Collapsible title={viewMode === "by_file" ? relativePath(group) : group} badgeCount={issues.length} open={groupedIssues.length <= 10}>
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
        </Collapsible>
      {/each}
    </section>
  {:else if !loading}
    <EmptyState message="Select a directory to view .qa sidecars generated by Saga" />
  {/if}
</ContainerLayout>

<style>
  .controls {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md) var(--space-xl);
    margin-bottom: var(--space-xl);
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
    color: var(--action-primary);
  }

  .issue-canary .detail-label {
    color: var(--severity-warning);
  }

  .issue-location {
    color: var(--text-secondary);
  }

  .issue-file {
    color: var(--action-primary);
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
</style>
