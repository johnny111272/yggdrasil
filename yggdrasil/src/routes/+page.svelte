<script lang="ts">
  import HlidskjalfView from "$hlidskjalf/HlidskjalfView.svelte";
  import SvalinnView from "$svalinn/SvalinnView.svelte";
  import KvasirView from "$kvasir/KvasirView.svelte";
  import RatatoskrView from "$ratatoskr/RatatoskrView.svelte";
  import { YggContainer, Button } from "@yggdrasil/ui";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let activeTab = $state("hlidskjalf");
  let mounted = $state(new Set(["hlidskjalf"]));
  let openFilePath: string | null = $state(null);
  let openFileLine: number | null = $state(null);

  function selectTab(id: string) {
    mounted = new Set([...mounted, id]);
    activeTab = id;
  }

  function clearDormant() {
    mounted = new Set(["hlidskjalf"]);
    activeTab = "hlidskjalf";
    openFilePath = null;
    openFileLine = null;
  }

  const tabs = [
    { id: "hlidskjalf", label: "Hlidskjalf", desc: "Agent Monitor" },
    { id: "svalinn",    label: "Svalinn",    desc: "Code Quality" },
    { id: "kvasir",     label: "Kvasir",     desc: "Workspace Inspector" },
    { id: "ratatoskr",  label: "Ratatoskr",  desc: "Graph Viewer" },
  ];

  let hasDormant = $derived(mounted.size > 1);

  onMount(() => {
    let unlisten: (() => void) | undefined;

    invoke<string | null>("get_pending_file").then((pending) => {
      if (pending) {
        selectTab("kvasir");
        openFilePath = pending;
      }
    });

    listen<string>("open-file", (event) => {
      selectTab("kvasir");
      openFilePath = event.payload;
    }).then((fn) => { unlisten = fn; });

    return () => { unlisten?.(); };
  });
</script>

{#snippet appTabBar()}
  <nav class="app-tabs">
    {#each tabs as tab}
      <Button
        variant="ghost"
        size="sm"
        class="app-tab {mounted.has(tab.id) && activeTab !== tab.id ? 'mounted' : ''}"
        active={activeTab === tab.id}
        onclick={() => selectTab(tab.id)}
        title="{tab.label} — {tab.desc}"
      >
        {tab.label}
      </Button>
    {/each}
    {#if hasDormant}
      <Button variant="ghost" size="sm" class="app-tab clear-tab" onclick={clearDormant} title="Clear dormant tabs">
        &times;
      </Button>
    {/if}
  </nav>
{/snippet}

<div class="shell">
  <div class="view-pane" class:active={activeTab === "hlidskjalf"}>
    <YggContainer>
      <HlidskjalfView storagePrefix="ygg" appTabs={appTabBar} commands={{
        start_monitor: "hlid_start_monitor",
        speak: "hlid_speak",
        open_in_editor: "hlid_open_in_editor",
      }} onOpenFile={(path, line) => {
        openFilePath = path;
        openFileLine = line ?? null;
        selectTab("kvasir");
      }} />
    </YggContainer>
  </div>
  {#if mounted.has("svalinn")}
    <div class="view-pane" class:active={activeTab === "svalinn"}>
      <YggContainer>
        <SvalinnView storagePrefix="ygg" appTabs={appTabBar} commands={{
          scan_directory: "sval_scan_directory",
          list_qa_tree: "sval_list_qa_tree",
          open_in_editor: "sval_open_in_editor",
          run_saga: "sval_run_saga",
        }} />
      </YggContainer>
    </div>
  {/if}
  {#if mounted.has("kvasir")}
    <div class="view-pane" class:active={activeTab === "kvasir"}>
      <YggContainer>
        <KvasirView storagePrefix="ygg" appTabs={appTabBar} commands={{
          list_directory: "kvas_list_directory",
          read_file: "kvas_read_file",
          open_in_editor: "kvas_open_in_editor",
          convert_to_all_formats: "kvas_convert_to_all_formats",
          detect_data_format: "kvas_detect_data_format",
          read_jsonl_info: "kvas_read_jsonl_info",
          read_jsonl_entry: "kvas_read_jsonl_entry",
          export_entry_as: "kvas_export_entry_as",
          read_table: "kvas_read_table",
          export_table_csv: "kvas_export_table_csv",
        }} openFile={openFilePath} openLine={openFileLine} />
      </YggContainer>
    </div>
  {/if}
  {#if mounted.has("ratatoskr")}
    <div class="view-pane" class:active={activeTab === "ratatoskr"}>
      <YggContainer>
        <RatatoskrView storagePrefix="ygg" appTabs={appTabBar} commands={{
          list_directory: "rata_list_directory",
          load_graph: "rata_load_graph",
          save_graph: "rata_save_graph",
          get_graph_stats: "rata_get_graph_stats",
          generate_sample_graph: "rata_generate_sample_graph",
        }} />
      </YggContainer>
    </div>
  {/if}
</div>

<style>
  .shell {
    height: 100vh;
    position: relative;
    background: var(--bg-primary);
    color: var(--text-primary);
    overflow: hidden;
  }

  .view-pane {
    position: absolute;
    inset: 0;
    overflow: hidden;
    visibility: hidden;
    pointer-events: none;
  }

  .view-pane.active {
    visibility: visible;
    pointer-events: auto;
  }

  /* ── App tabs (rendered inside ContainerLayout via appTabs snippet) ── */

  :global(.app-tabs) {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-md);
  }

  :global(.app-tab) {
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 500;
    cursor: pointer;
    padding: var(--space-2xs) var(--space-lg);
    transition: var(--transition-fast);
  }

  :global(.app-tab:hover) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  :global(.app-tab.mounted) {
    color: var(--text-primary);
  }

  :global(.app-tab.active) {
    background: var(--action-primary);
    color: var(--text-primary);
    font-weight: 700;
  }

  :global(.clear-tab) {
    margin-left: auto;
    font-size: var(--text-sm);
  }
</style>
