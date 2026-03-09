<script lang="ts">
  import Badge from "./Badge.svelte";

  interface TreeNodeData {
    name: string;
    path: string;
    is_dir: boolean;
    expanded?: boolean;
    loading?: boolean;
    children?: TreeNodeData[];
  }

  interface Props {
    node: TreeNodeData;
    depth?: number;
    selected?: string | null;
    badgeCount?: number;
    badgeSeverity?: "blocked" | "error" | "warning" | "info" | "neutral";
    onToggle?: (path: string) => void;
    onSelect?: (path: string) => void;
    getBadgeCount?: (path: string, isDir: boolean) => number;
    getBadgeSeverity?: (path: string, isDir: boolean) => string;
    getIcon?: (path: string, isDir: boolean) => string;
  }

  let {
    node,
    depth = 0,
    selected = null,
    onToggle,
    onSelect,
    getBadgeCount,
    getBadgeSeverity,
    getIcon: getIconProp,
  }: Props = $props();

  function handleClick() {
    if (node.is_dir) {
      onToggle?.(node.path);
    } else {
      onSelect?.(node.path);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") handleClick();
  }

  function getIcon(): string {
    if (node.is_dir) {
      if (node.loading) return "⏳";
      return node.expanded ? "📂" : "📁";
    }
    if (getIconProp) return getIconProp(node.path, node.is_dir);
    return "📄";
  }

  let count = $derived(getBadgeCount?.(node.path, node.is_dir) ?? 0);
  let severity = $derived(getBadgeSeverity?.(node.path, node.is_dir) ?? "neutral");
</script>

<div
  class="tree-node"
  class:selected={selected === node.path}
  style="padding-left: {depth * 16 + 8}px"
  onclick={handleClick}
  onkeydown={handleKeydown}
  role="button"
  tabindex="0"
>
  <span class="tree-icon">{getIcon()}</span>
  <span class="tree-name">{node.name}</span>
  {#if count > 0}
    <Badge {count} severity={severity as "blocked" | "error" | "warning" | "success" | "neutral"} />
  {/if}
</div>

{#if node.expanded && node.children}
  {#each node.children as child}
    <svelte:self
      node={child}
      depth={depth + 1}
      {selected}
      {onToggle}
      {onSelect}
      {getBadgeCount}
      {getBadgeSeverity}
      getIcon={getIconProp}
    />
  {/each}
{/if}

<style>
  .tree-node {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-sm) var(--space-md);
    cursor: pointer;
    user-select: none;
  }

  .tree-node:hover {
    background: var(--bg-hover);
  }

  .tree-node.selected {
    background: var(--bg-selected);
  }

  .tree-icon {
    font-size: var(--text-sm);
    width: 1.25rem;
    text-align: center;
  }

  .tree-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: var(--text-sm);
  }
</style>
