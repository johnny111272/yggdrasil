<script lang="ts">
  import { onMount } from "svelte";

  interface Props {
    orientation?: "vertical" | "horizontal";
  }

  let { orientation = "vertical" }: Props = $props();

  const themes = [
    { id: "darkly",    glyph: "D", label: "Darkly — neutral dark" },
    { id: "light",     glyph: "L", label: "Light — bright environments" },
    { id: "warm-dark", glyph: "W", label: "Warm — evening, reduced blue" },
    { id: "cool-dark", glyph: "C", label: "Cool — focus work, crisp" },
  ] as const;

  const STORAGE_KEY = "yggdrasil-theme";

  let current = $state("darkly");

  function apply(themeId: string) {
    current = themeId;
    if (themeId === "darkly") {
      delete document.body.dataset.theme;
    } else {
      document.body.dataset.theme = themeId;
    }
    localStorage.setItem(STORAGE_KEY, themeId);
  }

  onMount(() => {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) apply(saved);
  });
</script>

<div class="theme-switcher" class:horizontal={orientation === "horizontal"}>
  {#each themes as theme}
    <button
      class="theme-btn"
      class:active={current === theme.id}
      data-theme-id={theme.id}
      onclick={() => apply(theme.id)}
      title={theme.label}
    >
      {theme.glyph}
    </button>
  {/each}
</div>

<style>
  .theme-switcher {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }

  .theme-switcher.horizontal {
    flex-direction: row;
    gap: var(--space-xs);
  }

  .theme-btn {
    width: 20px;
    height: 20px;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: 0.65rem;
    font-weight: 600;
    cursor: pointer;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.15s ease, border-color 0.15s ease;
  }

  .theme-btn:hover {
    color: var(--text-primary);
    border-color: var(--border-default);
  }

  .theme-btn.active {
    color: var(--text-primary);
    border-color: var(--action-primary);
  }
</style>
