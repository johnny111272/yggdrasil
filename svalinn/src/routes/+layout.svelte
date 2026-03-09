<script lang="ts">
  import "@yggdrasil/ui/css/tokens.css";
  import "@yggdrasil/ui/css/base.css";

  import type { Snippet } from "svelte";

  interface Props {
    children?: Snippet;
  }

  let { children }: Props = $props();
</script>

{#if children}
  {@render children()}
{/if}
