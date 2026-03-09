<script lang="ts">
  import KvasirView from "$lib/KvasirView.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let openFile: string | null = $state(null);

  onMount(async () => {
    const pending = await invoke<string | null>("get_pending_file");
    if (pending) openFile = pending;

    const unlisten = await listen<string>("open-file", (event) => {
      openFile = event.payload;
    });

    return unlisten;
  });
</script>

<KvasirView {openFile} />
