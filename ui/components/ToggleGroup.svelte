<script lang="ts">
  interface ToggleOption {
    value: string;
    label: string;
    icon?: string;
  }

  interface Props {
    options: ToggleOption[];
    selected?: string;
    multi?: boolean;
    selectedSet?: Set<string>;
    highlightValue?: string;
    onToggle?: (value: string) => void;
  }

  let {
    options,
    selected = $bindable(""),
    multi = false,
    selectedSet,
    highlightValue,
    onToggle,
  }: Props = $props();

  function handleClick(value: string) {
    if (onToggle) {
      onToggle(value);
    } else if (!multi) {
      selected = value;
    }
  }

  function isActive(value: string): boolean {
    if (multi && selectedSet) return selectedSet.has(value);
    return value === selected;
  }
</script>

<div class="toggle-group">
  {#each options as opt}
    <button
      class="toggle-btn"
      class:active={isActive(opt.value)}
      class:highlighted={highlightValue === opt.value}
      onclick={() => handleClick(opt.value)}
    >
      {#if opt.icon}<span class="toggle-icon">{opt.icon}</span>{/if}
      {opt.label}
    </button>
  {/each}
</div>

<style>
  .toggle-group {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    flex-wrap: wrap;
  }

  .toggle-btn {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    padding: var(--space-2xs) var(--space-lg);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-full);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    white-space: nowrap;
    transition: var(--transition-fast);
  }

  .toggle-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .toggle-btn.active {
    background: var(--action-primary);
    color: var(--text-primary);
    border-color: var(--action-primary);
  }

  .toggle-btn.highlighted {
    border: 2px solid var(--action-primary);
  }

  .toggle-icon {
    margin-right: var(--space-2xs);
  }
</style>
