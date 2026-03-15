<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    appName: string;
    subtitle: string;
    right?: Snippet;
    children?: Snippet;
  }

  let { appName, subtitle, right, children }: Props = $props();
</script>

<header class="app-header">
  <div class="header-left">
    <h1 class="title">
      <span class="app-name">{appName}</span>
      <span class="sep">::</span>
      <span class="subtitle">{subtitle}</span>
    </h1>
    {#if children}
      {@render children()}
    {/if}
  </div>
  {#if right}
    <div class="header-right">
      {@render right()}
    </div>
  {/if}
</header>

<style>
  .app-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-sm) var(--space-lg);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: var(--space-lg);
  }

  .title {
    font-size: var(--text-sm);
    margin: 0;
    white-space: nowrap;
  }

  .app-name {
    font-weight: var(--fw-extrabold);
    letter-spacing: 0.12em;
  }

  .sep {
    font-weight: var(--fw-light);
    opacity: 0.5;
    margin: 0 var(--space-xs);
  }

  .subtitle {
    font-size: var(--text-sm);
    font-weight: var(--fw-light);
    color: var(--text-secondary);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: var(--space-lg);
  }
</style>
