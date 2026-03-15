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
    const display = fullPath.startsWith(prefix)
      ? "@" + fullPath.slice(prefix.length)
      : fullPath;

    const parts = display.split("/").filter(Boolean);
    const result: Crumb[] = [];
    let accumulated = fullPath.startsWith("/") ? "" : "";

    const rawParts = fullPath.split("/").filter(Boolean);
    for (let i = 0; i < rawParts.length; i++) {
      accumulated += "/" + rawParts[i];
      result.push({
        label: parts[i] ?? rawParts[i],
        path: accumulated,
      });
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
