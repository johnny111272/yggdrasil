<script lang="ts">
  import HlidskjalfView from "$lib/HlidskjalfView.svelte";
  import { openPath } from "@tauri-apps/plugin-opener";

  function openFile(path: string, _line?: number) {
    openPath(path);
  }
</script>

<HlidskjalfView onOpenFile={openFile} />
