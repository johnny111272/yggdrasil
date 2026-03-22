<script lang="ts">
  import type { Snippet } from "svelte";
  import type { ModeOption, SidebarMode, FontFamily, FontTarget } from "../types/container-types.js";
  import SettingsBar from "./SettingsBar.svelte";
  import ModeBar from "./ModeBar.svelte";
  import Breadcrumbs from "./Breadcrumbs.svelte";
  import TreeNode from "./TreeNode.svelte";
  import Button from "./Button.svelte";

  // ── Exported types for consumers ────────────────────────────────────
  export type { ModeOption, SidebarMode, FontFamily, FontTarget };

  // Re-export TreeNodeData shape from TreeNode
  interface TreeNodeData {
    name: string;
    path: string;
    is_dir: boolean;
    expanded?: boolean;
    loading?: boolean;
    children?: TreeNodeData[];
  }

  // ── Props ───────────────────────────────────────────────────────────
  interface Props {
    // Identity
    appName: string;
    storagePrefix?: string;

    // Sidebar
    sidebarMode?: SidebarMode;
    sidebarTitle?: string;
    sidebar?: Snippet;

    // Tree mode (sidebarMode="tree")
    directory?: string;
    showHidden?: boolean;
    treeNodes?: TreeNodeData[];
    selectedFile?: string | null;
    onTreeToggle?: (path: string) => void;
    onTreeSelect?: (path: string) => void;
    onTreeDblClickDir?: (path: string) => void;
    onDirectoryChange?: (path: string) => void;
    getTreeBadgeCount?: (path: string, isDir: boolean) => number;
    getTreeBadgeSeverity?: (path: string, isDir: boolean) => string;
    getTreeIcon?: (path: string, isDir: boolean) => string;

    // Mode bar
    modeOptions?: ModeOption[];
    activeMode?: string;
    onModeChange?: (value: string) => void;

    // Breadcrumbs
    breadcrumbPath?: string;
    onBreadcrumbNavigate?: (path: string) => void;

    // CWD picker (provided by host — keeps ContainerLayout free of Tauri deps)
    onSetCwd?: () => void;

    // Tabs (Ygg only)
    appTabs?: Snippet;

    // Layout
    fullWidth?: boolean;
    noPadding?: boolean;
    children?: Snippet;
  }

  let {
    appName,
    storagePrefix = "solo",

    sidebarMode = "none",
    sidebarTitle = "Files",
    sidebar,

    directory = $bindable(""),
    showHidden = $bindable(false),
    treeNodes = [],
    selectedFile = null,
    onTreeToggle,
    onTreeSelect,
    onTreeDblClickDir,
    onDirectoryChange,
    getTreeBadgeCount,
    getTreeBadgeSeverity,
    getTreeIcon,

    modeOptions = [],
    activeMode = $bindable(""),
    onModeChange,

    breadcrumbPath,
    onBreadcrumbNavigate,
    onSetCwd,

    appTabs,

    fullWidth = false,
    noPadding = false,
    children,
  }: Props = $props();

  // ── Constants ─────────────────────────────────────────────────────────
  const HOME = "/Users/johnny";
  const AI_HOME = `${HOME}/.ai`;

  // ── Persisted state ─────────────────────────────────────────────────
  function storageKey(property: string): string {
    return `${storagePrefix}-${appName}-${property}`;
  }

  function loadPersisted(property: string, fallback: string): string {
    if (typeof localStorage === "undefined") return fallback;
    return localStorage.getItem(storageKey(property)) ?? fallback;
  }

  function persist(property: string, value: string) {
    if (typeof localStorage === "undefined") return;
    localStorage.setItem(storageKey(property), value);
  }

  let sidebarCollapsed = $state(loadPersisted("sidebarCollapsed", "false") === "true");
  let sidebarWidth = $state(parseInt(loadPersisted("sidebarWidth", "280"), 10));
  let settingsCollapsed = $state(loadPersisted("settingsCollapsed", "false") === "true");
  let contentFontSize = $state(parseInt(loadPersisted("contentFontSize", "14"), 10));
  let fontFamily: FontFamily = $state(loadPersisted("fontFamily", "mono") as FontFamily);
  let homeBrowse = $state(loadPersisted("homeBrowse", "false") === "true");

  // Persist on change
  $effect(() => { persist("sidebarCollapsed", String(sidebarCollapsed)); });
  $effect(() => { persist("sidebarWidth", String(sidebarWidth)); });
  $effect(() => { persist("settingsCollapsed", String(settingsCollapsed)); });
  $effect(() => { persist("contentFontSize", String(contentFontSize)); });
  $effect(() => { persist("fontFamily", fontFamily); });
  $effect(() => { persist("homeBrowse", String(homeBrowse)); });

  // Font family mapping
  const fontFamilyMap: Record<FontFamily, string> = {
    mono: "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, monospace",
    dyslexie: "'Dyslexie', ui-monospace, monospace",
    sans: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
    serif: "Georgia, 'Times New Roman', serif",
  };

  // Apply CSS custom properties for content font
  $effect(() => {
    const root = document.documentElement;
    root.style.setProperty("--content-font-size", `${contentFontSize}px`);
    root.style.setProperty("--content-font-family", fontFamilyMap[fontFamily]);
  });

  // ── CWD boundary ─────────────────────────────────────────────────────
  let cwdRoot = $derived(homeBrowse ? HOME : AI_HOME);

  // Enforce boundary: clamp directory if it escapes the root
  $effect(() => {
    if (directory && !directory.startsWith(cwdRoot)) {
      directory = cwdRoot;
      onDirectoryChange?.(cwdRoot);
    }
  });

  // ── Sidebar resize ─────────────────────────────────────────────────
  let dragging = $state(false);
  const MIN_SIDEBAR = 180;
  const MAX_SIDEBAR = 600;

  function onResizePointerDown(event: PointerEvent) {
    dragging = true;
    (event.target as HTMLElement).setPointerCapture(event.pointerId);
    event.preventDefault();
  }

  function onResizePointerMove(event: PointerEvent) {
    if (!dragging) return;
    sidebarWidth = Math.min(MAX_SIDEBAR, Math.max(MIN_SIDEBAR, event.clientX));
  }

  function onResizePointerUp() {
    dragging = false;
  }

  // ── CWD navigation ──────────────────────────────────────────────────
  function navigateUp() {
    if (!directory || directory === cwdRoot) return;
    const parent = directory.replace(/\/[^/]+\/?$/, "") || "/";
    if (!parent.startsWith(cwdRoot)) return;
    directory = parent;
    onDirectoryChange?.(parent);
  }

  // Wrapped breadcrumb navigation — enforce boundary
  function handleBreadcrumbNavigate(path: string) {
    if (!path.startsWith(cwdRoot)) return;
    onBreadcrumbNavigate?.(path);
  }

  // ── Derived ─────────────────────────────────────────────────────────
  let hasSidebar = $derived(sidebarMode !== "none");
  let showSidebar = $derived(hasSidebar && !sidebarCollapsed);
  let effectiveBreadcrumbPath = $derived(breadcrumbPath ?? directory);
  let hasBreadcrumbs = $derived(!!effectiveBreadcrumbPath && !!onBreadcrumbNavigate);
  let hasModeBar = $derived(modeOptions.length > 0);
  let atBoundary = $derived(directory === cwdRoot);
</script>

<div class="container-layout" class:dragging>
  <!-- ═══ SIDEBAR ═══ -->
  {#if hasSidebar}
    <div class="sidebar-toggle-bar" onclick={() => sidebarCollapsed = !sidebarCollapsed} onkeydown={(e) => e.key === 'Enter' && (sidebarCollapsed = !sidebarCollapsed)} role="button" tabindex="0" title={sidebarCollapsed ? "Show sidebar" : "Hide sidebar"}>
      <span class="toggle-bar-chevron">{sidebarCollapsed ? '\u25B6' : '\u25C0'}</span>
    </div>
    {#if showSidebar}
      <aside class="container-sidebar" style="width: {sidebarWidth}px">
        <div class="sidebar-header">
          <span class="sidebar-cwd" title={directory}>
            {#if sidebarMode === "tree" && directory}
              {directory.split("/").pop() || directory}
            {:else}
              {sidebarTitle}
            {/if}
          </span>
          {#if sidebarMode === "tree"}
            <Button variant="ghost" size="sm" active={showHidden} onclick={() => showHidden = !showHidden} title={showHidden ? "Hide dotfiles" : "Show dotfiles"}>.*</Button>
            {#if directory}
              <Button variant="ghost" size="sm" onclick={navigateUp} disabled={atBoundary} title="Parent directory">&#x25B2;</Button>
            {/if}
          {/if}
        </div>

        <div class="sidebar-body">
          {#if sidebarMode === "tree"}
            {#if treeNodes.length > 0}
              {#each treeNodes as node}
                <TreeNode
                  {node}
                  selected={selectedFile}
                  onToggle={onTreeToggle}
                  onSelect={onTreeSelect}
                  onDblClickDir={onTreeDblClickDir}
                  getBadgeCount={getTreeBadgeCount}
                  getBadgeSeverity={getTreeBadgeSeverity}
                  getIcon={getTreeIcon}
                />
              {/each}
            {:else if directory}
              <div class="sidebar-empty">No files</div>
            {/if}
          {:else if sidebarMode === "custom" && sidebar}
            {@render sidebar()}
          {/if}
        </div>

        <div class="sidebar-footer">
          {#if onSetCwd}
            <Button variant="ghost" size="sm" onclick={onSetCwd}>Set CWD</Button>
          {/if}
          {#if sidebarMode === "tree"}
            <Button variant="ghost" size="sm" active={homeBrowse} onclick={() => homeBrowse = !homeBrowse} title={homeBrowse ? "Browsing ~/ (click to restrict to ~/.ai/)" : "Browsing ~/.ai/ (click to allow ~/)"}>
              {homeBrowse ? "~/" : "@/"}
            </Button>
          {/if}
        </div>

        <div
          class="resize-handle"
          onpointerdown={onResizePointerDown}
          onpointermove={onResizePointerMove}
          onpointerup={onResizePointerUp}
          role="separator"
          aria-orientation="vertical"
        ></div>
      </aside>
    {/if}
  {/if}

  <!-- ═══ MAIN AREA ═══ -->
  <div class="container-main">
    <SettingsBar
      bind:collapsed={settingsCollapsed}
      bind:contentFontSize
      bind:fontFamily
    />

    <div class="content-frame" class:full-width={fullWidth} class:no-padding={noPadding}>
      {#if hasModeBar}
        <ModeBar options={modeOptions} bind:selected={activeMode} onSelect={onModeChange} />
      {/if}
      <div class="app-body">
        {#if children}
          {@render children()}
        {/if}
      </div>
    </div>

    {#if hasBreadcrumbs}
      <div class="breadcrumbs-bar">
        <Breadcrumbs path={effectiveBreadcrumbPath} rootPrefix={cwdRoot} onNavigate={handleBreadcrumbNavigate} />
      </div>
    {/if}

    {#if appTabs}
      <div class="app-tabs-bar">
        {@render appTabs()}
      </div>
    {/if}
  </div>
</div>

<style>
  .container-layout {
    display: flex;
    height: 100%;
  }

  .container-layout.dragging {
    cursor: col-resize;
    user-select: none;
  }

  /* ═══ Sidebar ═══ */

  .container-sidebar {
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
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--border-default);
    gap: var(--space-xs);
  }

  .sidebar-cwd {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: var(--fw-semibold);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }

  .sidebar-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-xs) 0;
  }

  .sidebar-empty {
    padding: var(--space-xl);
    text-align: center;
    color: var(--text-secondary);
    font-size: var(--text-xs);
  }

  .sidebar-footer {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-sm) var(--space-lg);
    border-top: 1px solid var(--border-default);
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
  .container-layout.dragging .resize-handle {
    background: var(--action-primary);
    opacity: 0.4;
  }

  /* ═══ Sidebar toggle bar (collapsed state) ═══ */

  .sidebar-toggle-bar {
    width: 28px;
    flex-shrink: 0;
    border-right: 1px solid var(--border-default);
    background: var(--bg-control);
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .sidebar-toggle-bar:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .toggle-bar-chevron {
    font-size: var(--text-2xs);
  }

  /* ═══ Main area ═══ */

  .container-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
  }

  .content-frame {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .app-body {
    flex: 1;
    overflow-y: auto;
    max-width: 1200px;
    margin: 0 auto;
    padding: var(--space-xl);
    width: 100%;
    font-size: var(--content-font-size, 14px);
    font-family: var(--content-font-family, var(--font-mono));
  }

  .content-frame.full-width .app-body {
    max-width: none;
  }

  .content-frame.no-padding .app-body {
    padding: 0;
  }

  .breadcrumbs-bar {
    flex-shrink: 0;
    padding: var(--space-xs) var(--space-lg);
    border-top: 1px solid var(--border-subtle);
    background: var(--bg-secondary);
  }

  .app-tabs-bar {
    flex-shrink: 0;
    border-top: 1px solid var(--border-default);
    background: var(--bg-secondary);
  }
</style>
