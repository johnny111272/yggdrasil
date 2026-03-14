<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    header?: string;
    children?: Snippet;
  }

  let { header, children }: Props = $props();
</script>

<div class="panel">
  {#if header}
    <div class="panel-header">{header}</div>
  {/if}
  <div class="panel-content">
    {#if children}
      {@render children()}
    {/if}
  </div>
</div>

<style>
  .panel {
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    overflow: auto;
  }

  .panel-header {
    padding: var(--space-lg) var(--space-xl);
    border-bottom: 1px solid var(--border-default);
    font-weight: 600;
  }

  .panel-content {
    padding: var(--space-xl);
  }
</style>
