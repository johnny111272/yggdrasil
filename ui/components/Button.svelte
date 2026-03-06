<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    variant?: "primary" | "special" | "neutral" | "ghost";
    disabled?: boolean;
    onclick?: () => void;
    children?: Snippet;
  }

  let { variant = "neutral", disabled = false, onclick, children }: Props = $props();
</script>

<button class="btn btn-{variant}" {disabled} {onclick}>
  {#if children}
    {@render children()}
  {/if}
</button>

<style>
  .btn {
    padding: var(--space-md) var(--space-xl);
    border: none;
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: var(--text-sm);
    color: var(--text-primary);
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-neutral {
    background: var(--action-neutral);
  }

  .btn-neutral:hover:not(:disabled) {
    background: var(--action-neutral-hover);
  }

  .btn-primary {
    background: var(--action-primary);
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--action-primary-hover);
  }

  .btn-special {
    background: var(--action-special);
  }

  .btn-special:hover:not(:disabled) {
    background: var(--action-special-hover);
  }

  .btn-ghost {
    background: transparent;
    padding: var(--space-md);
  }

  .btn-ghost:hover:not(:disabled) {
    background: var(--bg-hover);
  }
</style>
