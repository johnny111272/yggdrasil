<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children?: Snippet;
  }

  let { children }: Props = $props();
</script>

<div class="solo-shell">
  {#if children}
    {@render children()}
  {/if}
</div>

<style>
  .solo-shell {
    height: 100vh;
    overflow: hidden;
  }
</style>
