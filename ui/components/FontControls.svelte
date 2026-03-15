<script lang="ts">
  type FontFamily = "mono" | "dyslexie" | "sans" | "serif";

  interface Props {
    fontSize?: number;
    fontFamily?: FontFamily;
    fontFamilies?: FontFamily[];
    minSize?: number;
    maxSize?: number;
  }

  let {
    fontSize = $bindable(14),
    fontFamily = $bindable("mono" as FontFamily),
    fontFamilies = ["mono", "dyslexie", "sans", "serif"] as FontFamily[],
    minSize = 10,
    maxSize = 24,
  }: Props = $props();

  function sizeDown() { if (fontSize > minSize) fontSize--; }
  function sizeUp() { if (fontSize < maxSize) fontSize++; }

  function cycleFamily() {
    const i = fontFamilies.indexOf(fontFamily);
    fontFamily = fontFamilies[(i + 1) % fontFamilies.length];
  }
</script>

<div class="font-controls">
  <button class="font-btn" onclick={sizeDown} title="Decrease font size">A−</button>
  <span class="font-size-display">{fontSize}</span>
  <button class="font-btn" onclick={sizeUp} title="Increase font size">A+</button>
  <button class="font-btn family-btn" onclick={cycleFamily} title="Cycle font family">{fontFamily}</button>
</div>

<style>
  .font-controls {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
  }

  .font-btn {
    padding: var(--space-sm) var(--space-md);
    border: none;
    border-radius: var(--radius-sm);
    background: var(--action-neutral);
    color: var(--text-primary);
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .font-btn:hover {
    background: var(--action-neutral-hover);
  }

  .font-size-display {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--text-secondary);
    min-width: 1.5rem;
    text-align: center;
  }

  .family-btn {
    text-transform: lowercase;
  }
</style>
