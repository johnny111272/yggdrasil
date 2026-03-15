# Shared Component Migration Plan

**Generated:** 2026-03-15
**Scope:** All 13 shared components in `@yggdrasil/ui`, all 5 apps, all supporting sub-components

---

## Executive Summary

The `@yggdrasil/ui` library contains 13 shared components. Current adoption is deeply uneven:

| App | Components Imported | Components Available But Unused |
|-----|--------------------|---------------------------------|
| **Svalinn** | 6 (Button, SidebarLayout, TreeNode, StatCard, SearchInput, FilterBanner) | 7 |
| **Kvasir** | 3 (Button, SidebarLayout, TreeNode) + Button in JsonlViewer | 9 |
| **Ratatoskr** | 1 (Button) | 12 |
| **Hlidskjalf** | 0 | 13 |
| **Yggdrasil** (host) | 1 (ThemeSwitcher) | 12 |

Svalinn is the reference implementation for shared component adoption. Hlidskjalf has zero shared component imports -- it builds everything with raw HTML/CSS, including stat cards, filter bars, collapsible sections, and badge elements that have direct shared component equivalents.

**Priority order:**
1. **Hlidskjalf** -- highest impact, 0 shared components, most raw HTML duplication
2. **Ratatoskr** -- only uses Button, has Panel/StatCard/Collapsible equivalents in raw CSS
3. **Kvasir sub-components** -- TableViewer and FormatControls bypass shared components
4. **Cross-app new components** -- AppHeader, EmptyState, CodeViewer, ErrorBanner patterns duplicated 3-4x

---

## Per-Component Migration Plan

### 1. Button

**Current state:** Accepts `variant` (primary/special/neutral/ghost), `disabled`, `onclick`, `children` snippet. Provides themed button with hover states and disabled styling.

**Who uses it:**
- Svalinn (`SvalinnView.svelte` line 6) -- Select Directory, Refresh, Run Saga, tree toggle
- Kvasir (`KvasirView.svelte` line 6) -- Select Directory, Refresh, Focus/Browse, Open in Editor
- Kvasir (`JsonlViewer.svelte` line 3) -- Open in Editor
- Ratatoskr (`RatatoskrView.svelte` line 4) -- Sample Graph, Load JSON, Save, Reset Zoom, Hide Stats, Close

**Who SHOULD use it:**
- **Hlidskjalf** `HlidskjalfView.svelte`:
  - Lines 327-333: `<button class="filter-btn">` (all/kind filter buttons) -- these are toggle-style buttons, not standard Button. May need a `ToggleButton` variant or a new `active` prop on Button.
  - Line 352: `<button class="filter-btn" ...>expand</button>` -- same toggle pattern
  - Line 365: `<button class="clear-btn" onclick={clearFeed}>clear</button>` -- could use `Button variant="ghost"` but has custom destructive-hover styling (red on hover). Would need a `danger` variant or keep custom.
  - Line 357-364: `<button class="voice-btn">` -- toggle-style button with active state. Same toggle pattern.
- **Kvasir** `TableViewer.svelte`:
  - Line 149: `<button class="export-btn">Export CSV</button>` -- should be `Button variant="ghost"` or `Button variant="neutral"`
- **Kvasir** `FormatControls.svelte`:
  - Lines 31-38: `<button class="format-btn">` -- toggle-style format selector buttons. Same pattern as Hlidskjalf filter buttons.

**Gap analysis:** Button lacks an `active` boolean prop for toggle-button states. Many app buttons act as toggles (filter kind, format selector, expand/collapse). Adding `active?: boolean` with `.btn.active { background: var(--action-primary); }` would cover the Hlidskjalf filter bar, format selectors, and view mode toggles.

The `clear-btn` destructive hover (red background) is app-specific enough to leave as custom CSS.

**Migration steps:**
1. Add `active?: boolean` prop to `Button.svelte` with `class:active` binding
2. Add `.btn.active` style matching existing `.filter-btn.active` / `.format-btn.active` pattern
3. Replace `TableViewer.svelte` export button with `<Button variant="neutral">`
4. Consider (but do not force) Hlidskjalf filter buttons -- these are a dense pill-bar pattern that may be better as a new `TogglePill` component

**Risk:** Low. Button is already widely used. Adding `active` is additive.

---

### 2. Badge

**Current state:** Accepts `count` (number) and `severity` (blocked/error/warning/success/neutral). Renders a pill with colored background.

**Who uses it:**
- Used internally by `TreeNode` and `Collapsible` (both in `ui/components/`)
- No direct app imports

**Who SHOULD use it:**
- **Svalinn** `SvalinnView.svelte`:
  - Lines 462-464: `<span class="group-count">{issues.length}</span>` -- this is a count badge in a collapsible summary, styled identically to Badge (pill, action-primary background, rounded). Direct replacement.
- **Hlidskjalf** `HlidskjalfView.svelte`:
  - Lines 423-427: `<span class="expanded-badge">src: {datagram.source}</span>` etc. -- these are text badges (not count badges), so Badge does not fit directly. Badge only accepts `count: number`.
- **Hlidskjalf** `TrafficReport.svelte`:
  - Lines 153-156: `<span class="traffic-kind ...">`, `<span class="injection-badge">injection</span>` -- text badges, not count badges.

**Gap analysis:** Badge only accepts `count: number`. Many badge usages in the codebase are text-based ("SYN", "injection", "src: bifrost"). The component needs either:
- A `text?: string` alternative to `count` (render text instead of number)
- Or a new `TextBadge` component for labeled pills

**Migration steps:**
1. Add `text?: string` prop to Badge -- when provided, renders text instead of count
2. Make `count` optional (require at least one of `count` or `text`)
3. Replace Svalinn `group-count` span with `<Badge count={issues.length} />`
4. Replace Hlidskjalf `expanded-badge` spans with `<Badge text="..." severity="neutral" />`
5. Replace TrafficReport badge elements similarly

**Risk:** Low. Badge is only used internally by TreeNode and Collapsible. Adding `text` is additive.

---

### 3. Input

**Current state:** Accepts `value` (bindable), `placeholder`, `type` (text/search/password), `disabled`, `oninput`. Styled text input with focus ring.

**Who uses it:** Nobody imports it directly.

**Who SHOULD use it:**
- **Svalinn** `SvalinnView.svelte`:
  - Lines 387-392: `<input type="text" bind:value={directory} placeholder="Or paste path here..." class="directory-input" />` -- exact match for Input component. Same styling pattern (border, bg-primary, text-primary, font-body).
- **Kvasir** `KvasirView.svelte`:
  - Lines 442-448: `<input type="text" bind:value={directory} placeholder="Or paste path here..." class="directory-input" .../>` -- same as Svalinn. Has extra `onkeydown` handler for Enter.
- **Kvasir** `TableViewer.svelte`:
  - Lines 139-144: `<input type="text" class="table-filter" placeholder="Filter rows..." bind:value={filterText} />` -- text input with filter role.

**Gap analysis:** Input does not accept `onkeydown`. Kvasir needs `onkeydown` for Enter-to-submit. Add `onkeydown?: (e: KeyboardEvent) => void` prop.

The styling difference between app inputs and the shared Input is trivial -- both use the same token-based approach. The shared Input sets `width: 100%` which may conflict with flex-layout inputs that should `flex: 1` instead. Consider adding `style` pass-through or a `fullWidth` boolean.

**Migration steps:**
1. Add `onkeydown?: (e: KeyboardEvent) => void` prop to Input
2. Consider adding `class` or `style` prop for flex overrides (or wrap in a container)
3. Replace Svalinn `directory-input` with `<Input bind:value={directory} placeholder="Or paste path here..." />`
4. Replace Kvasir `directory-input` similarly, passing `onkeydown`
5. Replace TableViewer `table-filter` with `<Input bind:value={filterText} placeholder="Filter rows..." />`

**Risk:** Low-medium. `width: 100%` on Input may need a flex override in Svalinn/Kvasir directory-row context. Test visually.

---

### 4. Select

**Current state:** Accepts `value` (bindable), `disabled`, `onchange`, `children` snippet for options. Styled select dropdown.

**Who uses it:** Nobody imports it directly.

**Who SHOULD use it:**
- **Svalinn** `SvalinnView.svelte`:
  - Lines 429-436: `<select bind:value={severityFilter}>` -- severity filter dropdown. Styled with local CSS `select { ... }`.
  - Lines 439-445: `<select bind:value={toolFilter}>` -- tool filter dropdown. Same local styling.

Both Svalinn selects use identical styling to the shared Select component (border, radius, bg-primary, text-primary).

**Gap analysis:** None. The shared Select is a direct replacement.

**Migration steps:**
1. Import `Select` in Svalinn: add to existing `import { Button, SidebarLayout, TreeNode, StatCard, SearchInput, FilterBanner } from "@yggdrasil/ui"` line
2. Replace `<select bind:value={severityFilter}>` with `<Select bind:value={severityFilter}>`
3. Replace `<select bind:value={toolFilter}>` with `<Select bind:value={toolFilter}>`
4. Remove local `select { ... }` CSS rule (lines 650-656)

**Risk:** Very low. Direct replacement.

---

### 5. Panel

**Current state:** Accepts `header` (optional string) and `children` snippet. Renders a card with bg-secondary background, border-radius, optional header with border-bottom, and content padding.

**Who uses it:** Nobody imports it directly.

**Who SHOULD use it:**
- **Svalinn** `SvalinnView.svelte`:
  - Lines 381-407: `<section class="controls">` -- a card panel with bg-secondary, padding, border-radius. Matches Panel with no header.
- **Kvasir** `KvasirView.svelte`:
  - Lines 439-453: `<section class="controls">` -- same pattern.
  - Lines 537-551: `<section class="file-info">` -- a panel displaying file path and metadata. Could be Panel with header.
- **Ratatoskr** `RatatoskrView.svelte`:
  - Lines 306-324: `<section class="controls">` -- same controls-card pattern.
  - Lines 336-365: `<aside class="stats-panel">` -- a positioned panel. Different positioning context (absolute), so Panel would need wrapping.
  - Lines 369-388: `<aside class="node-panel">` -- same, absolute positioned.

**Gap analysis:** Panel works well for the `controls` sections. The Ratatoskr stats-panel and node-panel are absolutely positioned overlays -- Panel would need to be wrapped in a positioned container, which may negate the benefit. The `file-info` section in Kvasir has a split layout (path on left, actions on right) that Panel's simple header string doesn't support. Panel would need a `header` snippet (not just string) to accommodate action buttons in the header.

**Migration steps:**
1. Consider changing Panel `header` from `string` to `Snippet | string` to support rich headers
2. Replace Svalinn `.controls` section with `<Panel>` (no header)
3. Replace Kvasir `.controls` section with `<Panel>` (no header)
4. Replace Ratatoskr `.controls` section with `<Panel>` (no header)
5. Do NOT migrate Ratatoskr absolutely-positioned panels -- keep custom
6. Evaluate Kvasir `file-info` after Panel header becomes a snippet

**Risk:** Low for controls sections. Medium for file-info (requires Panel header enhancement).

---

### 6. StatCard

**Current state:** Accepts `value` (number/string), `label`, `severity` (blocked/error/warning/success/neutral). Renders centered stat with large value, small label, and optional severity color.

**Who uses it:**
- Svalinn (`SvalinnView.svelte` line 6) -- sidecars read, total issues, blocked, errors, warnings (lines 411-416)

**Who SHOULD use it:**
- **Hlidskjalf** `HlidskjalfView.svelte`:
  - Lines 310-322: Three inline stat blocks in the header-right area: events (total), high (warning-colored), critical (error-colored). These are functionally StatCards but rendered inline in a header bar, not as standalone cards. The shared StatCard has `flex: 1`, `background: var(--bg-secondary)`, `padding: var(--space-xl)`, `border-radius: var(--radius-md)`, `text-align: center` -- this is a block-level card, not an inline header stat.
- **Ratatoskr** `RatatoskrView.svelte`:
  - Lines 338-345: Stats panel with Nodes, Edges, Density. These are rendered in a `<dl>` definition list inside an absolutely-positioned panel. Different layout from StatCard.

**Gap analysis:** Hlidskjalf's header stats are compact inline elements (no background card, no padding, no border-radius) inside a flex row. StatCard is a full card component. Making StatCard work inline would require a `compact?: boolean` or `variant?: "card" | "inline"` prop that strips the card styling. This is a significant ergonomic change.

Ratatoskr's stats are in a definition list -- a completely different layout pattern. StatCard is not a fit.

**Migration steps:**
1. Add `compact?: boolean` prop to StatCard that removes background, padding, and border-radius, rendering just the value/label pair
2. Migrate Hlidskjalf header stats to `<StatCard value={stats.total} label="events" compact />`
3. Keep Ratatoskr stats panel as-is (definition list pattern is fundamentally different)

**Risk:** Medium. `compact` mode changes StatCard semantics. Must test that existing Svalinn usage is unaffected.

---

### 7. TreeNode

**Current state:** Recursive tree node with expand/collapse, selection, double-click, custom icons, badge counts, badge severity. Accepts `node`, `depth`, `selected`, `onToggle`, `onSelect`, `onDblClickDir`, `getBadgeCount`, `getBadgeSeverity`, `getIcon`.

**Who uses it:**
- Svalinn (`SvalinnView.svelte` line 6) -- file tree with badge counts and severity
- Kvasir (`KvasirView.svelte` line 6) -- file tree with custom file-type icons

**Who SHOULD use it:** No other apps have tree views. This is fully adopted.

**Gap analysis:** None. Fully adopted by both tree-using apps.

**Migration steps:** None needed.

**Risk:** N/A.

---

### 8. SidebarLayout

**Current state:** Full sidebar+main layout with resizable sidebar, header, close button, sidebar/children/headerExtra snippets. Handles pointer drag for resize.

**Who uses it:**
- Svalinn (`SvalinnView.svelte` line 6) -- full sidebar layout with file tree
- Kvasir (`KvasirView.svelte` line 6) -- full sidebar layout with file tree

**Who SHOULD use it:**
- **Ratatoskr** `RatatoskrView.svelte`: Currently uses `<main>` + `.main-content` with no sidebar. The stats panel and node panel are overlays, not a sidebar. SidebarLayout is not a fit unless Ratatoskr redesigns to put stats in a collapsible sidebar.
- **Hlidskjalf** `HlidskjalfView.svelte`: Full-viewport feed with header and filter bar. No sidebar concept. Not a fit.

**Gap analysis:** SidebarLayout is correctly used by the two apps that have file trees. The other two apps have fundamentally different layouts.

**Migration steps:** None needed.

**Risk:** N/A.

---

### 9. Collapsible

**Current state:** `<details>` element with title, optional badge count/severity, `open` prop, children snippet. Uses Badge internally.

**Who uses it:** Nobody imports it directly.

**Who SHOULD use it:**
- **Svalinn** `SvalinnView.svelte`:
  - Lines 460-503: `<details class="group" open={...}>` with `<summary>` containing group name and count badge. This is nearly identical to Collapsible. The styling matches (bg-secondary, padding, cursor, hover). The badge rendering is manual (`<span class="group-count">`).
- **Hlidskjalf** `TrafficReport.svelte`:
  - Lines 172-190: Content sections with clickable headers that expand/collapse. Uses manual expand state (`expandedSections` Set) rather than `<details>`. Functionally equivalent to Collapsible but with left-border-color variants per content type.
  - Lines 193-230: Tool invocations section -- same collapsible pattern.

**Gap analysis:**
- Svalinn: Nearly direct replacement. The badge uses a raw `<span class="group-count">` instead of Badge component. If Badge gets `text` support (see Badge section), Collapsible would auto-render it.
- TrafficReport: Uses imperative state management (`expandedSections` Set + `toggleSection()`) instead of native `<details>`. The visual pattern is collapsible-like but with colored left borders per content type. Would need Collapsible to accept a `borderColor` prop or a CSS class. Not a simple drop-in.

**Migration steps:**
1. Migrate Svalinn `<details class="group">` blocks to `<Collapsible title={group} badgeCount={issues.length}>` -- requires Collapsible to support initial `open` based on group count (currently only accepts static `open` boolean)
2. For TrafficReport: do NOT migrate. The expand state is programmatically managed across multiple sections simultaneously, and the colored left borders are content-type-specific. Keep custom.

**Risk:** Low for Svalinn. Collapsible `open` prop may need to be reactive (bindable) if Svalinn wants conditional open based on group count.

---

### 10. SearchInput

**Current state:** Wraps an input type="search" with `value` (bindable), `placeholder`, `oninput`. Adds bottom margin via wrapper div.

**Who uses it:**
- Svalinn (`SvalinnView.svelte` line 6) -- search filter for issues (line 454)

**Who SHOULD use it:**
- **Kvasir** `TableViewer.svelte`:
  - Lines 139-144: `<input type="text" class="table-filter" placeholder="Filter rows..." bind:value={filterText} />` -- filter input. Similar to SearchInput but uses `type="text"` instead of `type="search"`. Functionally equivalent.

**Gap analysis:** TableViewer's filter input is functionally SearchInput but styled differently (no bottom margin, different width via `flex: 0 0 200px`). SearchInput's `margin-bottom: var(--space-xl)` wrapper would need to be optional or removed. The component adds a wrapping div that imposes layout opinions.

**Migration steps:**
1. Consider making SearchInput's wrapper margin configurable or removing it (let parent control margin)
2. Replace TableViewer filter input with `<SearchInput bind:value={filterText} placeholder="Filter rows..." />`
3. Override width in parent CSS if needed

**Risk:** Low-medium. SearchInput's wrapper div and margin may conflict with TableViewer's flex layout.

---

### 11. FilterBanner

**Current state:** Accepts `label`, `value`, `onClear`. Shows a highlighted bar with "Label: **Value**" and a "Clear filter" button.

**Who uses it:**
- Svalinn (`SvalinnView.svelte` line 6) -- file filter banner (line 451)

**Who SHOULD use it:**
- **Hlidskjalf** `HlidskjalfView.svelte`: Currently has no active filter banner, but the kind/priority filters are managed via toggle buttons. When workspace/session filtering is implemented (per `DISPLAY_AND_FILTERING.md`), FilterBanner would be the right component for showing active filter state.
- No other apps currently have filterable views that show active filter state.

**Gap analysis:** None for current usage. FilterBanner may need to accept multiple label/value pairs when session-aware filtering is implemented.

**Migration steps:** None needed now. Plan for FilterBanner usage when implementing session filtering.

**Risk:** N/A.

---

### 12. ListItem

**Current state:** Clickable `<li>` with hover state, keyboard support, children snippet. Styled with mono font, flex layout, border-bottom.

**Who uses it:** Nobody imports it directly.

**Who SHOULD use it:**
- **Svalinn** `SvalinnView.svelte`:
  - Lines 467-499: `<li class="issue-row">` containing a clickable issue div with location, code, message. The issue row IS a list item -- clickable, hoverable, border-bottom. However, the issue row has internal structure (issue div + detail div) that is more complex than ListItem's flat children pattern.

**Gap analysis:** ListItem provides basic clickable row styling. Svalinn's issue rows have two-level structure (main row + expandable detail) that goes beyond ListItem's scope. ListItem would work for the outer `<li>` but the inner `.issue` div with its specific flex layout and the `.issue-detail` expansion would still need custom CSS. Marginal benefit.

**Migration steps:**
1. **Do not migrate** Svalinn issue rows. The complexity of the two-level structure (main + detail) means ListItem adds a wrapper but doesn't reduce code.
2. Consider ListItem for future simple list views.

**Risk:** N/A (not recommended).

---

### 13. ThemeSwitcher

**Current state:** Theme selector with 4 themes (Darkly, Light, Warm Dark, Cool Dark). Stores preference in localStorage. Vertical or horizontal orientation.

**Who uses it:**
- Yggdrasil (`+page.svelte` line 6) -- in the tab strip area

**Who SHOULD use it:**
- Standalone apps (Hlidskjalf, Svalinn, Kvasir, Ratatoskr) do not have ThemeSwitcher. This means standalone app users cannot switch themes.

**Gap analysis:** Standalone apps have no mechanism to access ThemeSwitcher. Each standalone app's `+page.svelte` is minimal (just renders the view component). Adding ThemeSwitcher to each standalone app's `+page.svelte` or `+layout.svelte` would provide theme switching capability.

**Migration steps:**
1. Add `<ThemeSwitcher orientation="horizontal" />` to each standalone app's `+page.svelte` or `+layout.svelte`
2. Position it in a corner or as a floating element, since standalone apps don't have Yggdrasil's tab strip

**Risk:** Low. Pure addition, no replacement.

---

## New Components Needed

### N1. AppHeader

**Pattern found in:** Svalinn, Kvasir, Ratatoskr (3 apps)

**Current implementations:**
- **Svalinn** lines 377-379: `<header><h1><span class="app-name">SVALINN</span> <span class="separator">::</span> <span class="subtitle">Code Quality</span></h1></header>`
- **Kvasir** lines 435-437: `<header><h1><span class="app-name">KVASIR</span> <span class="separator">::</span> <span class="subtitle">Workspace Inspector</span></h1></header>`
- **Ratatoskr** lines 302-304: `<header><h1><span class="app-name">RATATOSKR</span> <span class="separator">::</span> <span class="subtitle">Graph Viewer</span></h1></header>`
- **Hlidskjalf** lines 302-308: Similar but more complex -- includes status badge and inline stats.

All three simple versions share IDENTICAL CSS:
```css
.app-name { font-weight: 800; letter-spacing: 0.12em; }
.separator { font-weight: 300; opacity: 0.5; color: var(--text-secondary); }
.subtitle { font-weight: 300; font-size: var(--text-sm); color: var(--text-secondary); letter-spacing: 0.04em; }
h1 { margin: 0; font-size: var(--text-lg); display: flex; align-items: baseline; gap: var(--space-md); }
```

**Proposed component:**
```svelte
<!-- AppHeader.svelte -->
<script lang="ts">
  import type { Snippet } from "svelte";
  interface Props {
    name: string;
    subtitle: string;
    extra?: Snippet;
  }
  let { name, subtitle, extra }: Props = $props();
</script>

<header>
  <h1>
    <span class="app-name">{name}</span>
    <span class="separator">::</span>
    <span class="subtitle">{subtitle}</span>
  </h1>
  {#if extra}{@render extra()}{/if}
</header>
```

**Consumers:** Svalinn, Kvasir, Ratatoskr. Hlidskjalf could use `extra` snippet for status + stats.

**CSS removed from each app:** ~20 lines per app (60 lines total).

**Risk:** Very low. Pure extraction.

---

### N2. EmptyState

**Pattern found in:** Svalinn, Kvasir, Ratatoskr, Hlidskjalf (4 apps), plus JsonlViewer

**Current implementations:**
- **Svalinn** lines 506-508: `<section class="empty-state"><p>Select a directory...</p></section>`
- **Kvasir** lines 627-633: `<section class="empty-state"><p>Select a file...</p></section>` (two different messages)
- **Ratatoskr** lines 391-395: `<section class="empty-state"><p>Load a graph JSON file...</p></section>`
- **Hlidskjalf** lines 371-375: `<div class="empty"><p class="empty-icon">...</p><p>Watching for events...</p></div>`
- **JsonlViewer** lines 212-214: `<section class="empty-state"><p>Empty JSONL file</p></section>`

Common CSS pattern:
```css
.empty-state { text-align: center; padding: 4rem var(--space-2xl); color: var(--text-secondary); }
```

Hlidskjalf variant has flex centering and a large icon.

**Proposed component:**
```svelte
<!-- EmptyState.svelte -->
<script lang="ts">
  interface Props {
    icon?: string;
    message: string;
    fill?: boolean;  // flex fill parent height
  }
  let { icon, message, fill = false }: Props = $props();
</script>

<section class="empty-state" class:fill>
  {#if icon}<p class="empty-icon">{icon}</p>{/if}
  <p>{message}</p>
</section>
```

**Consumers:** All 4 apps + JsonlViewer.

**CSS removed from each app:** ~5-10 lines per app.

**Risk:** Very low. Pure extraction.

---

### N3. ErrorBanner

**Pattern found in:** Kvasir, Ratatoskr (2 apps)

**Current implementations:**
- **Kvasir** `KvasirView.svelte` lines 457-459: `<section class="error-banner">{error}</section>`
- **Ratatoskr** `RatatoskrView.svelte` lines 327-329: `<section class="error-banner">{error}</section>`

Both share identical CSS:
```css
.error-banner {
  background: var(--severity-error);
  color: var(--text-primary);
  padding: var(--space-lg) var(--space-xl);
  border-radius: var(--radius-sm);
  margin-bottom: var(--space-xl);
}
```

**Proposed component:**
```svelte
<!-- ErrorBanner.svelte -->
<script lang="ts">
  interface Props {
    message: string;
    onDismiss?: () => void;
  }
  let { message, onDismiss }: Props = $props();
</script>

{#if message}
  <section class="error-banner">
    <span>{message}</span>
    {#if onDismiss}<button onclick={onDismiss}>Dismiss</button>{/if}
  </section>
{/if}
```

**Consumers:** Kvasir, Ratatoskr. Hlidskjalf could use it for connection errors.

**CSS removed:** ~6 lines from each of 2 apps.

**Risk:** Very low.

---

### N4. CodeViewer

**Pattern found in:** Kvasir (`KvasirView.svelte`), JsonlViewer (2 components, identical code)

**Current implementations:**
- **Kvasir** `KvasirView.svelte` lines 620-623: `<section class="code-viewer" class:wrap79 class:wrapwidth><pre><code>{#each ...}</code></pre></section>`
- **JsonlViewer** lines 207-209: Identical structure and CSS.

Both share ~40 lines of identical CSS for `.code-viewer`, `.line-number`, `.line-content`, wrap modes.

**Proposed component:**
```svelte
<!-- CodeViewer.svelte -->
<script lang="ts">
  interface Props {
    content: string;
    highlighted: string;
    wrapMode?: "nowrap" | "wrap79" | "wrapwidth";
  }
  let { content, highlighted, wrapMode = "nowrap" }: Props = $props();
</script>

<section class="code-viewer" class:wrap79={wrapMode === "wrap79"} class:wrapwidth={wrapMode === "wrapwidth"}>
  <pre><code>{#each content.split('\n') as line, i}
    {@const hl = highlighted.split('\n')[i] || ''}
    <span class="line-number" data-line={i + 1}>{i + 1}</span>
    <span class="line-content">{@html hl}</span>
  {/each}</code></pre>
</section>
```

**Consumers:** KvasirView.svelte, JsonlViewer.svelte.

**CSS removed:** ~40 lines from each of 2 components (80 lines total).

**Risk:** Low. The `{@html}` rendering is already present in both components. The component would need to accept `data-line` attribute support for scroll-to-line functionality.

---

### N5. FormatSelector (Toggle Button Group)

**Pattern found in:** FormatControls, JsonlViewer, Svalinn view modes (3 components)

**Current implementations:**
- **FormatControls** lines 29-38: `.format-selector` with `.format-btn` buttons (JSON/YAML/TOML/TOON/RON)
- **JsonlViewer** lines 192-200: Identical `.format-selector` + `.format-btn` pattern
- **Svalinn** lines 419-424: `.view-modes` with `.view-btn` buttons (By File/By Error Type/By Tool)

All three share the same CSS pattern: flex row of buttons with `active` class highlighting.

**Proposed component:**
```svelte
<!-- ToggleGroup.svelte -->
<script lang="ts">
  interface Props {
    options: { value: string; label: string }[];
    selected: string;
    onSelect: (value: string) => void;
  }
  let { options, selected, onSelect }: Props = $props();
</script>

<div class="toggle-group">
  {#each options as opt}
    <button
      class="toggle-btn"
      class:active={selected === opt.value}
      onclick={() => onSelect(opt.value)}
    >{opt.label}</button>
  {/each}
</div>
```

**Consumers:** FormatControls, JsonlViewer, Svalinn.

**CSS removed:** ~15 lines from each of 3 components.

**Risk:** Low. Pure extraction of a visual pattern.

---

### N6. ControlsPanel (Directory Controls)

**Pattern found in:** Svalinn, Kvasir (2 apps)

**Current implementations:**
- **Svalinn** lines 381-407: `.controls` section with directory-row (sidebar toggle, Select Directory button, path input, Refresh button, Run Saga button) + options-row (Include tests checkbox)
- **Kvasir** lines 439-453: `.controls` section with directory-row (Select Directory button, path input, Refresh button)

The `directory-row` flex pattern with button + input + button is shared. However, the button sets differ between apps (Svalinn has Saga, tree toggle; Kvasir does not). This is probably too app-specific to share as a component.

**Recommendation:** Do NOT create a shared ControlsPanel. The shared Panel component (with no header) already covers the card styling. The directory-row layout varies enough between apps that sharing it would require complex conditional rendering. Use Panel for the card wrapper and keep the inner layout app-specific.

---

### N7. UpButton (Sidebar Header Navigation)

**Pattern found in:** Svalinn, Kvasir (2 apps)

**Current implementations:**
- **Svalinn** line 361: `<button class="up-btn" onclick={navigateUp} ...>&#8593;</button>`
- **Kvasir** line 414: `<button class="up-btn" onclick={navigateUp} ...>&#8593;</button>`

Identical CSS (~12 lines). Both are used in `SidebarLayout` `headerExtra` snippet.

**Recommendation:** This is a very small component (a styled up-arrow button). Could be extracted but the overhead of a new shared component for 12 lines of CSS used in 2 places is marginal. **Low priority -- consider extracting only if sidebar header actions grow more complex.**

---

## Migration Dependency Order

The migrations have dependencies. This is the recommended execution order:

### Phase 1: Shared Component Enhancements (No App Changes)

1. **Badge**: Add `text?: string` prop, make `count` optional
2. **Button**: Add `active?: boolean` prop with active styling
3. **Input**: Add `onkeydown` prop
4. **Panel**: Consider changing `header` to accept Snippet
5. **StatCard**: Add `compact?: boolean` prop
6. **SearchInput**: Consider removing wrapper margin

### Phase 2: New Shared Components

7. **AppHeader**: Extract from Svalinn/Kvasir/Ratatoskr
8. **EmptyState**: Extract from all 4 apps
9. **ErrorBanner**: Extract from Kvasir/Ratatoskr
10. **CodeViewer**: Extract from KvasirView/JsonlViewer
11. **ToggleGroup**: Extract from FormatControls/JsonlViewer/Svalinn

### Phase 3: App Migrations (Per-App)

12. **Svalinn** -- Replace `<select>` with `Select`, `<details class="group">` with `Collapsible`, header with AppHeader, empty-state with EmptyState, directory input with Input
13. **Kvasir** -- Replace header with AppHeader, error-banner with ErrorBanner, empty-state with EmptyState, code-viewer with CodeViewer, directory input with Input
14. **Ratatoskr** -- Replace header with AppHeader, error-banner with ErrorBanner, empty-state with EmptyState
15. **Hlidskjalf** -- Replace expanded badges with Badge, header stats with StatCard (compact), empty state with EmptyState
16. **Kvasir sub-components** -- Replace JsonlViewer code-viewer with CodeViewer, FormatControls format buttons with ToggleGroup, TableViewer export button with Button
17. **Standalone ThemeSwitcher** -- Add ThemeSwitcher to all 4 standalone app layouts

### Phase 4: Cleanup

18. Remove orphaned CSS from each migrated component
19. Verify all apps build and render correctly
20. Visual regression test across all 4 themes

---

## Risk Assessment Per Migration

| Migration | Risk | Rationale |
|-----------|------|-----------|
| Badge text prop | Very Low | Additive change, no existing consumers affected |
| Button active prop | Very Low | Additive change, 4 consumers unaffected |
| Input onkeydown prop | Very Low | Additive change |
| Panel header snippet | Low | Existing string usage still works if using union type |
| StatCard compact prop | Low-Medium | Must verify Svalinn cards unchanged |
| SearchInput margin removal | Low-Medium | Svalinn uses it; check spacing |
| AppHeader extraction | Very Low | Pure extraction, identical CSS |
| EmptyState extraction | Very Low | Pure extraction |
| ErrorBanner extraction | Very Low | Pure extraction |
| CodeViewer extraction | Low | `{@html}` rendering, scroll-to-line attribute |
| ToggleGroup extraction | Low | Pure extraction |
| Svalinn Select migration | Very Low | Direct replacement |
| Svalinn Collapsible migration | Low | `open` prop reactivity |
| Kvasir Input migration | Low | `width: 100%` vs flex interaction |
| Ratatoskr Panel migration | Very Low | Simple card wrapper |
| Hlidskjalf Badge migration | Low | Adding import to zero-import component |
| Hlidskjalf StatCard migration | Medium | Compact mode is a new concept |
| Standalone ThemeSwitcher | Very Low | Pure addition |

**Highest risk item:** Hlidskjalf has zero shared component imports. Any migration there introduces a new dependency. Start with the lowest-risk additions (EmptyState, Badge) before attempting StatCard compact mode.

**Cross-cutting risk:** Every migration must be tested in BOTH standalone mode AND in the Yggdrasil unified shell. The `./` import constraint means new shared components must be imported from `@yggdrasil/ui`, not from relative paths within the component tree.

---

## Summary Metrics

| Metric | Count |
|--------|-------|
| Shared components with zero direct app imports | 6 (Input, Select, Panel, Collapsible, ListItem, Badge) |
| Shared components fully adopted | 2 (TreeNode, SidebarLayout) |
| Shared components partially adopted | 4 (Button, StatCard, SearchInput, FilterBanner) |
| Shared components with single consumer | 1 (ThemeSwitcher -- only Yggdrasil host) |
| New components recommended | 5 (AppHeader, EmptyState, ErrorBanner, CodeViewer, ToggleGroup) |
| New components NOT recommended | 2 (ControlsPanel, UpButton -- too app-specific or too small) |
| Total CSS lines removable via migration | ~300-400 lines across all apps |
| Apps requiring zero shared component changes | 0 (all apps have migration opportunities) |
