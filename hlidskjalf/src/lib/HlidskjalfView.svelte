<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  import QualityReport from "./QualityReport.svelte";
  import TrafficReport from "./TrafficReport.svelte";

  // ── Command Map ──────────────────────────────────────────────────────
  let {
    commands = {
      start_monitor: "start_monitor",
      speak: "speak",
      open_in_editor: "open_in_editor",
    },
    onOpenFile,
  }: {
    commands?: { start_monitor: string; speak: string; open_in_editor: string };
    onOpenFile?: (path: string, line?: number) => void;
  } = $props();

  function handleOpenFile(path: string, line?: number) {
    if (onOpenFile) {
      onOpenFile(path, line);
    } else {
      invoke(commands.open_in_editor, { path, line: line ?? 1 });
    }
  }

  // ── Types ──────────────────────────────────────────────────────────

  interface Datagram {
    timestamp: number;
    source: string;
    kind: string;
    classifier?: string;
    priority: string;
    workspace: string;
    detail?: string;
    speech?: string;
    payload?: {
      [key: string]: unknown;
    };
  }

  // ── Priority helpers ────────────────────────────────────────────────

  function priorityNumeric(priority: string): number {
    switch (priority) {
      case "critical": return 4;
      case "high": return 3;
      case "normal": return 2;
      case "low": return 1;
      case "trace": return 0;
      default: return 0;
    }
  }

  function priorityClass(priority: string): string {
    switch (priority) {
      case "critical": return "priority-critical";
      case "high": return "priority-high";
      case "normal": return "priority-normal";
      case "low": return "priority-low";
      case "trace": return "priority-trace";
      default: return "priority-trace";
    }
  }

  // ── Kind helpers ──────────────────────────────────────────────────

  function kindIcon(kind: string): string {
    switch (kind) {
      case "alert": return "\uD83D\uDED1";
      case "warning": return "\u26A0\uFE0F";
      case "quality": return "\u274C";
      case "canary": return "\uD83D\uDC25";
      case "notify": return "\u2755";
      case "traffic": return "\uD83D\uDCE6";
      default: return "\u2753";
    }
  }

  function kindLabel(kind: string): string {
    switch (kind) {
      case "alert": return "ALERT";
      case "warning": return "WARNING";
      case "quality": return "QUALITY";
      case "canary": return "CANARY";
      case "notify": return "NOTIFY";
      case "traffic": return "TRAFFIC";
      default: return kind.toUpperCase();
    }
  }

  // ── Workspace colors ──────────────────────────────────────────────

  const WORKSPACE_HUES = [
    "hsl(210, 70%, 55%)",
    "hsl(35, 85%, 55%)",
    "hsl(0, 70%, 55%)",
    "hsl(270, 60%, 60%)",
    "hsl(160, 60%, 45%)",
    "hsl(320, 60%, 55%)",
  ];

  let workspaceColorMap: Map<string, string> = $state(new Map());

  function assignWorkspaceColor(workspace: string) {
    if (!workspace || workspaceColorMap.has(workspace)) return;
    const idx = workspaceColorMap.size % WORKSPACE_HUES.length;
    workspaceColorMap = new Map([...workspaceColorMap, [workspace, WORKSPACE_HUES[idx]]]);
  }

  function workspaceColor(workspace: string): string {
    if (!workspace) return "var(--text-secondary)";
    return workspaceColorMap.get(workspace) ?? "var(--text-secondary)";
  }

  // ── State ──────────────────────────────────────────────────────────

  let datagrams: Datagram[] = $state([]);
  let connected = $state(false);
  let autoScroll = $state(true);
  let enabledKinds: Set<string> = $state(new Set<string>());
  let filterPriorityMin = $state(0); // trace and above (show everything)
  let speechMinPriority = $state(3); // high+ by default
  let feedElement: HTMLElement | undefined = $state();
  let expandedRows: Set<number> = $state(new Set());
  let autoExpand = $state(false);

  function toggleRow(timestamp: number) {
    const next = new Set(expandedRows);
    if (next.has(timestamp)) next.delete(timestamp);
    else next.add(timestamp);
    expandedRows = next;
  }

  function toggleExpandAll() {
    autoExpand = !autoExpand;
    if (autoExpand) {
      const withPayload = datagrams.filter((entry) => entry.payload).map((entry) => entry.timestamp);
      expandedRows = new Set(withPayload);
    } else {
      expandedRows = new Set();
    }
  }

  // ── Derived ────────────────────────────────────────────────────────

  let allEnabled = $derived(datagramKinds.length > 0 && datagramKinds.every((kind) => enabledKinds.has(kind)));

  function toggleKind(kind: string) {
    const next = new Set(enabledKinds);
    if (next.has(kind)) next.delete(kind);
    else next.add(kind);
    enabledKinds = next;
  }

  function toggleAll() {
    enabledKinds = allEnabled ? new Set() : new Set(datagramKinds);
  }

  let filteredDatagrams = $derived(
    datagrams.filter((entry) => {
      if (!enabledKinds.has(entry.kind)) return false;
      if (priorityNumeric(entry.priority) < filterPriorityMin) return false;
      return true;
    }),
  );

  let datagramKinds = $derived([
    ...new Set(datagrams.map((entry) => entry.kind)),
  ]);

  let stats = $derived({
    total: datagrams.length,
    critical: datagrams.filter((entry) => entry.priority === "critical").length,
    high: datagrams.filter((entry) => entry.priority === "high").length,
  });

  // ── Lifecycle ──────────────────────────────────────────────────────

  onMount(async () => {
    const unlisten = await listen<Datagram>("datagram", (event) => {
      const incoming = event.payload;
      const cutoff = Date.now() / 1000 - 3600;

      assignWorkspaceColor(incoming.workspace);
      if (!enabledKinds.has(incoming.kind)) {
        enabledKinds = new Set([...enabledKinds, incoming.kind]);
      }
      datagrams = [...datagrams.filter((prior) => prior.timestamp > cutoff), incoming];

      if (autoExpand && incoming.payload) {
        expandedRows = new Set([...expandedRows, incoming.timestamp]);
      }

      if (incoming.speech && priorityNumeric(incoming.priority) >= speechMinPriority) {
        invoke(commands.speak, { text: incoming.speech });
      }

      if (autoScroll && feedElement) {
        requestAnimationFrame(() => {
          feedElement!.scrollTop = feedElement!.scrollHeight;
        });
      }
    });

    try {
      await invoke(commands.start_monitor);
      connected = true;
    } catch {
      connected = false;
    }

    return () => {
      unlisten();
    };
  });

  // ── Path detection ────────────────────────────────────────────────

  interface TextSegment {
    text: string;
    path?: string;
    line?: number;
  }

  function resolveNotation(notation: string): string {
    // @path/to/file or @seg:seg — both resolve under ~/.ai/
    const relative = notation.slice(1).replace(/:/g, "/");
    return `/Users/johnny/.ai/${relative}`;
  }

  function parsePathSegments(text: string): TextSegment[] {
    // Absolute paths (/...) or @-notation paths (@path/to/file:line or @seg:seg)
    const pattern = /(\/[\w.\/_-]+(?::(\d+))?)|((@[\w][\w.\/:_-]+?)(?::(\d+))?(?=\s|$|[,;)\]}"']))/g;
    const segments: TextSegment[] = [];
    let lastIndex = 0;
    let match;

    while ((match = pattern.exec(text)) !== null) {
      if (match.index > lastIndex) {
        segments.push({ text: text.slice(lastIndex, match.index) });
      }

      if (match[1]) {
        // Absolute path, possibly with :line
        const fullMatch = match[1];
        const lineNum = match[2] ? parseInt(match[2]) : undefined;
        const filePath = lineNum ? fullMatch.replace(`:${match[2]}`, "") : fullMatch;
        segments.push({ text: fullMatch, path: filePath, line: lineNum });
      } else if (match[4]) {
        // @-notation path → resolve to absolute path
        const notation = match[4];
        const lineNum = match[5] ? parseInt(match[5]) : undefined;
        const displayText = match[3]; // includes :line if present
        segments.push({ text: displayText, path: resolveNotation(notation), line: lineNum });
      }

      lastIndex = pattern.lastIndex;
    }

    if (lastIndex < text.length) {
      segments.push({ text: text.slice(lastIndex) });
    }

    return segments.length ? segments : [{ text }];
  }

  // ── Helpers ────────────────────────────────────────────────────────

  function formatTime(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    return date.toLocaleTimeString("en-US", { hour12: false });
  }

  function clearFeed() {
    datagrams = [];
  }

  function cycleSpeech() {
    // Cycle: 3 (high+) → 2 (normal+) → 4 (critical only) → 5 (silent) → 3
    if (speechMinPriority === 3) speechMinPriority = 2;
    else if (speechMinPriority === 2) speechMinPriority = 4;
    else if (speechMinPriority === 4) speechMinPriority = 5;
    else speechMinPriority = 3;
  }

  function speechLabel(): string {
    if (speechMinPriority >= 5) return "silent";
    if (speechMinPriority === 4) return "critical";
    if (speechMinPriority === 3) return "high+";
    if (speechMinPriority === 2) return "normal+";
    return "all";
  }
</script>

<div class="watchtower">
  <!-- Header bar -->
  <header class="header">
    <div class="header-left">
      <h1 class="title"><span class="app-name">HLIDSKJALF</span> <span class="sep">::</span> <span class="subtitle">Agent Watchtower</span></h1>
      <span class="status" class:connected>
        {connected ? "listening" : "disconnected"}
      </span>
    </div>
    <div class="header-right">
      <div class="stat">
        <span class="stat-value">{stats.total}</span>
        <span class="stat-label">events</span>
      </div>
      <div class="stat high">
        <span class="stat-value">{stats.high}</span>
        <span class="stat-label">high</span>
      </div>
      <div class="stat critical">
        <span class="stat-value">{stats.critical}</span>
        <span class="stat-label">critical</span>
      </div>
    </div>
  </header>

  <!-- Filter bar -->
  <div class="filters">
    <button
      class="filter-btn"
      class:active={allEnabled}
      onclick={toggleAll}
    >
      all
    </button>
    {#each datagramKinds as kind}
      <button
        class="filter-btn"
        class:active={enabledKinds.has(kind)}
        onclick={() => toggleKind(kind)}
      >
        {kindIcon(kind)}
        {kind}
      </button>
    {/each}
    <div class="filter-spacer"></div>
    <button
      class="filter-btn"
      class:active={autoExpand}
      onclick={toggleExpandAll}
      title={autoExpand ? "Collapse all" : "Expand all"}
    >
      {autoExpand ? "\u25BC" : "\u25B6"} expand
    </button>
    <label class="auto-scroll-toggle">
      <input type="checkbox" bind:checked={autoScroll} />
      auto-scroll
    </label>
    <button
      class="voice-btn"
      class:active={speechMinPriority < 5}
      onclick={cycleSpeech}
      title="Speech: {speechLabel()}"
    >
      {speechMinPriority >= 5 ? "\uD83D\uDD07" : "\uD83D\uDD0A"} {speechLabel()}
    </button>
    <button class="clear-btn" onclick={clearFeed}>clear</button>
  </div>

  <!-- Event feed -->
  <div class="feed" bind:this={feedElement}>

    {#if filteredDatagrams.length === 0}
      <div class="empty">
        <p class="empty-icon">{connected ? "\uD83D\uDC41" : "\u23F3"}</p>
        <p>{connected ? "Watching for events..." : "Connecting..."}</p>
      </div>
    {:else}
      {#each filteredDatagrams as datagram}
        {@const isExpanded = expandedRows.has(datagram.timestamp)}
        {@const hasPayload = !!datagram.payload}
        {@const isCanary = datagram.kind === "canary"}

        <div
          class="event-row {priorityClass(datagram.priority)}"
          class:canary={isCanary}
          class:expanded={isExpanded}
        >
          <div
            class="event-base"
            class:canary-base={isCanary}
            class:clickable={hasPayload}
            onclick={() => { if (hasPayload) toggleRow(datagram.timestamp); }}
            role={hasPayload ? "button" : undefined}
            tabindex={hasPayload ? 0 : undefined}
            onkeydown={(keyEvent) => { if (hasPayload && (keyEvent.key === "Enter" || keyEvent.key === " ")) { keyEvent.preventDefault(); toggleRow(datagram.timestamp); } }}
          >
            <span class="col-time">{formatTime(datagram.timestamp)}</span>
            <span class="col-kind-icon">{kindIcon(datagram.kind)}</span>

            {#if isCanary}
              <span class="col-source-inline">{datagram.source}</span>
              {#if datagram.workspace}
                <button class="col-workspace workspace-link" style="color: {workspaceColor(datagram.workspace)}" onclick={(event) => { event.stopPropagation(); handleOpenFile(resolveNotation(datagram.workspace)); }}>{datagram.workspace}</button>
              {/if}
            {:else}
              <span class="col-kind-label">{kindLabel(datagram.kind)}</span>
              {#if datagram.classifier}
                <span class="col-classifier">{datagram.classifier}</span>
              {/if}
              <button class="col-workspace workspace-link" style="color: {workspaceColor(datagram.workspace)}" onclick={(event) => { event.stopPropagation(); handleOpenFile(resolveNotation(datagram.workspace)); }}>{datagram.workspace}</button>
              {#if datagram.detail}
                <span class="col-detail">{#each parsePathSegments(datagram.detail) as segment}{#if segment.path}<button class="detail-path-link" onclick={(event) => { event.stopPropagation(); handleOpenFile(segment.path!, segment.line); }}>{segment.text}</button>{:else}{segment.text}{/if}{/each}</span>
              {/if}
              {#if hasPayload && !isExpanded}
                <span class="col-expand-hint">+</span>
              {/if}
              <span class="col-source">{datagram.source}</span>
            {/if}
          </div>

          {#if isExpanded && hasPayload}
            <div class="event-expanded">
              <div class="expanded-meta">
                <span class="expanded-badge">src: {datagram.source}</span>
                <span class="expanded-badge">{datagram.priority}</span>
                {#if datagram.classifier}
                  <span class="expanded-badge">{datagram.classifier}</span>
                {/if}
              </div>
              {#if datagram.kind === "quality"}
                <QualityReport
                  payload={datagram.payload}
                  workspace={datagram.workspace}
                  timestamp={formatTime(datagram.timestamp)}
                  onOpenFile={handleOpenFile}
                />
              {:else if datagram.kind === "traffic"}
                <TrafficReport
                  payload={datagram.payload}
                  workspace={datagram.workspace}
                  timestamp={formatTime(datagram.timestamp)}
                />
              {:else}
                <pre class="expanded-json">{JSON.stringify(datagram.payload, null, 2)}</pre>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .watchtower {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family: var(--font-mono);
  }

  /* ── Header ──────────────────────────────────────────────────────── */

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-sm) var(--space-lg);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--bg-tertiary);
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: baseline;
    gap: var(--space-md);
  }

  .title {
    font-size: var(--text-lg);
    font-weight: 700;
    margin: 0;
    display: flex;
    align-items: baseline;
    gap: var(--space-md);
  }

  .app-name {
    font-weight: 800;
    letter-spacing: 0.12em;
  }

  .sep {
    font-weight: 300;
    opacity: 0.5;
    color: var(--text-secondary);
  }

  .subtitle {
    font-size: var(--text-sm);
    font-weight: 300;
    color: var(--text-secondary);
    letter-spacing: 0.04em;
  }

  .status {
    font-size: var(--text-xs);
    padding: 2px 8px;
    border-radius: var(--radius-full);
    background: var(--severity-error);
    color: var(--bg-primary);
  }

  .status.connected {
    background: var(--severity-success);
  }

  .header-right {
    display: flex;
    gap: var(--space-lg);
  }

  .stat {
    text-align: center;
  }

  .stat-value {
    display: block;
    font-size: var(--text-xl);
    font-weight: 700;
  }

  .stat-label {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .stat.high .stat-value {
    color: var(--severity-warning);
  }

  .stat.critical .stat-value {
    color: var(--severity-error);
  }

  /* ── Filters ─────────────────────────────────────────────────────── */

  .filters {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-lg);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--bg-tertiary);
    flex-shrink: 0;
  }

  .filter-btn {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    padding: 2px 10px;
    border: 1px solid var(--bg-tertiary);
    border-radius: var(--radius-full);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .filter-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .filter-btn.active {
    background: var(--action-primary);
    color: var(--text-primary);
    border-color: var(--action-primary);
  }

  .filter-spacer {
    flex: 1;
  }

  .auto-scroll-toggle {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .voice-btn {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    padding: 2px 10px;
    border: 1px solid var(--bg-tertiary);
    border-radius: var(--radius-full);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .voice-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .voice-btn.active {
    border-color: var(--action-primary);
    color: var(--action-primary);
  }

  .clear-btn {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    padding: 2px 10px;
    border: 1px solid var(--bg-tertiary);
    border-radius: var(--radius-full);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .clear-btn:hover {
    background: var(--severity-error);
    color: var(--bg-primary);
    border-color: var(--severity-error);
  }

  /* ── Feed ─────────────────────────────────────────────────────────── */

  .feed {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-sm) var(--space-lg);
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary);
  }

  .empty-icon {
    font-size: 48px;
    margin-bottom: var(--space-md);
  }

  /* ── Event row — priority visual weight ─────────────────────────── */

  .event-row {
    padding: 0;
    border-bottom: 1px solid var(--bg-tertiary);
    font-size: var(--text-sm);
    border-left: 3px solid transparent;
  }

  .event-row.priority-trace {
    opacity: 0.5;
  }

  .event-row.priority-low {
    opacity: 0.7;
  }

  .event-row.priority-normal {
    border-left-color: var(--bg-tertiary);
  }

  .event-row.priority-high {
    border-left-color: var(--severity-warning);
    background: var(--priority-high-tint);
  }

  .event-row.priority-critical {
    border-left-color: var(--severity-error);
    background: var(--priority-critical-tint);
  }

  .event-row.canary {
    opacity: 0.5;
    border-left-color: transparent;
  }

  /* ── Base row layout ──────────────────────────────────────────── */

  .event-base {
    display: flex;
    gap: var(--space-sm);
    align-items: baseline;
    padding: var(--space-xs) var(--space-sm) var(--space-xs) var(--space-md);
    position: relative;
    min-height: 1.6em;
  }

  .event-base.clickable {
    cursor: pointer;
  }

  .event-base.clickable:hover {
    background: var(--bg-hover);
  }

  .canary-base {
    font-size: var(--text-xs);
    color: var(--text-muted);
    min-height: 1.2em;
  }

  /* ── Columns ──────────────────────────────────────────────────── */

  .col-time {
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
    min-width: 72px;
    flex-shrink: 0;
  }

  .col-kind-icon {
    min-width: 20px;
    text-align: center;
    flex-shrink: 0;
  }

  .col-kind-label {
    font-weight: 600;
    font-size: var(--text-xs);
    letter-spacing: 0.04em;
    min-width: 60px;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .col-classifier {
    font-size: var(--text-xs);
    color: var(--text-muted);
    padding: 1px 6px;
    border: 1px solid var(--bg-tertiary);
    border-radius: var(--radius-sm);
    flex-shrink: 0;
    white-space: nowrap;
  }

  .col-workspace {
    font-weight: 600;
    font-size: var(--text-xs);
    min-width: 80px;
    flex-shrink: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .workspace-link {
    background: none;
    border: none;
    font-family: inherit;
    font-size: inherit;
    font-weight: inherit;
    cursor: pointer;
    padding: 0;
    text-align: left;
  }

  .workspace-link:hover {
    text-decoration: underline;
  }

  .col-detail {
    color: var(--text-primary);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .detail-path-link {
    background: none;
    border: none;
    font-family: inherit;
    font-size: inherit;
    color: var(--action-primary);
    cursor: pointer;
    padding: 0;
  }

  .detail-path-link:hover {
    text-decoration: underline;
  }

  .col-expand-hint {
    font-size: var(--text-xs);
    color: var(--text-muted);
    opacity: 0;
    flex-shrink: 0;
    transition: opacity 0.15s;
  }

  .event-row:hover .col-expand-hint {
    opacity: 1;
  }

  .col-source-inline {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  /* ── Source (hover pill) ──────────────────────────────────────── */

  .col-source {
    position: absolute;
    right: var(--space-md);
    top: 50%;
    transform: translateY(-50%);
    font-size: var(--text-xs);
    color: var(--text-muted);
    background: var(--bg-secondary);
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.15s;
    z-index: 1;
  }

  .event-base:hover .col-source {
    opacity: 1;
  }

  /* ── Expanded section ─────────────────────────────────────────── */

  .event-expanded {
    padding: var(--space-sm) var(--space-md) var(--space-md) var(--space-lg);
    border-top: 1px solid var(--bg-tertiary);
  }

  .expanded-meta {
    display: flex;
    gap: var(--space-sm);
    font-size: var(--text-xs);
    color: var(--text-muted);
    margin-bottom: var(--space-sm);
  }

  .expanded-badge {
    padding: 1px 6px;
    border: 1px solid var(--bg-tertiary);
    border-radius: var(--radius-sm);
  }

  .expanded-json {
    font-size: var(--text-xs);
    padding: var(--space-sm);
    background: var(--bg-secondary);
    border-radius: var(--radius-sm);
    overflow-x: auto;
    white-space: pre-wrap;
    max-height: 300px;
    overflow-y: auto;
    margin: 0;
  }
</style>
