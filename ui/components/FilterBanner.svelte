<script lang="ts">
  interface Props {
    label: string;
    value: string;
    onClear?: () => void;
  }

  let { label, value, onClear }: Props = $props();
</script>

<section class="filter-banner">
  <span>{label}: <strong>{value}</strong></span>
  {#if onClear}
    <button onclick={onClear}>Clear filter</button>
  {/if}
</section>

<style>
  .filter-banner {
    background: var(--bg-selected);
    padding: var(--space-lg) var(--space-xl);
    border-radius: var(--radius-sm);
    margin-bottom: var(--space-xl);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .filter-banner button {
    background: var(--action-neutral-hover);
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    padding: var(--space-xs) var(--space-lg);
    cursor: pointer;
    font-size: var(--text-sm);
  }

  .filter-banner button:hover {
    background: var(--border-default);
  }
</style>
