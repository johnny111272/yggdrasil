<script lang="ts">
  import { ToggleGroup } from "@yggdrasil/ui";
  import type { AllFormats, DataFormat } from "./kvasir-types";

  const formatOptions = [
    { value: "json", label: "JSON" },
    { value: "yaml", label: "YAML" },
    { value: "toml", label: "TOML" },
    { value: "toon", label: "TOON" },
    { value: "ron", label: "RON" },
  ];

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
      ron: { count: dataFormats.ron.token_count, savings: baseline - dataFormats.ron.token_count },
      source,
      baseline,
    };
  });
</script>

<section class="data-controls">
  <ToggleGroup options={formatOptions} bind:selected={selectedFormat} highlightValue={tokenStats.source} />
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
    color: var(--action-primary);
  }

  .token-item.active {
    background: var(--bg-hover);
  }

  .savings {
    font-size: var(--text-xs);
    margin-left: var(--space-xs);
  }

  .savings.positive {
    color: var(--severity-success);
  }

  .savings.negative {
    color: var(--severity-warning);
  }
</style>
