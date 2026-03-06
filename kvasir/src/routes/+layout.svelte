<script lang="ts">
  import "@introspection/ui/css/tokens.css";
  import "@introspection/ui/css/base.css";

  import type { Snippet } from "svelte";

  interface Props {
    children?: Snippet;
  }

  let { children }: Props = $props();
</script>

{#if children}
  {@render children()}
{/if}
