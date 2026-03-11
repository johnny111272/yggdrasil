<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    onclick?: () => void;
    children?: Snippet;
  }

  let { onclick, children }: Props = $props();

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") onclick?.();
  }
</script>

<li
  class="list-item"
  {onclick}
  onkeydown={handleKeydown}
  role="button"
  tabindex="0"
>
  {#if children}
    {@render children()}
  {/if}
</li>

<style>
  .list-item {
    padding: var(--space-md) var(--space-xl);
    border-bottom: 1px solid var(--border-subtle);
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    display: flex;
    gap: var(--space-md);
    flex-wrap: wrap;
  }

  .list-item:hover {
    background: var(--bg-hover);
  }

  .list-item:last-child {
    border-bottom: none;
  }
</style>
