<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    value?: string;
    disabled?: boolean;
    onchange?: (e: Event) => void;
    children?: Snippet;
  }

  let { value = $bindable(""), disabled = false, onchange, children }: Props = $props();
</script>

<select class="select" bind:value {disabled} {onchange}>
  {#if children}
    {@render children()}
  {/if}
</select>

<style>
  .select {
    padding: var(--space-md);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family: var(--font-body);
    font-size: var(--text-sm);
    cursor: pointer;
  }

  .select:focus {
    outline: none;
    border-color: var(--action-primary);
  }

  .select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
