<script lang="ts">
  // ── Types ──────────────────────────────────────────────────────────

  interface Location {
    file: string;
    line?: number;
    lines?: number[];
  }

  interface CheckGroup {
    tool: string;
    code: string;
    message: string;
    severity: string;
    count: number;
    file_count: number;
    signal: string;
    direction: string;
    canary: string;
    locations: Location[];
  }

  function format_location(location: Location): string[] {
    if (location.line !== undefined) {
      return [`${location.file}:${location.line}`];
    }
    if (location.lines) {
      return [location.file, `[${location.lines.join(",")}]`];
    }
    return [location.file];
  }

  interface QualityPayload {
    total: number;
    check_types: number;
    groups: CheckGroup[];
  }

  // ── Props ──────────────────────────────────────────────────────────

  interface Props {
    payload: QualityPayload;
    workspace: string;
    timestamp: string;
  }

  let { payload, workspace, timestamp }: Props = $props();

  // ── Helpers ────────────────────────────────────────────────────────

  function severity_class(severity: string): string {
    switch (severity) {
      case "error":
      case "blocked":
        return "severity-error";
      case "warning":
        return "severity-warning";
      default:
        return "severity-success";
    }
  }

  function files_word(count: number): string {
    return count === 1 ? "file" : "files";
  }
</script>

<div class="quality-report">
  <!-- Banner -->
  <div class="banner">
    <div class="banner-bar"></div>
    <div class="banner-content">
      <span class="banner-badge">SYN</span>
      <span class="banner-stats">
        <strong>{payload.total}</strong> violations across
        <strong>{payload.check_types}</strong> check types
      </span>
      <span class="banner-meta">{workspace} &middot; {timestamp}</span>
    </div>
    <div class="banner-bar"></div>
  </div>

  <!-- Groups -->
  {#each payload.groups as group}
    <div class="group {severity_class(group.severity)}">
      <!-- Group header -->
      <div class="group-header">
        <span class="group-code">{group.code}</span>
        <span class="group-severity">{group.severity}</span>
        <span class="group-count">
          <strong>{group.count}</strong> in {group.file_count}
          {files_word(group.file_count)}
        </span>
      </div>

      <!-- Educational content -->
      <div class="group-education">
        {#if group.signal}
          <div class="edu-row">
            <span class="edu-label signal-label">Signal</span>
            <span class="edu-text">{group.signal}</span>
          </div>
        {/if}
        {#if group.direction}
          <div class="edu-row">
            <span class="edu-label direction-label">Direction</span>
            <span class="edu-text">{group.direction}</span>
          </div>
        {/if}
        {#if group.canary}
          <div class="edu-row">
            <span class="edu-label canary-label">Canary</span>
            <span class="edu-text">{group.canary}</span>
          </div>
        {/if}
      </div>

      <!-- Locations -->
      <div class="group-locations">
        {#each group.locations as loc}
          {#each format_location(loc) as entry}
            <div class="location">{entry}</div>
          {/each}
        {/each}
      </div>
    </div>
  {/each}
</div>

<style>
  .quality-report {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    padding: var(--space-md) 0;
  }

  /* ── Banner ──────────────────────────────────────────────────────── */

  .banner {
    margin-bottom: var(--space-lg);
  }

  .banner-bar {
    height: 2px;
    background: var(--severity-error);
  }

  .banner-content {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-sm) var(--space-md);
  }

  .banner-badge {
    font-weight: 700;
    font-size: var(--text-xs);
    padding: 2px 10px;
    background: var(--severity-error);
    color: var(--bg-primary);
    border-radius: var(--radius-sm);
    letter-spacing: 0.5px;
  }

  .banner-stats {
    color: var(--text-primary);
  }

  .banner-stats strong {
    color: var(--text-primary);
    font-weight: 700;
  }

  .banner-meta {
    margin-left: auto;
    color: var(--text-secondary);
    font-size: var(--text-xs);
  }

  /* ── Group ───────────────────────────────────────────────────────── */

  .group {
    border-left: 3px solid var(--bg-tertiary);
    margin: var(--space-md) 0;
    padding: 0 0 0 var(--space-md);
  }

  .group.severity-error {
    border-left-color: var(--severity-error);
  }

  .group.severity-warning {
    border-left-color: var(--severity-warning);
  }

  .group.severity-success {
    border-left-color: var(--severity-success);
  }

  /* ── Group header ────────────────────────────────────────────────── */

  .group-header {
    display: flex;
    align-items: baseline;
    gap: var(--space-md);
    padding: var(--space-xs) 0;
  }

  .group-code {
    font-weight: 700;
    color: var(--text-primary);
  }

  .severity-error .group-code {
    color: var(--severity-error);
  }

  .severity-warning .group-code {
    color: var(--severity-warning);
  }

  .group-severity {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .group-count {
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  .group-count strong {
    color: var(--text-primary);
  }

  /* ── Educational content ─────────────────────────────────────────── */

  .group-education {
    padding: var(--space-xs) 0 var(--space-sm) 0;
  }

  .edu-row {
    display: flex;
    gap: var(--space-md);
    padding: 2px 0;
    line-height: 1.5;
  }

  .edu-label {
    flex-shrink: 0;
    font-weight: 700;
    font-size: var(--text-xs);
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    align-self: flex-start;
    margin-top: 2px;
  }

  .signal-label {
    color: var(--severity-error);
    border: 1px solid var(--severity-error);
  }

  .direction-label {
    color: var(--action-primary);
    border: 1px solid var(--action-primary);
  }

  .canary-label {
    color: var(--severity-warning);
    border: 1px solid var(--severity-warning);
  }

  .edu-text {
    color: var(--text-muted);
  }

  /* ── Locations ───────────────────────────────────────────────────── */

  .group-locations {
    padding: var(--space-xs) 0 var(--space-sm) 0;
    border-top: 1px solid var(--bg-tertiary);
  }

  .location {
    color: var(--text-secondary);
    font-size: var(--text-xs);
    padding: 1px 0;
  }

  .location:hover {
    color: var(--text-primary);
  }
</style>
