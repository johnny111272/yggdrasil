<script lang="ts">
  interface Props {
    path: string;
    rootPrefix?: string;
    onNavigate: (path: string) => void;
  }

  let { path, rootPrefix = "/Users/johnny/.ai", onNavigate }: Props = $props();

  interface Crumb {
    label: string;
    path: string;
  }

  let crumbs = $derived(buildCrumbs(path, rootPrefix));

  function buildCrumbs(fullPath: string, prefix: string): Crumb[] {
    if (!fullPath) return [];

    // Path outside the prefix — show full path segments
    if (!fullPath.startsWith(prefix)) {
      const parts = fullPath.split("/").filter(Boolean);
      const result: Crumb[] = [];
      let accumulated = "";
      for (const part of parts) {
        accumulated += "/" + part;
        result.push({ label: part, path: accumulated });
      }
      return result;
    }

    // Root crumb for the prefix
    const rootLabel = prefix.endsWith("/.ai") ? "@" : "~";
    const result: Crumb[] = [{ label: rootLabel, path: prefix }];

    // Segments below the prefix
    const remainder = fullPath.slice(prefix.length);
    const parts = remainder.split("/").filter(Boolean);
    let accumulated = prefix;
    for (const part of parts) {
      accumulated += "/" + part;
      result.push({ label: part, path: accumulated });
    }

    return result;
  }
</script>

<nav class="breadcrumbs">
  {#each crumbs as crumb, i}
    {#if i > 0}
      <span class="breadcrumb-sep">/</span>
    {/if}
    {#if i === crumbs.length - 1}
      <span class="breadcrumb-current">{crumb.label}</span>
    {:else}
      <button class="breadcrumb-link" onclick={() => onNavigate(crumb.path)}>
        {crumb.label}
      </button>
    {/if}
  {/each}
</nav>

<style>
  .breadcrumbs {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--text-secondary);
    overflow: hidden;
  }

  .breadcrumb-sep {
    opacity: 0.4;
  }

  .breadcrumb-link {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: var(--space-2xs) var(--space-xs);
    border-radius: var(--radius-sm);
    font-family: inherit;
    font-size: inherit;
    white-space: nowrap;
  }

  .breadcrumb-link:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .breadcrumb-current {
    color: var(--text-primary);
    font-weight: var(--fw-medium);
    white-space: nowrap;
  }
</style>
