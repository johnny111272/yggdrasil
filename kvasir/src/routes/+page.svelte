<script lang="ts">
  import KvasirView from "$lib/KvasirView.svelte";
  import { SoloContainer } from "@yggdrasil/ui";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let openFile: string | null = $state(null);

  onMount(() => {
    let unlisten: (() => void) | undefined;

    invoke<string | null>("get_pending_file").then((pending) => {
      if (pending) openFile = pending;
    });

    listen<string>("open-file", (event) => {
      openFile = event.payload;
    }).then((fn) => { unlisten = fn; });

    return () => { unlisten?.(); };
  });
</script>

<SoloContainer>
  <KvasirView {openFile} />
</SoloContainer>
