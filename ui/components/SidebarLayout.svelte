<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    showSidebar?: boolean;
    sidebarTitle?: string;
    onCloseSidebar?: () => void;
    sidebar?: Snippet;
    children?: Snippet;
    headerExtra?: Snippet;
    fullWidth?: boolean;
    noPadding?: boolean;
    minWidth?: number;
    maxWidth?: number;
  }

  let {
    showSidebar = true,
    sidebarTitle = "Files",
    onCloseSidebar,
    sidebar,
    children,
    headerExtra,
    fullWidth = false,
    noPadding = false,
    minWidth = 180,
    maxWidth = 600,
  }: Props = $props();

  let sidebarWidth = $state(280);
  let dragging = $state(false);

  function onPointerDown(event: PointerEvent) {
    dragging = true;
    (event.target as HTMLElement).setPointerCapture(event.pointerId);
    event.preventDefault();
  }

  function onPointerMove(event: PointerEvent) {
    if (!dragging) return;
    const clamped = Math.min(maxWidth, Math.max(minWidth, event.clientX));
    sidebarWidth = clamped;
  }

  function onPointerUp() {
    dragging = false;
  }
</script>

<main class:with-sidebar={showSidebar} class:dragging>
  {#if showSidebar && sidebar}
    <aside class="sidebar" style="width: {sidebarWidth}px">
      <div class="sidebar-header">
        <span class="sidebar-title">{sidebarTitle}</span>
        <div class="sidebar-header-actions">
          {#if headerExtra}
            {@render headerExtra()}
          {/if}
          {#if onCloseSidebar}
            <button class="sidebar-close" onclick={onCloseSidebar}>×</button>
          {/if}
        </div>
      </div>
      <div class="sidebar-content">
        {@render sidebar()}
      </div>
      <div
        class="resize-handle"
        onpointerdown={onPointerDown}
        onpointermove={onPointerMove}
        onpointerup={onPointerUp}
        role="separator"
        aria-orientation="vertical"
      ></div>
    </aside>
  {/if}

  <div class="main-content" class:full-width={fullWidth} class:no-padding={noPadding}>
    {#if children}
      {@render children()}
    {/if}
  </div>
</main>

<style>
  main {
    display: flex;
    height: 100%;
  }

  main.dragging {
    cursor: col-resize;
    user-select: none;
  }

  .sidebar {
    flex-shrink: 0;
    position: relative;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-default);
    display: flex;
    flex-direction: column;
    z-index: var(--z-sidebar);
  }

  .sidebar-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-xl);
    border-bottom: 1px solid var(--border-default);
  }

  .sidebar-title {
    font-weight: bold;
  }

  .sidebar-header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-md);
  }

  .sidebar-close {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }

  .sidebar-close:hover {
    color: var(--text-primary);
  }

  .sidebar-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-md) 0;
  }

  .resize-handle {
    position: absolute;
    right: -3px;
    top: 0;
    bottom: 0;
    width: 6px;
    cursor: col-resize;
    z-index: var(--z-resize);
  }

  .resize-handle:hover,
  main.dragging .resize-handle {
    background: var(--action-primary);
    opacity: 0.4;
  }

  .main-content {
    flex: 1;
    max-width: 1200px;
    margin: 0 auto;
    padding: var(--space-3xl);
    overflow-y: auto;
  }

  .main-content.full-width {
    max-width: none;
  }

  .main-content.no-padding {
    padding: 0;
  }
</style>
