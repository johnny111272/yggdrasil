<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children?: Snippet;
  }

  let { children }: Props = $props();
</script>

<div class="ygg-shell">
  {#if children}
    {@render children()}
  {/if}
</div>

<style>
  .ygg-shell {
    height: 100%;
    overflow: hidden;
  }
</style>
