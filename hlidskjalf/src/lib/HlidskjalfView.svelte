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
    },
  }: {
    commands?: { start_monitor: string; speak: string };
  } = $props();

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
      case "alert": return "\u26A0";
      case "quality": return "\uD83D\uDCCA";
      case "canary": return "\uD83D\uDC26";
      case "notify": return "\u2139";
      case "traffic": return "\uD83D\uDD04";
      default: return "\u2753";
    }
  }

  function kindLabel(kind: string): string {
    switch (kind) {
      case "alert": return "ALERT";
      case "quality": return "QUALITY";
      case "canary": return "CANARY";
      case "notify": return "NOTIFY";
      case "traffic": return "TRAFFIC";
      default: return k.toUpperCase();
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

  function workspaceColor(workspace: string): string {
    if (!workspace) return "var(--text-secondary)";
    if (!workspaceColorMap.has(workspace)) {
      const idx = workspaceColorMap.size % WORKSPACE_HUES.length;
      workspaceColorMap = new Map([...workspaceColorMap, [workspace, WORKSPACE_HUES[idx]]]);
    }
    return workspaceColorMap.get(workspace)!;
  }

  // ── State ──────────────────────────────────────────────────────────

  let datagrams: Datagram[] = $state([]);
  let connected = $state(false);
  let autoScroll = $state(true);
  let filterKind = $state("all");
  let filterPriorityMin = $state(0); // trace and above (show everything)
  let speechMinPriority = $state(3); // high+ by default
  let feedElement: HTMLElement | undefined = $state();
  let expandedRows: Set<number> = $state(new Set());

  function toggleRow(timestamp: number) {
    const next = new Set(expandedRows);
    if (next.has(timestamp)) next.delete(timestamp);
    else next.add(timestamp);
    expandedRows = next;
  }

  // ── Derived ────────────────────────────────────────────────────────

  let filteredDatagrams = $derived(
    datagrams.filter((ev) => {
      if (filterKind !== "all" && ev.kind !== filterKind) return false;
      if (priorityNumeric(ev.priority) < filterPriorityMin) return false;
      return true;
    }),
  );

  let datagramKinds = $derived([
    ...new Set(datagrams.map((ev) => ev.kind)),
  ]);

  let stats = $derived({
    total: datagrams.length,
    critical: datagrams.filter((ev) => ev.priority === "critical").length,
    high: datagrams.filter((ev) => ev.priority === "high").length,
  });

  // ── Lifecycle ──────────────────────────────────────────────────────

  onMount(async () => {
    const unlisten = await listen<Datagram>("datagram", (event) => {
      const ev = event.payload;
      const cutoff = Date.now() / 1000 - 3600;
      datagrams = [...datagrams.filter(d => d.timestamp > cutoff), ev];

      if (ev.speech && priorityNumeric(ev.priority) >= speechMinPriority) {
        invoke(commands.speak, { text: ev.speech });
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
      class:active={filterKind === "all"}
      onclick={() => (filterKind = "all")}
    >
      all
    </button>
    {#each datagramKinds as t}
      <button
        class="filter-btn"
        class:active={filterKind === t}
        onclick={() => (filterKind = t)}
      >
        {kindIcon(t)}
        {t}
      </button>
    {/each}
    <div class="filter-spacer"></div>
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
      {#each filteredDatagrams as ev}
        {@const isExpanded = expandedRows.has(ev.timestamp)}
        {@const hasPayload = !!ev.payload}
        {@const isCanary = ev.kind === "canary"}

        <div
          class="event-row {priorityClass(ev.priority)}"
          class:canary={isCanary}
          class:expanded={isExpanded}
        >
          <div
            class="event-base"
            class:canary-base={isCanary}
            class:clickable={hasPayload}
            onclick={() => { if (hasPayload) toggleRow(ev.timestamp); }}
            role={hasPayload ? "button" : undefined}
            tabindex={hasPayload ? 0 : undefined}
            onkeydown={(e) => { if (hasPayload && (e.key === "Enter" || e.key === " ")) { e.preventDefault(); toggleRow(ev.timestamp); } }}
          >
            <span class="col-time">{formatTime(ev.timestamp)}</span>
            <span class="col-kind-icon">{kindIcon(ev.kind)}</span>

            {#if isCanary}
              <span class="col-workspace" style="color: {workspaceColor(ev.workspace)}">{ev.workspace}</span>
            {:else}
              <span class="col-kind-label">{kindLabel(ev.kind)}</span>
              {#if ev.classifier}
                <span class="col-classifier">{ev.classifier}</span>
              {/if}
              <span class="col-workspace" style="color: {workspaceColor(ev.workspace)}">{ev.workspace}</span>
              {#if ev.detail}
                <span class="col-detail">{ev.detail}</span>
              {/if}
              {#if hasPayload && !isExpanded}
                <span class="col-expand-hint">+</span>
              {/if}
              <span class="col-source">{ev.source}</span>
            {/if}
          </div>

          {#if isExpanded && hasPayload}
            <div class="event-expanded">
              <div class="expanded-meta">
                <span class="expanded-badge">src: {ev.source}</span>
                <span class="expanded-badge">{ev.priority}</span>
                {#if ev.classifier}
                  <span class="expanded-badge">{ev.classifier}</span>
                {/if}
              </div>
              {#if ev.kind === "quality"}
                <QualityReport
                  payload={ev.payload}
                  workspace={ev.workspace}
                  timestamp={formatTime(ev.timestamp)}
                />
              {:else if ev.kind === "traffic"}
                <TrafficReport
                  payload={ev.payload}
                  workspace={ev.workspace}
                  timestamp={formatTime(ev.timestamp)}
                />
              {:else}
                <pre class="expanded-json">{JSON.stringify(ev.payload, null, 2)}</pre>
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

  .col-detail {
    color: var(--text-primary);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
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
