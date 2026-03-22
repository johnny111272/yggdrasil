<script lang="ts">
  import type { Snippet } from "svelte";
  import Badge from "./Badge.svelte";

  interface Props {
    title: string;
    badgeCount?: number;
    badgeSeverity?: "blocked" | "error" | "warning" | "success" | "neutral";
    open?: boolean;
    children?: Snippet;
  }

  let { title, badgeCount, badgeSeverity = "neutral", open = false, children }: Props = $props();
</script>

<details class="collapsible" {open}>
  <summary>
    <span class="collapsible-title">{title}</span>
    {#if badgeCount !== undefined}
      <Badge count={badgeCount} severity={badgeSeverity} />
    {/if}
  </summary>
  <div class="collapsible-content">
    {#if children}
      {@render children()}
    {/if}
  </div>
</details>

<style>
  .collapsible {
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
  }

  .collapsible summary {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-lg) var(--space-xl);
    cursor: pointer;
    user-select: none;
  }

  .collapsible summary:hover {
    background: var(--bg-hover);
  }

  .collapsible-title {
    font-family: var(--font-mono);
  }

  .collapsible-content {
    border-top: 1px solid var(--border-default);
  }
</style>
