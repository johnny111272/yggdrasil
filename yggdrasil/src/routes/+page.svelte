<script lang="ts">
  import HlidskjalfView from "$hlidskjalf/HlidskjalfView.svelte";
  import SvalinnView from "$svalinn/SvalinnView.svelte";
  import KvasirView from "$kvasir/KvasirView.svelte";
  import RatatoskrView from "$ratatoskr/RatatoskrView.svelte";

  let activeTab = $state("hlidskjalf");

  const tabs = [
    { id: "hlidskjalf", label: "Hlidskjalf", desc: "Agent Monitor" },
    { id: "svalinn",    label: "Svalinn",    desc: "Code Quality" },
    { id: "kvasir",     label: "Kvasir",     desc: "Workspace Inspector" },
    { id: "ratatoskr",  label: "Ratatoskr",  desc: "Graph Viewer" },
  ];
</script>

<div class="shell">
  <div class="view-area">
    <div class="view-pane" class:active={activeTab === "hlidskjalf"}>
      <HlidskjalfView commands={{
        start_monitor: "hlid_start_monitor",
        speak: "hlid_speak",
      }} />
    </div>
    <div class="view-pane" class:active={activeTab === "svalinn"}>
      <SvalinnView commands={{
        scan_directory: "sval_scan_directory",
        list_qa_tree: "sval_list_qa_tree",
        open_in_editor: "sval_open_in_editor",
        run_saga: "sval_run_saga",
      }} />
    </div>
    <div class="view-pane" class:active={activeTab === "kvasir"}>
      <KvasirView commands={{
        list_directory: "kvas_list_directory",
        read_file: "kvas_read_file",
        open_in_editor: "kvas_open_in_editor",
        convert_to_all_formats: "kvas_convert_to_all_formats",
        detect_data_format: "kvas_detect_data_format",
      }} />
    </div>
    <div class="view-pane" class:active={activeTab === "ratatoskr"}>
      <RatatoskrView commands={{
        load_graph: "rata_load_graph",
        save_graph: "rata_save_graph",
        get_graph_stats: "rata_get_graph_stats",
        generate_sample_graph: "rata_generate_sample_graph",
      }} />
    </div>
  </div>

  <nav class="tab-strip">
    {#each tabs as tab}
      <button
        class="tab-btn"
        class:active={activeTab === tab.id}
        onclick={() => activeTab = tab.id}
        title="{tab.label} — {tab.desc}"
      >
        {#each tab.label.split("") as char}
          <span class="tab-char">{char}</span>
        {/each}
      </button>
    {/each}
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
    width: 22px;
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
    font-size: 0.5625rem;
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
    height: 0.75rem;
    line-height: 0.75rem;
  }

  .tab-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .tab-btn.active {
    background: var(--action-primary);
    color: var(--text-primary);
    font-weight: 700;
  }
</style>
