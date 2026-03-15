<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    variant?: "primary" | "special" | "neutral" | "ghost";
    size?: "sm" | "md";
    active?: boolean;
    disabled?: boolean;
    title?: string;
    onclick?: () => void;
    children?: Snippet;
  }

  let { variant = "neutral", size = "md", active = false, disabled = false, title, onclick, children }: Props = $props();
</script>

<button class="btn btn-{variant}" class:btn-sm={size === "sm"} class:active {disabled} {title} {onclick}>
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
    transition: background var(--transition-normal);
  }

  .btn-sm {
    padding: var(--space-xs) var(--space-md);
    font-size: var(--text-xs);
  }

  .btn.active {
    background: var(--action-primary);
    color: var(--text-primary);
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
