<script lang="ts">
  import type { FontFamily } from "../types/container-types.js";
  import ThemeSwitcher from "./ThemeSwitcher.svelte";
  import FontControls from "./FontControls.svelte";

  interface Props {
    collapsed?: boolean;
    contentFontSize?: number;
    fontFamily?: FontFamily;
  }

  let {
    collapsed = $bindable(false),
    contentFontSize = $bindable(14),
    fontFamily = $bindable("mono" as FontFamily),
  }: Props = $props();
</script>

{#if collapsed}
<div class="settings-toggle-bar" onclick={() => collapsed = false} onkeydown={(e) => e.key === 'Enter' && (collapsed = false)} role="button" tabindex="0" title="Show settings">
</div>
{:else}
  <div class="settings-bar" onclick={(e) => { if (e.target === e.currentTarget) collapsed = !collapsed; }} onkeydown={(e) => { if (e.key === 'Enter' && e.target === e.currentTarget) collapsed = !collapsed; }} role="button" tabindex="0" title="Click to hide settings">
    <ThemeSwitcher orientation="horizontal" />
    <div class="separator"></div>
    <FontControls bind:fontSize={contentFontSize} bind:fontFamily />
  </div>
{/if}

<style>
  .settings-toggle-bar {
    height: 4px;
    width: 100%;
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-control);
    cursor: pointer;
    flex-shrink: 0;
    transition: background var(--transition-fast);
  }

  .settings-toggle-bar:hover {
    background: var(--bg-hover);
    border-bottom-color: var(--border-default);
  }

  .settings-bar {
    display: flex;
    align-items: center;
    padding: var(--space-xs) var(--space-md);
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-control);
    cursor: pointer;
    flex-shrink: 0;
    gap: var(--space-md);
    flex-wrap: wrap;
  }

  .separator {
    width: 1px;
    height: 16px;
    background: var(--border-default);
    flex-shrink: 0;
  }
</style>
