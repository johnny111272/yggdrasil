<script lang="ts">
  interface Props {
    checked: boolean;
    label?: string;
    disabled?: boolean;
  }

  let { checked = $bindable(false), label, disabled = false }: Props = $props();
</script>

<label class="checkbox" class:disabled>
  <input type="checkbox" bind:checked {disabled} />
  {#if label}<span class="checkbox-label">{label}</span>{/if}
</label>

<style>
  .checkbox {
    display: inline-flex;
    align-items: center;
    gap: var(--space-sm);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--text-secondary);
    cursor: pointer;
    user-select: none;
  }

  .checkbox.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .checkbox input {
    accent-color: var(--action-primary);
    cursor: inherit;
  }
</style>
