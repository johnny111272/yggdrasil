<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
    onDismiss?: () => void;
  }

  let { children, onDismiss }: Props = $props();
</script>

<section class="error-banner">
  <div class="error-content">
    {@render children()}
  </div>
  {#if onDismiss}
    <button class="error-dismiss" onclick={onDismiss}>×</button>
  {/if}
</section>

<style>
  .error-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--severity-error);
    color: var(--text-primary);
    padding: var(--space-lg) var(--space-xl);
    border-radius: var(--radius-sm);
  }

  .error-content {
    flex: 1;
  }

  .error-dismiss {
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: var(--text-lg);
    cursor: pointer;
    padding: 0;
    line-height: 1;
    opacity: 0.7;
  }

  .error-dismiss:hover {
    opacity: 1;
  }
</style>
