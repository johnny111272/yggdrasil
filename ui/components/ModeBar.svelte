<script lang="ts">
  import type { ModeOption } from "../types/container-types.js";

  interface Props {
    options: ModeOption[];
    selected?: string;
    onSelect?: (value: string) => void;
  }

  let {
    options,
    selected = $bindable(""),
    onSelect,
  }: Props = $props();

  function cycle() {
    if (options.length === 0) return;
    const i = options.findIndex(o => o.value === selected);
    const next = options[(i + 1) % options.length];
    selected = next.value;
    onSelect?.(next.value);
  }

  let current = $derived(options.find(o => o.value === selected));
</script>

{#if options.length > 0}
  <div
    class="mode-bar"
    onclick={cycle}
    onkeydown={(e) => e.key === 'Enter' && cycle()}
    role="button"
    tabindex="0"
    title={current ? `${current.label} (click to cycle)` : "Cycle mode"}
  >
    <span class="mode-icon">{current?.icon ?? current?.label[0] ?? "?"}</span>
  </div>
{/if}

<style>
  .mode-bar {
    width: 28px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    border-right: 1px solid var(--border-subtle);
    background: var(--bg-control);
    color: var(--text-secondary);
    cursor: pointer;
    transition: var(--transition-fast);
  }

  .mode-bar:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .mode-icon {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 600;
    writing-mode: vertical-rl;
    text-orientation: mixed;
  }
</style>
