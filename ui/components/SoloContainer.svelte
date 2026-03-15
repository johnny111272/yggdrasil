<script lang="ts">
  import type { Snippet } from "svelte";
  import { onMount } from "svelte";

  interface Props {
    appName: string;
    children?: Snippet;
  }

  let { appName, children }: Props = $props();

  let uiFontSize = $state(14);
  let contentFontSize = $state(14);

  onMount(() => {
    const storedUI = localStorage.getItem(`solo-${appName}-ui-font-size`);
    const storedContent = localStorage.getItem(`solo-${appName}-content-font-size`);
    if (storedUI) uiFontSize = parseInt(storedUI, 10);
    if (storedContent) contentFontSize = parseInt(storedContent, 10);
  });

  function persistUI(size: number) {
    localStorage.setItem(`solo-${appName}-ui-font-size`, String(size));
  }

  function persistContent(size: number) {
    localStorage.setItem(`solo-${appName}-content-font-size`, String(size));
  }

  $effect(() => { persistUI(uiFontSize); });
  $effect(() => { persistContent(contentFontSize); });

  let containerStyle = $derived(
    `--ui-font-size: ${uiFontSize}px; --content-font-size: ${contentFontSize}px`
  );
</script>

<div class="solo-container" style={containerStyle}>
  {#if children}
    {@render children()}
  {/if}
</div>

<style>
  .solo-container {
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
</style>
