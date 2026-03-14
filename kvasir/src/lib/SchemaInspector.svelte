<script lang="ts">
  import type {
    InspectedSchema,
    InspectedSection,
    InspectedField,
    InspectedAlt,
    FieldExtensions,
  } from "./schema-inspect";

  interface Props {
    schema: InspectedSchema;
  }

  let { schema }: Props = $props();

  // Toggle state
  let showDescriptions = $state(false);
  let showSemantic = $state(false);
  let showConstraints = $state(false);
  let showConditionals = $state(false);
  let showExcludes = $state(false);

  // Section collapse state
  let collapsedSections = $state(new Set<string>());

  function toggleSection(name: string) {
    const next = new Set(collapsedSections);
    if (next.has(name)) next.delete(name);
    else next.add(name);
    collapsedSections = next;
  }

  function reqLabel(requirement: { label: string; cssClass: string }): { label: string; cssClass: string; condCount: number } {
    const parts = requirement.label.split(" + ");
    const condCount = parts.filter(part => part.includes("WHEN")).length;
    if (showConditionals) return { ...requirement, condCount: 0 };
    if (condCount === 0) return { ...requirement, condCount: 0 };
    const kept = parts.filter(part => !part.includes("WHEN"));
    if (kept.length === 0) return { label: "CONDITIONAL", cssClass: "req-conditional", condCount };
    return { label: kept.join(" + "), cssClass: kept.some(part => part === "REQUIRED") ? "req-required" : requirement.cssClass, condCount };
  }
</script>

<!-- Controls bar -->
<div class="controls">
  <label>
    <input type="checkbox" bind:checked={showDescriptions} />
    descriptions
  </label>
  <label>
    <input type="checkbox" bind:checked={showSemantic} />
    semantic
  </label>
  <label>
    <input type="checkbox" bind:checked={showConstraints} />
    constraints
  </label>
  <label>
    <input type="checkbox" bind:checked={showConditionals} />
    conditionals
  </label>
  <label>
    <input type="checkbox" bind:checked={showExcludes} />
    excludes
  </label>
</div>

<!-- Header -->
<div class="header">
  <h2>{schema.title}</h2>
  {#if schema.description}
    <div class="subtitle">{schema.description}</div>
  {/if}
  <div class="stats">
    <span class="stat"><b>{schema.stats.sections}</b> sections</span>
    <span class="stat"><b>{schema.stats.fields}</b> fields</span>
    {#if schema.stats.semantic[1] > 0}
      <span class="stat">
        <b>{schema.stats.semantic[0]}/{schema.stats.semantic[1]}</b> semantic coverage
      </span>
    {/if}
  </div>
</div>

<!-- Sections -->
<div class="schema-body">
  {#each schema.sections as section}
    {@const sReq = reqLabel(section.requirement)}
    <div class="section-block">
      <button
        class="section-header"
        onclick={() => toggleSection(section.name)}
      >
        <span class="section-arrow">{collapsedSections.has(section.name) ? '\u25B6' : '\u25BC'}</span>
        <span class="section-name">{section.name}</span>
        <span class={sReq.cssClass}>
          ({sReq.label})
        </span>
        {#if section.nullable}
          <span class="section-nullable">nullable</span>
        {/if}
      </button>
      {#if !collapsedSections.has(section.name)}
        <div class="section-body">
          {#each section.fields as field}
            {@render fieldBlock(field, 0)}
          {/each}
        </div>
      {/if}
    </div>
  {/each}

  <!-- Cross-group conditionals -->
  {#if showConditionals && Object.keys(schema.crossGroupConds).length > 0}
    <div class="section-block">
      <div class="section-header cross-group-header">
        <span class="section-name">Cross-Group Conditionals</span>
      </div>
      <div class="section-body">
        {#each Object.entries(schema.crossGroupConds) as [path, conds]}
          <div class="line">
            <span class="field-name">{path}</span>:
            {#each conds as cond}
              <span class="req-conditional"> {cond}</span>
            {/each}
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<!-- Recursive field rendering -->
{#snippet fieldBlock(field: InspectedField, depth: number)}
  <div class="field-entry" style="padding-left: {depth * 16}px">
    {#if field.children}
      <!-- Group (object with children) -->
      {@const req = reqLabel(field.requirement)}
      <div class="line">
        <span class="group-name">{field.name}</span>
        (<span class={req.cssClass}>{req.label}</span>)
        {#if req.condCount}<button class="ext ext-conditional badge-btn" onclick={() => showConditionals = !showConditionals}>[conditional: {req.condCount}]</button>{/if}
        {#if field.type.includes('>=')}
          <span class="note"> [{field.type.replace('object ', '')}]</span>
        {/if}
      </div>
      {#each field.children as child}
        {@render fieldBlock(child, depth + 1)}
      {/each}
    {:else if field.alternatives}
      <!-- Array with oneOf alternatives -->
      {@const req = reqLabel(field.requirement)}
      <div class="line">
        <span class="field-name">{field.name}</span>:
        <span class="field-type">array</span>
        — <span class={req.cssClass}>{req.label}</span>
        {#if req.condCount}<button class="ext ext-conditional badge-btn" onclick={() => showConditionals = !showConditionals}>[conditional: {req.condCount}]</button>{/if}
        {@render badges(field.extensions)}
      </div>
      {@render extensionPanels(field.extensions, depth)}
      {#each field.alternatives as alt}
        {@render altBlock(alt, depth + 1)}
      {/each}
    {:else}
      <!-- Leaf field -->
      {@const req = reqLabel(field.requirement)}
      <div class="line">
        {#if field.isArrayItem}
          [<span class="field-name">{field.name}</span>:
          <span class="field-type">{field.type}</span>
          {#if field.defaultValue !== undefined}<span class="field-default"> = {JSON.stringify(field.defaultValue)}</span>{/if}
          — <span class={req.cssClass}>{req.label}</span>
          {#if req.condCount}<button class="ext ext-conditional badge-btn" onclick={() => showConditionals = !showConditionals}>[conditional: {req.condCount}]</button>{/if}
          {@render badges(field.extensions)}]
        {:else}
          <span class="field-name">{field.name}</span>:
          <span class="field-type">{field.type}</span>
          {#if field.defaultValue !== undefined}<span class="field-default"> = {JSON.stringify(field.defaultValue)}</span>{/if}
          — <span class={req.cssClass}>{req.label}</span>
          {#if req.condCount}<button class="ext ext-conditional badge-btn" onclick={() => showConditionals = !showConditionals}>[conditional: {req.condCount}]</button>{/if}
          {@render badges(field.extensions)}
        {/if}
      </div>
      {#if showDescriptions && field.description}
        <div class="line field-desc" style="padding-left: {(depth + 1) * 16}px">
          <span class="desc-text">{field.description}</span>
        </div>
      {/if}
      {@render extensionPanels(field.extensions, depth)}
    {/if}
  </div>
{/snippet}

<!-- Alt rendering -->
{#snippet altBlock(alt: InspectedAlt, depth: number)}
  <div class="field-entry" style="padding-left: {depth * 16}px">
    {#if alt.fields && alt.fields.length > 0}
      <div class="line">
        <span class="alt-label">| {alt.label}</span>
      </div>
      {#each alt.fields as f}
        {@render fieldBlock(f, depth + 1)}
      {/each}
    {:else}
      <div class="line">
        <span class="alt-label">| {alt.label}</span>:
        <span class="field-type">{alt.type}</span>
      </div>
    {/if}
  </div>
{/snippet}

<!-- Extension badges (clickable — toggle detail panels globally) -->
{#snippet badges(exts: FieldExtensions)}
  {#if exts.format}
    <span class="ext ext-format">[format: {exts.format}]</span>
  {/if}
  {#if exts.notBlock && exts.notBlock.length > 0}
    <button class="ext ext-excludes badge-btn" class:badge-active={showExcludes} onclick={() => showExcludes = !showExcludes}>
      [excludes: {exts.notBlock.length}]
    </button>
  {/if}
  {#if exts.semantic}
    <button class="ext ext-semantic sev-{exts.semantic.severity} badge-btn" class:badge-active={showSemantic} onclick={() => showSemantic = !showSemantic}>
      [semantic: {exts.semantic.severity}]
    </button>
  {/if}
  {#if exts.constraint}
    <button class="ext ext-constraint badge-btn" class:badge-active={showConstraints} onclick={() => showConstraints = !showConstraints}>
      [constraint: {exts.constraint.constraints.length}]
    </button>
  {/if}
{/snippet}

<!-- Extension detail panels -->
{#snippet extensionPanels(exts: FieldExtensions, depth: number)}
  {#if showSemantic && exts.semantic}
    <div class="ext-panel ext-panel-semantic sev-{exts.semantic.severity}" style="margin-left: {(depth + 1) * 16}px">
      {#if exts.semantic.intent}
        <div class="panel-row">
          <span class="panel-key">intent</span> {exts.semantic.intent}
        </div>
      {/if}
      {#if exts.semantic.checks}
        {#each exts.semantic.checks as check}
          <div class="panel-row">
            <span class="panel-check">{'\u2713'}</span> {check}
          </div>
        {/each}
      {/if}
      {#if exts.semantic.antiPatterns}
        {#each exts.semantic.antiPatterns as ap}
          <div class="panel-row">
            <span class="panel-anti">{'\u2717'}</span> {ap}
          </div>
        {/each}
      {/if}
    </div>
  {/if}

  {#if showConstraints && exts.constraint}
    <div class="ext-panel ext-panel-constraint" style="margin-left: {(depth + 1) * 16}px">
      {#each exts.constraint.constraints as c}
        <div class="panel-row">
          <span class="panel-rule sev-{c.severity ?? 'error'}">{c.rule}</span>
          {#if c.field}
            <span class="panel-key"> {'\u2192'} {c.field}</span>
          {/if}
          {#if c.reason}
            — {c.reason}
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  {#if showExcludes && exts.notBlock && exts.notBlock.length > 0}
    <div class="ext-panel ext-panel-excludes" style="margin-left: {(depth + 1) * 16}px">
      {#each exts.notBlock as pat}
        <div class="panel-row">
          <span class="panel-exclude">{'\u2260'}</span>
          <span class="field-type">/{pat}/</span>
        </div>
      {/each}
    </div>
  {/if}
{/snippet}

<style>
  .controls {
    position: sticky;
    top: 0;
    z-index: 10;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-default);
    padding: var(--space-md) var(--space-xl);
    display: flex;
    align-items: center;
    gap: var(--space-xl);
    flex-wrap: wrap;
  }

  .controls label {
    cursor: pointer;
    user-select: none;
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    color: var(--text-secondary);
    font-size: 0.8rem;
    font-family: var(--font-mono);
  }

  .controls label:hover { color: var(--text-primary); }
  .controls input[type="checkbox"] { cursor: pointer; accent-color: var(--action-primary); }

  .header {
    padding: var(--space-xl);
    border-bottom: 1px solid var(--border-default);
  }

  .header h2 {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .subtitle {
    color: var(--text-secondary);
    font-size: 0.8rem;
    margin-top: var(--space-xs);
  }

  .stats {
    margin-top: var(--space-md);
    display: flex;
    gap: var(--space-xl);
    font-size: 0.8rem;
  }

  .stat { color: var(--text-secondary); }
  .stat b { color: var(--text-primary); font-weight: 600; }

  .schema-body { padding: var(--space-md) 0; }

  .section-block {
    border-bottom: 1px solid var(--border-subtle);
    padding: var(--space-xs) 0;
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md) var(--space-xl);
    font-weight: 600;
    font-size: 0.9rem;
    font-family: var(--font-mono);
    background: none;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    width: 100%;
    text-align: left;
  }

  .section-header:hover { background: var(--bg-hover); }

  .section-arrow {
    font-size: 0.7rem;
    color: var(--text-secondary);
    width: 12px;
  }

  .section-name { color: var(--syntax-identifier); }
  .section-nullable { color: var(--syntax-annotation); font-weight: 400; font-size: 0.8rem; }

  .cross-group-header { cursor: default; }
  .cross-group-header:hover { background: none; }

  .section-body {
    padding: 0 var(--space-xl) var(--space-md);
    font-family: var(--font-mono);
    font-size: 0.82rem;
    line-height: 1.6;
  }

  .field-entry { padding: 1px 0; }

  .line { white-space: pre-wrap; padding: 1px 0; }

  .field-name { color: var(--text-primary); }
  .group-name { color: var(--syntax-identifier); font-weight: 600; }
  .field-type { color: var(--syntax-keyword); }
  .field-default { color: var(--syntax-value); }
  .alt-label { color: var(--syntax-annotation); }
  .note { color: var(--text-secondary); font-size: 0.8rem; }

  .field-desc {
    font-size: 0.78rem;
    color: var(--text-secondary);
    font-style: italic;
  }
  .desc-text { color: var(--text-secondary); }

  .req-required { color: var(--severity-error); }
  .req-optional { color: var(--severity-success); }
  .req-oneof { color: var(--syntax-annotation); }
  .req-conditional { color: var(--severity-warning); }

  .ext {
    font-size: 0.78rem;
    margin-left: 4px;
  }

  .ext-format { color: var(--syntax-annotation); }
  .ext-excludes { color: var(--syntax-error); }
  .ext-semantic { color: var(--syntax-conditional); }
  .ext-constraint { color: var(--syntax-constraint); }
  .ext-conditional { color: var(--syntax-conditional); }

  .badge-btn {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    font-size: 0.78rem;
    cursor: pointer;
    opacity: 0.7;
  }

  .badge-btn:hover {
    opacity: 1;
    text-decoration: underline;
  }

  .badge-btn.badge-active {
    opacity: 1;
  }

  .sev-error { color: var(--syntax-error); }
  .sev-warning { color: var(--syntax-conditional); }

  .ext-panel {
    font-size: 0.78rem;
    line-height: 1.5;
    padding: var(--space-xs) 0;
  }

  .ext-panel-semantic { border-left: 2px solid var(--border-default); padding-left: var(--space-md); }
  .ext-panel-semantic.sev-error { border-left-color: var(--syntax-error); }
  .ext-panel-semantic.sev-warning { border-left-color: var(--syntax-conditional); }

  .ext-panel-constraint { border-left: 2px solid var(--syntax-constraint); padding-left: var(--space-md); }
  .ext-panel-excludes { border-left: 2px solid var(--syntax-error); padding-left: var(--space-md); }

  .panel-row { white-space: pre-wrap; }
  .panel-key { color: var(--text-secondary); font-weight: 600; }
  .panel-check { color: var(--syntax-value); }
  .panel-anti { color: var(--syntax-error); }
  .panel-rule { font-weight: 600; }
  .panel-exclude { color: var(--syntax-error); font-weight: bold; }
</style>
