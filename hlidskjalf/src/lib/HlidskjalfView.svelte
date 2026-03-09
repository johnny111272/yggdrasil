<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  import GleipnirReport from "./GleipnirReport.svelte";

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
    type: string;
    priority: string;
    workspace: string;
    detail?: string;
    speech?: string;
    payload?: {
      type?: string;
      [key: string]: unknown;
    };
  }

  // ── Priority helpers ────────────────────────────────────────────────

  function priorityNumeric(p: string): number {
    switch (p) {
      case "critical": return 4;
      case "high": return 3;
      case "normal": return 2;
      case "low": return 1;
      case "trace": return 0;
      default: return 0;
    }
  }

  function priorityColor(p: string): string {
    switch (p) {
      case "critical": return "var(--severity-error)";
      case "high": return "var(--severity-warning)";
      case "normal": return "var(--severity-info)";
      case "low": return "var(--text-secondary)";
      case "trace": return "var(--text-muted)";
      default: return "var(--text-secondary)";
    }
  }

  function typeIcon(t: string): string {
    switch (t) {
      case "alert": return "\u26A0";
      case "report": return "\uD83D\uDCCA";
      case "canary": return "\uD83D\uDC26";
      case "notify": return "\u2139";
      default: return "\u2753";
    }
  }

  // ── State ──────────────────────────────────────────────────────────

  let datagrams: Datagram[] = $state([]);
  let connected = $state(false);
  let autoScroll = $state(true);
  let filterType = $state("all");
  let filterPriorityMin = $state(0); // trace and above (show everything)
  let speechMinPriority = $state(3); // high+ by default
  let feedElement: HTMLElement | undefined = $state();

  // ── Derived ────────────────────────────────────────────────────────

  let filteredDatagrams = $derived(
    datagrams.filter((ev) => {
      if (filterType !== "all" && ev.type !== filterType) return false;
      if (priorityNumeric(ev.priority) < filterPriorityMin) return false;
      return true;
    }),
  );

  let datagramTypes = $derived([
    ...new Set(datagrams.map((ev) => ev.type)),
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
      datagrams = [...datagrams.filter(d => d.ts > cutoff), ev];

      // Speech fires when ev.speech is present AND priority >= speechMinPriority
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
    } catch (err) {
      console.error("Failed to start listener:", err);
    }

    return () => {
      unlisten();
    };
  });

  // ── Helpers ────────────────────────────────────────────────────────

  function formatTime(ts: number): string {
    const date = new Date(ts * 1000);
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
      class:active={filterType === "all"}
      onclick={() => (filterType = "all")}
    >
      all
    </button>
    {#each datagramTypes as t}
      <button
        class="filter-btn"
        class:active={filterType === t}
        onclick={() => (filterType = t)}
      >
        {typeIcon(t)}
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
      {#each filteredDatagrams as ev, idx}
        <div
          class="event-row"
          class:critical={ev.priority === "critical"}
          class:high={ev.priority === "high"}
          class:canary={ev.type === "canary"}
        >
          {#if ev.type === "canary"}
            <!-- Compact canary rendering -->
            <div class="event-meta canary-row">
              <span class="event-time">{formatTime(ev.timestamp)}</span>
              <span class="event-type-icon">{typeIcon(ev.type)}</span>
              <span class="event-source">{ev.source}</span>
              <span class="event-workspace">{ev.workspace}</span>
            </div>
          {:else}
            <div class="event-meta">
              <span class="event-time">{formatTime(ev.timestamp)}</span>
              <span
                class="event-priority"
                style="color: {priorityColor(ev.priority)}"
              >
                {ev.priority.toUpperCase()}
              </span>
              <span class="event-type-icon">{typeIcon(ev.type)}</span>
              <span class="event-source">{ev.source}</span>
              <span class="event-workspace">{ev.workspace}</span>
              {#if ev.detail && !ev.payload}
                <span class="event-detail">{ev.detail}</span>
              {/if}
            </div>

            <!-- Gleipnir/syn report payload -->
            {#if ev.payload?.type === "gleipnir_report" || ev.payload?.type === "syn_report"}
              <div class="event-payload">
                <GleipnirReport
                  payload={ev.payload}
                  workspace={ev.workspace}
                  timestamp={formatTime(ev.timestamp)}
                />
              </div>
            {:else if ev.payload}
              <details class="event-context">
                <summary>payload{ev.payload.type ? ` (${ev.payload.type})` : ""}</summary>
                <pre>{JSON.stringify(ev.payload, null, 2)}</pre>
              </details>
            {/if}
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
    background: var(--severity-info);
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
    border-color: var(--severity-info);
    color: var(--severity-info);
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

  /* ── Event row ───────────────────────────────────────────────────── */

  .event-row {
    padding: var(--space-xs) 0;
    border-bottom: 1px solid var(--bg-tertiary);
    font-size: var(--text-sm);
  }

  .event-row.critical {
    background: rgba(255, 51, 51, 0.08);
  }

  .event-row.high {
    background: rgba(255, 153, 0, 0.05);
  }

  .event-row.canary {
    opacity: 0.6;
  }

  .event-meta {
    display: flex;
    gap: var(--space-sm);
    align-items: baseline;
  }

  .canary-row {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .event-time {
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
    min-width: 80px;
  }

  .event-priority {
    font-weight: 700;
    font-size: var(--text-xs);
    min-width: 65px;
  }

  .event-type-icon {
    min-width: 20px;
    text-align: center;
  }

  .event-source {
    color: var(--text-muted);
    min-width: 100px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .event-workspace {
    color: var(--text-secondary);
    min-width: 100px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .event-detail {
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .event-payload {
    padding: var(--space-xs) 0 var(--space-sm) 0;
  }

  .event-context {
    margin-top: var(--space-xs);
  }

  .event-context summary {
    font-size: var(--text-xs);
    color: var(--action-primary);
    cursor: pointer;
  }

  .event-context pre {
    font-size: var(--text-xs);
    padding: var(--space-sm);
    background: var(--bg-secondary);
    border-radius: var(--radius-sm);
    overflow-x: auto;
    white-space: pre-wrap;
    max-height: 300px;
    overflow-y: auto;
  }
</style>
