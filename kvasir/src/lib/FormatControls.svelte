<script lang="ts">
  import type { AllFormats, DataFormat } from "./kvasir-types";

  let {
    dataFormats,
    selectedFormat = $bindable("json"),
  }: {
    dataFormats: AllFormats;
    selectedFormat: DataFormat;
  } = $props();

  let tokenStats = $derived.by(() => {
    const source = dataFormats.source_format as DataFormat;
    const baseline = dataFormats[source].token_count;
    return {
      json: { count: dataFormats.json.token_count, savings: baseline - dataFormats.json.token_count },
      yaml: { count: dataFormats.yaml.token_count, savings: baseline - dataFormats.yaml.token_count },
      toml: { count: dataFormats.toml.token_count, savings: baseline - dataFormats.toml.token_count },
      toon: { count: dataFormats.toon.token_count, savings: baseline - dataFormats.toon.token_count },
      source,
      baseline,
    };
  });
</script>

<section class="data-controls">
  <div class="format-selector">
    {#each ["json", "yaml", "toml", "toon", "ron"] as fmt}
      <button
        class="format-btn"
        class:active={selectedFormat === fmt}
        class:source={tokenStats.source === fmt}
        onclick={() => selectedFormat = fmt as DataFormat}
      >
        {fmt.toUpperCase()}
      </button>
    {/each}
  </div>
  <div class="token-stats">
    <span class="token-label">Tokens:</span>
    {#each ["json", "yaml", "toml", "toon", "ron"] as fmt}
      {@const stat = tokenStats[fmt as DataFormat]}
      <span
        class="token-item"
        class:source={tokenStats.source === fmt}
        class:active={selectedFormat === fmt}
      >
        {fmt.toUpperCase()}: {stat.count.toLocaleString()}
        {#if tokenStats.source !== fmt}
          <span class="savings" class:positive={stat.savings > 0} class:negative={stat.savings < 0}>
            ({stat.savings > 0 ? '-' : '+'}{Math.abs(Math.round(stat.savings / tokenStats.baseline * 100))}%)
          </span>
        {/if}
      </span>
    {/each}
  </div>
</section>

<style>
  .data-controls {
    background: var(--bg-secondary);
    padding: var(--space-lg);
    border-radius: var(--radius-md);
    margin-bottom: var(--space-lg);
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-lg);
  }

  .format-selector {
    display: flex;
    gap: var(--space-sm);
  }

  .format-btn {
    padding: var(--space-sm) var(--space-lg);
    border: none;
    border-radius: var(--radius-sm);
    background: var(--action-neutral);
    color: var(--text-primary);
    cursor: pointer;
    font-size: var(--text-sm);
    font-weight: 500;
  }

  .format-btn:hover {
    background: var(--action-neutral-hover);
  }

  .format-btn.active {
    background: var(--action-primary);
  }

  .format-btn.source {
    border: 2px solid var(--severity-info);
  }

  .token-stats {
    display: flex;
    gap: var(--space-lg);
    align-items: center;
    font-size: var(--text-sm);
    color: var(--text-secondary);
    flex-wrap: wrap;
  }

  .token-label {
    font-weight: 500;
  }

  .token-item {
    padding: var(--space-xs) var(--space-sm);
    border-radius: var(--radius-sm);
  }

  .token-item.source {
    color: var(--severity-info);
  }

  .token-item.active {
    background: var(--bg-hover);
  }

  .savings {
    font-size: var(--text-xs);
    margin-left: var(--space-xs);
  }

  .savings.positive {
    color: var(--severity-info);
  }

  .savings.negative {
    color: var(--severity-warning);
  }
</style>
