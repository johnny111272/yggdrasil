<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Button, Input } from "@yggdrasil/ui";
  import type { TableData } from "./kvasir-types";

  let {
    commands,
    path,
    refreshKey = 0,
  }: {
    commands: {
      read_table: string;
      export_table_csv: string;
      open_in_editor: string;
    };
    path: string;
    refreshKey?: number;
  } = $props();

  let tableData: TableData | null = $state(null);
  let loading = $state(false);
  let error: string | null = $state(null);
  let sortColumn: number | null = $state(null);
  let sortDirection: "asc" | "desc" | null = $state(null);
  let filterText = $state("");

  let filteredRows = $derived.by(() => {
    if (!tableData) return [];
    if (!filterText.trim()) return tableData.rows;
    const needle = filterText.toLowerCase();
    return tableData.rows.filter((cells) =>
      cells.some((cell) => cell.toLowerCase().includes(needle)),
    );
  });

  let sortedRows = $derived.by(() => {
    if (sortColumn === null || sortDirection === null) return filteredRows;
    const col = sortColumn;
    const dir = sortDirection === "asc" ? 1 : -1;
    return [...filteredRows].sort((rowA, rowB) => {
      const va = rowA[col] ?? "";
      const vb = rowB[col] ?? "";
      const na = Number(va);
      const nb = Number(vb);
      if (!isNaN(na) && !isNaN(nb) && va !== "" && vb !== "") {
        return (na - nb) * dir;
      }
      return va.localeCompare(vb) * dir;
    });
  });

  let rowCountLabel = $derived.by(() => {
    if (!tableData) return "";
    const total = tableData.row_count.toLocaleString();
    if (filterText.trim()) {
      return `${sortedRows.length.toLocaleString()} of ${total} rows`;
    }
    return `${total} rows`;
  });

  async function loadTable(tablePath: string) {
    loading = true;
    error = null;
    sortColumn = null;
    sortDirection = null;
    filterText = "";
    try {
      tableData = await invoke<TableData>(commands.read_table, { path: tablePath });
    } catch (err) {
      error = String(err);
      tableData = null;
    } finally {
      loading = false;
    }
  }

  async function reloadTable() {
    if (!path) return;
    loading = true;
    error = null;
    try {
      tableData = await invoke<TableData>(commands.read_table, { path });
      if (sortColumn !== null && tableData && sortColumn >= tableData.column_count) {
        sortColumn = null;
        sortDirection = null;
      }
    } catch (err) {
      error = String(err);
      tableData = null;
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    void refreshKey;
    if (refreshKey > 0) reloadTable();
  });

  function toggleSort(colIndex: number) {
    if (sortColumn === colIndex) {
      if (sortDirection === "asc") {
        sortDirection = "desc";
      } else if (sortDirection === "desc") {
        sortColumn = null;
        sortDirection = null;
      }
    } else {
      sortColumn = colIndex;
      sortDirection = "asc";
    }
  }

  function sortIndicator(colIndex: number): string {
    if (sortColumn !== colIndex) return "";
    return sortDirection === "asc" ? " \u25B2" : " \u25BC";
  }

  async function exportCsv() {
    if (!tableData) return;
    try {
      const outputPath = await invoke<string>(commands.export_table_csv, {
        headers: tableData.headers,
        rows: sortedRows,
        sourcePath: tableData.path,
      });
      await invoke(commands.open_in_editor, { path: outputPath, line: 1 });
    } catch (err) {
      error = String(err);
    }
  }

  $effect(() => {
    if (path) loadTable(path);
  });
</script>

<div class="table-viewer">
  <div class="table-controls">
    <Input type="text" bind:value={filterText} placeholder="Filter rows..." />
    <span class="row-count">{rowCountLabel}</span>
    {#if tableData}
      <span class="format-badge">{tableData.source_format.toUpperCase()}</span>
    {/if}
    <Button variant="neutral" size="sm" disabled={!tableData} title="Export as CSV" onclick={exportCsv}>
      Export CSV
    </Button>
  </div>

  {#if loading}
    <div class="table-status">Loading...</div>
  {:else if error}
    <div class="table-status table-error">{error}</div>
  {:else if tableData}
    <div class="table-scroll">
      <table>
        <thead>
          <tr>
            {#each tableData.headers as header, i}
              <th onclick={() => toggleSort(i)}>
                {header}{sortIndicator(i)}
              </th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each sortedRows as row, rowIdx}
            <tr class:alt={rowIdx % 2 === 1}>
              {#each row as cell}
                <td>{cell}</td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .table-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .table-controls {
    display: flex;
    align-items: center;
    gap: var(--space-lg);
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--border-default);
    flex-shrink: 0;
  }

  .table-filter {
    flex: 0 0 200px;
    padding: var(--space-xs) var(--space-md);
    background: var(--bg-primary);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: var(--text-sm);
  }

  .table-filter::placeholder {
    color: var(--text-muted);
  }

  .row-count {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }

  .format-badge {
    font-size: var(--text-xs);
    font-weight: 600;
    padding: 2px var(--space-sm);
    border-radius: var(--radius-sm);
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    letter-spacing: 0.05em;
  }

  .export-btn {
    margin-left: auto;
    padding: var(--space-xs) var(--space-md);
    font-size: var(--text-xs);
    background: var(--action-neutral);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    cursor: pointer;
  }

  .export-btn:hover:not(:disabled) {
    background: var(--action-neutral-hover);
  }

  .export-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .table-status {
    padding: var(--space-3xl);
    text-align: center;
    color: var(--text-secondary);
  }

  .table-error {
    color: var(--severity-error);
  }

  .table-scroll {
    flex: 1;
    overflow: auto;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    line-height: 1.4;
  }

  thead {
    position: sticky;
    top: 0;
    z-index: 1;
  }

  th {
    background: var(--bg-secondary);
    border-bottom: 2px solid var(--border-default);
    padding: var(--space-sm) var(--space-md);
    text-align: left;
    font-weight: 600;
    color: var(--text-primary);
    cursor: pointer;
    user-select: none;
    white-space: nowrap;
  }

  th:hover {
    background: var(--bg-hover);
  }

  td {
    padding: var(--space-xs) var(--space-md);
    border-bottom: 1px solid var(--border-subtle);
    color: var(--text-muted);
    white-space: nowrap;
  }

  tr.alt {
    background: var(--bg-alt-row);
  }

  tr:hover td {
    background: var(--bg-hover);
  }
</style>
