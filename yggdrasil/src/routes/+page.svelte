<script lang="ts">
  import HlidskjalfView from "$hlidskjalf/HlidskjalfView.svelte";
  import SvalinnView from "$svalinn/SvalinnView.svelte";
  import KvasirView from "$kvasir/KvasirView.svelte";
  import RatatoskrView from "$ratatoskr/RatatoskrView.svelte";
  import { ThemeSwitcher } from "@yggdrasil/ui";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let activeTab = $state("hlidskjalf");
  let mounted = $state(new Set(["hlidskjalf"]));
  let openFilePath: string | null = $state(null);

  function selectTab(id: string) {
    mounted = new Set([...mounted, id]);
    activeTab = id;
  }

  function clearDormant() {
    mounted = new Set(["hlidskjalf"]);
    activeTab = "hlidskjalf";
    openFilePath = null;
  }

  const tabs = [
    { id: "hlidskjalf", label: "Hlidskjalf", desc: "Agent Monitor" },
    { id: "svalinn",    label: "Svalinn",    desc: "Code Quality" },
    { id: "kvasir",     label: "Kvasir",     desc: "Workspace Inspector" },
    { id: "ratatoskr",  label: "Ratatoskr",  desc: "Graph Viewer" },
  ];

  let hasDormant = $derived(mounted.size > 1);

  onMount(async () => {
    const pending = await invoke<string | null>("get_pending_file");
    if (pending) {
      selectTab("kvasir");
      openFilePath = pending;
    }

    const unlisten = await listen<string>("open-file", (event) => {
      selectTab("kvasir");
      openFilePath = event.payload;
    });

    return unlisten;
  });
</script>

<div class="shell">
  <div class="view-area">
    <div class="view-pane" class:active={activeTab === "hlidskjalf"}>
      <HlidskjalfView commands={{
        start_monitor: "hlid_start_monitor",
        speak: "hlid_speak",
      }} />
    </div>
    {#if mounted.has("svalinn")}
      <div class="view-pane" class:active={activeTab === "svalinn"}>
        <SvalinnView commands={{
          scan_directory: "sval_scan_directory",
          list_qa_tree: "sval_list_qa_tree",
          open_in_editor: "sval_open_in_editor",
          run_saga: "sval_run_saga",
        }} />
      </div>
    {/if}
    {#if mounted.has("kvasir")}
      <div class="view-pane" class:active={activeTab === "kvasir"}>
        <KvasirView commands={{
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
        }} openFile={openFilePath} />
      </div>
    {/if}
    {#if mounted.has("ratatoskr")}
      <div class="view-pane" class:active={activeTab === "ratatoskr"}>
        <RatatoskrView commands={{
          load_graph: "rata_load_graph",
          save_graph: "rata_save_graph",
          get_graph_stats: "rata_get_graph_stats",
          generate_sample_graph: "rata_generate_sample_graph",
        }} />
      </div>
    {/if}
  </div>

  <nav class="tab-strip">
    {#each tabs as tab}
      <button
        class="tab-btn"
        class:active={activeTab === tab.id}
        class:mounted={mounted.has(tab.id) && activeTab !== tab.id}
        onclick={() => selectTab(tab.id)}
        title="{tab.label} — {tab.desc}"
      >
        {#each tab.label.split("") as char}
          <span class="tab-char">{char}</span>
        {/each}
      </button>
    {/each}
    <div class="theme-area">
      <ThemeSwitcher />
    </div>
    {#if hasDormant}
      <button
        class="tab-btn clear-btn"
        onclick={clearDormant}
        title="Clear dormant tabs"
      >
        <span class="tab-char">&times;</span>
      </button>
    {/if}
  </nav>
</div>

<style>
  .shell {
    display: flex;
    height: 100vh;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .view-area {
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .view-pane {
    position: absolute;
    inset: 0;
    overflow: auto;
    visibility: hidden;
    pointer-events: none;
  }

  .view-pane.active {
    visibility: visible;
    pointer-events: auto;
  }

  .tab-strip {
    width: 28px;
    background: var(--bg-secondary);
    border-left: 1px solid var(--border-default);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--space-md) 0;
    gap: var(--space-md);
    flex-shrink: 0;
    overflow: hidden;
  }

  .tab-btn {
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: 0.75rem;
    font-weight: 500;
    letter-spacing: 0.05em;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0;
    padding: var(--space-xs) 2px;
    line-height: 1;
    width: 100%;
  }

  .tab-char {
    display: block;
    height: 0.9rem;
    line-height: 0.9rem;
  }

  .tab-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .tab-btn.mounted {
    color: var(--text-primary);
  }

  .tab-btn.active {
    background: var(--action-primary);
    color: var(--text-primary);
    font-weight: 700;
  }

  .theme-area {
    margin-top: auto;
    padding: var(--space-xs) 0;
    border-top: 1px solid var(--border-subtle);
  }

  .clear-btn {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }
</style>
