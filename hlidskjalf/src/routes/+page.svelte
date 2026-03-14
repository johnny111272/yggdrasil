<script lang="ts">
  import HlidskjalfView from "$lib/HlidskjalfView.svelte";
  import { invoke } from "@tauri-apps/api/core";

  function openFile(path: string, line?: number) {
    invoke("open_default", { path });
  }
</script>

<HlidskjalfView onOpenFile={openFile} />
