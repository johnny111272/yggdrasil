# CSS Architecture Audit -- Yggdrasil

**Date:** 2026-03-15
**Scope:** All `.css` files and `<style>` blocks in `.svelte` files across the workspace
**Files audited:** 2 CSS files, 13 shared UI components, 11 app components, 10 route pages/layouts

---

## Executive Summary

The CSS architecture is **well-structured overall**. The design token system (`tokens.css`) is comprehensive and consistently used for colors, spacing, and typography across the vast majority of the codebase. The `base.css` reset is minimal and correct. Every app layout imports tokens and base through the `+layout.svelte` pattern.

**Critical findings:**

1. **Three competing `height: 100vh` declarations** create a layout conflict when views are hosted inside Yggdrasil. SidebarLayout, HlidskjalfView, and the Yggdrasil shell each claim 100vh, producing nested 100vh containers. A `:global(main)` hack in Yggdrasil's `+page.svelte` patches one case but not all.

2. **SchemaInspector has 13 hardcoded font-size values** (`0.78rem`, `0.8rem`, `0.82rem`, `0.9rem`, `1rem`) that bypass the `--text-*` token scale entirely. This is the worst token compliance in the codebase.

3. **Three shared components impose external margin** (SearchInput, Collapsible, FilterBanner), violating the component composition principle that consumers should control external spacing.

4. **Workspace colors in HlidskjalfView use 6 hardcoded HSL values** that are not theme-aware and do not change across the 4 defined themes.

5. **Zero media queries anywhere.** No responsive CSS exists. The apps are desktop-only Tauri shells, so this is likely intentional, but it means no adaptation for different window sizes.

6. **Zero `!important` declarations.** This is good.

---

## 1. Layout Property Map

### Height Chain Analysis

The height ownership chain is the most critical layout concern. Here is how height flows from root to leaf:

| Layer | File | Element | height | overflow | position | display |
|-------|------|---------|--------|----------|----------|---------|
| Reset | `base.css` | `body` | (none -- defaults to auto) | (none) | (none) | (none) |
| **Ygg shell** | `yggdrasil/+page.svelte` | `.shell` | **100vh** | (none) | (none) | flex |
| Ygg view-area | `yggdrasil/+page.svelte` | `.view-area` | (none) | **hidden** | relative | (none -- flex:1) |
| Ygg view-pane | `yggdrasil/+page.svelte` | `.view-pane` | (none) | **auto** | **absolute, inset:0** | (none) |
| Ygg global patch | `yggdrasil/+page.svelte` | `.view-pane :global(main)` | **100%** | (none) | (none) | (none) |
| Ygg tab-strip | `yggdrasil/+page.svelte` | `.tab-strip` | (none) | **hidden** | (none) | flex-column |
| **SidebarLayout** | `ui/SidebarLayout.svelte` | `main` | **100vh** | (none) | (none) | flex |
| SidebarLayout sidebar | `ui/SidebarLayout.svelte` | `.sidebar` | (none) | (none) | **fixed** | flex-column |
| SidebarLayout sidebar-content | `ui/SidebarLayout.svelte` | `.sidebar-content` | (none) | **auto-y** | (none) | (none -- flex:1) |
| SidebarLayout main-content | `ui/SidebarLayout.svelte` | `.main-content` | (none) | **auto-y** | (none) | (none -- flex:1) |
| **HlidskjalfView** | `hlidskjalf/HlidskjalfView.svelte` | `.watchtower` | **100vh** | (none) | (none) | flex-column |
| Hlid header | same | `.header` | (none) | (none) | (none) | flex (flex-shrink:0) |
| Hlid filters | same | `.filters` | (none) | (none) | (none) | flex (flex-shrink:0) |
| Hlid feed | same | `.feed` | (none) | **auto-y** | (none) | (none -- flex:1) |
| Hlid event-base | same | `.event-base` | min-height:1.6em | (none) | relative | flex |
| Hlid col-source | same | `.col-source` | (none) | (none) | **absolute** | (none) |
| Hlid expanded-json | same | `.expanded-json` | max:300px | **auto x+y** | (none) | (none) |
| **RatatoskrView** | `ratatoskr/RatatoskrView.svelte` | `main` | **100vh** | (none) | (none) | flex |
| Rata main-content | same | `.main-content` | (none) | **auto** | (none) | flex-column (flex:1) |
| Rata graph-container | same | `.graph-container` | min:400px | **hidden** | relative | (none -- flex:1) |
| Rata stats-panel | same | `.stats-panel` | (none) | (none) | **absolute** | (none) |
| Rata node-panel | same | `.node-panel` | (none) | (none) | **absolute** | (none) |
| **KvasirView** | Uses SidebarLayout (height from there) | | | | | |
| Kvasir code-viewer | `kvasir/KvasirView.svelte` | `.code-viewer` | (none) | **hidden** (intentional clip) | (none) | (none) |
| Kvasir inspector-view | same | `.inspector-view` | (none) | **auto** | (none) | (none) |
| **SvalinnView** | Uses SidebarLayout (height from there) | | | | | |
| **Panel** | `ui/Panel.svelte` | `.panel` | (none) | **auto** | (none) | (none) |
| **TableViewer** | `kvasir/TableViewer.svelte` | `.table-viewer` | 100% | **hidden** | (none) | flex-column |
| Table thead | same | `thead` | (none) | (none) | **sticky top:0** | (none) |
| Table table-scroll | same | `.table-scroll` | (none) | **auto** | (none) | (none -- flex:1) |
| **SchemaInspector** | `kvasir/SchemaInspector.svelte` | `.controls` | (none) | (none) | **sticky top:0** | flex |

### Height Conflict: Triple 100vh

**Problem:** When HlidskjalfView runs inside Yggdrasil:
- `.shell` claims 100vh
- `.view-pane :global(main)` sets height:100% (patching SidebarLayout's `main` to inherit)
- But `.watchtower` (HlidskjalfView root) also claims **100vh** independently

This means `.watchtower` fills 100vh of the actual viewport, not 100% of its container. The `:global(main)` patch in Yggdrasil overrides SidebarLayout's `main` tag to use `height: 100%` instead of `100vh`, but HlidskjalfView's `.watchtower` div is not a `<main>` -- it is a `<div>`, so the patch does not reach it.

Similarly, RatatoskrView's `<main>` element also sets `height: 100vh`. The `:global(main)` patch does override this one, but it is fragile.

**Impact:** In standalone mode all three 100vh work fine. In Yggdrasil, the tab strip occupies 28px on the right, and views are positioned absolutely inside `.view-area`. The 100vh on `.watchtower` causes it to extend beyond its absolute-positioned container by the height of any layout overhead. Practically this may be invisible because `overflow: auto` on `.view-pane` absorbs it, but it is architecturally wrong.

**Recommendation:** Views should use `height: 100%` and rely on their host to provide the viewport-filling container. Standalone `+page.svelte` wraps them at 100vh; Yggdrasil wraps them in absolute-inset panes.

---

## 2. Hardcoded Values Inventory

### 2a. Hardcoded Font Sizes (should use --text-* tokens)

| File | Line | Value | Closest Token |
|------|------|-------|---------------|
| `ui/components/SidebarLayout.svelte` | 128 | `1.5rem` | `--text-xl` (1.5rem) -- exact match but not using token |
| `ui/components/ThemeSwitcher.svelte` | 72 | `0.65rem` | Below `--text-xs` (0.75rem) -- no token exists |
| `yggdrasil/src/routes/+page.svelte` | 183 | `0.75rem` | `--text-xs` (0.75rem) -- exact match but not using token |
| `yggdrasil/src/routes/+page.svelte` | 224 | `0.75rem` | `--text-xs` -- same |
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 651 | `48px` | No token (decorative empty-state icon) |
| `hlidskjalf/src/lib/TrafficReport.svelte` | 315 | `9px` | Below `--text-xs` -- no token exists |
| `hlidskjalf/src/lib/TrafficReport.svelte` | 387 | `9px` | Same |
| `hlidskjalf/src/lib/TrafficReport.svelte` | 451 | `8px` | Below `--text-xs` -- no token exists |
| `kvasir/src/lib/SchemaInspector.svelte` | 310 | `0.8rem` | Between `--text-xs` and `--text-sm` -- no token |
| `kvasir/src/lib/SchemaInspector.svelte` | 323 | `1rem` | `--text-base` -- exact match but not using token |
| `kvasir/src/lib/SchemaInspector.svelte` | 331 | `0.8rem` | Same as above |
| `kvasir/src/lib/SchemaInspector.svelte` | 339 | `0.8rem` | Same |
| `kvasir/src/lib/SchemaInspector.svelte` | 358 | `0.9rem` | Between `--text-sm` and `--text-base` -- no token |
| `kvasir/src/lib/SchemaInspector.svelte` | 371 | `0.7rem` | Below `--text-xs` -- no token exists |
| `kvasir/src/lib/SchemaInspector.svelte` | 377 | `0.8rem` | Same pattern |
| `kvasir/src/lib/SchemaInspector.svelte` | 385 | `0.82rem` | No token (non-standard size) |
| `kvasir/src/lib/SchemaInspector.svelte` | 398 | `0.8rem` | Same |
| `kvasir/src/lib/SchemaInspector.svelte` | 401,413,428,446 | `0.78rem` | No token (non-standard size) |
| `kvasir/src/lib/MarkdownPreview.svelte` | 53 | `0.9em` | Relative to parent -- intentional for inline code |
| `ratatoskr/src/lib/RatatoskrView.svelte` | 182 | `10px` (D3 attr) | N/A -- D3 programmatic, not CSS |
| `ratatoskr/src/lib/RatatoskrView.svelte` | 217 | `12px` (D3 attr) | N/A -- D3 programmatic, not CSS |

**SchemaInspector is the worst offender** with 13 hardcoded font-size values using 6 different non-standard sizes (0.7rem, 0.78rem, 0.8rem, 0.82rem, 0.9rem, 1rem). None use tokens.

### 2b. Hardcoded Spacing (px values for padding/margin/gap)

These are `px` values used for spacing that could potentially be tokens. Note: `1px` for borders is standard CSS and not considered a violation.

| File | Line | Value | Context | Suggested Token |
|------|------|-------|---------|-----------------|
| `ui/components/ThemeSwitcher.svelte` | 56 | `gap: 2px` | vertical button spacing | `--space-xs` (4px) or too small for tokens |
| `ui/components/SidebarLayout.svelte` | 146 | `right: -3px` | resize handle offset | Structural -- no token needed |
| `ui/components/SidebarLayout.svelte` | 149 | `width: 6px` | resize handle width | Structural -- no token needed |
| `ui/components/SidebarLayout.svelte` | 162 | `max-width: 1200px` | main content cap | `--sidebar-width` pattern (define `--content-max-width` token) |
| `ui/components/Badge.svelte` | 30 | `padding: 0.125rem` | badge vertical padding | Very small, half of `--space-xs` |
| `yggdrasil/src/routes/+page.svelte` | 165 | `width: 28px` | tab strip width | Define `--tab-strip-width` token |
| `yggdrasil/src/routes/+page.svelte` | 191 | `padding: ... 2px` | tab button horiz padding | Below `--space-xs` |
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 510 | `padding: 2px 8px` | status badge | `--space-xs --space-md` |
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 563,598,619 | `padding: 2px 10px` | filter/voice/clear buttons | No 10px token |
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 592 | `gap: 4px` | auto-scroll label gap | `--space-xs` (4px) -- exact match |
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 721 | `min-width: 72px` | time column | Structural |
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 726 | `min-width: 20px` | kind icon | Structural |
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 735 | `min-width: 60px` | kind label | Structural |
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 743 | `padding: 1px 6px` | classifier chip | Micro spacing |
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 753 | `min-width: 80px` | workspace column | Structural |
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 825,853 | `padding: 1px 6px` | source/meta badges | Micro spacing |
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 865 | `max-height: 300px` | expanded JSON | Structural |
| `hlidskjalf/src/lib/QualityReport.svelte` | 172 | `height: 2px` | banner bar | Decorative |
| `hlidskjalf/src/lib/QualityReport.svelte` | 186 | `padding: 2px 10px` | SYN badge | Micro spacing |
| `hlidskjalf/src/lib/QualityReport.svelte` | 273 | `padding: 2px 0` | edu row | Micro spacing |
| `hlidskjalf/src/lib/QualityReport.svelte` | 281 | `padding: 1px 6px` | edu label | Micro spacing |
| `hlidskjalf/src/lib/QualityReport.svelte` | 284 | `margin-top: 2px` | edu label | Micro spacing |
| `hlidskjalf/src/lib/QualityReport.svelte` | 320 | `padding: 1px 0` | location link | Micro spacing |
| `hlidskjalf/src/lib/TrafficReport.svelte` | 269 | `padding: 2px 10px` | kind badge | Same pattern |
| `hlidskjalf/src/lib/TrafficReport.svelte` | 302 | `padding: 1px 6px` | injection badge | Micro spacing |
| `hlidskjalf/src/lib/TrafficReport.svelte` | 310 | `gap: 3px` | field pips | Below `--space-xs` |
| `hlidskjalf/src/lib/TrafficReport.svelte` | 317,318 | `width/height: 16px` | field pip | `--space-xl` (16px) -- exact match |
| `hlidskjalf/src/lib/TrafficReport.svelte` | 322 | `border-radius: 2px` | field pip | Below `--radius-sm` (4px) |
| `hlidskjalf/src/lib/TrafficReport.svelte` | 439 | `padding: 2px ...` | tool summary | Micro spacing |
| `hlidskjalf/src/lib/TrafficReport.svelte` | 469 | `max-height: 300px` | tool detail | Structural |
| `hlidskjalf/src/lib/TrafficReport.svelte` | 486 | `padding: 1px 6px` | tool-added name | Micro spacing |
| `kvasir/src/lib/KvasirView.svelte` | 801 | `gap: 2px` | font controls | Below `--space-xs` |
| `kvasir/src/lib/SchemaInspector.svelte` | 373 | `width: 12px` | section arrow | Structural |
| `kvasir/src/lib/SchemaInspector.svelte` | 389 | `padding: 1px 0` | field entry | Micro spacing |
| `kvasir/src/lib/SchemaInspector.svelte` | 391 | `padding: 1px 0` | line | Micro spacing |
| `kvasir/src/lib/SchemaInspector.svelte` | 414 | `margin-left: 4px` | ext badge | `--space-xs` (4px) -- exact match |
| `kvasir/src/lib/JsonlViewer.svelte` | 265 | `height: 4px` | scrubber track | Structural |
| `kvasir/src/lib/JsonlViewer.svelte` | 268 | `border-radius: 2px` | scrubber track | Below `--radius-sm` |
| `kvasir/src/lib/JsonlViewer.svelte` | 275,276 | `width/height: 14px` | scrubber thumb | No token |
| `kvasir/src/lib/TableViewer.svelte` | 202 | `flex: 0 0 200px` | filter input width | Structural |
| `kvasir/src/lib/TableViewer.svelte` | 224 | `padding: 2px ...` | format badge | Micro spacing |
| `ratatoskr/src/lib/RatatoskrView.svelte` | 472 | `min-height: 400px` | graph container | Structural |
| `ratatoskr/src/lib/RatatoskrView.svelte` | 489 | `max-width: 250px` | stats/node panels | Structural |
| `ratatoskr/src/lib/RatatoskrView.svelte` | 549,550 | `width/height: 10px` | type dot | Structural |
| `svalinn/src/lib/SvalinnView.svelte` | 773 | `padding: 4rem ...` | empty state | No `--space-4xl` token |
| `kvasir/src/lib/KvasirView.svelte` | 898 | `padding: 4rem ...` | empty state | Same |
| `kvasir/src/lib/JsonlViewer.svelte` | 359 | `padding: 4rem ...` | empty state | Same |

### 2c. Hardcoded Colors (not in style blocks, in JS)

| File | Line | Value | Issue |
|------|------|-------|-------|
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 99-104 | 6 HSL workspace colors | Not theme-aware. Hardcoded in JS, not CSS custom properties. Will look wrong in light/warm-dark/cool-dark themes. |

No hardcoded hex colors (`#xxx`) were found in any `<style>` block. All color references use `var(--*)` tokens. The token system color compliance is excellent.

---

## 3. Spacing Leaks (Shared Components with External Margin)

Shared components should not impose external margin on themselves. The consumer should control spacing via `gap` on a parent flex/grid container or explicit margin on the consumer side.

| Component | File | Declaration | Impact |
|-----------|------|-------------|--------|
| **SearchInput** | `ui/components/SearchInput.svelte:23` | `.search-wrapper { margin-bottom: var(--space-xl) }` | Forces 16px below every SearchInput. Consumer cannot override without specificity fight. |
| **Collapsible** | `ui/components/Collapsible.svelte:34` | `.collapsible { margin-bottom: var(--space-md) }` | Forces 8px below every Collapsible. Spacing should be controlled by parent gap. |
| **FilterBanner** | `ui/components/FilterBanner.svelte:23` | `.filter-banner { margin-bottom: var(--space-xl) }` | Forces 16px below every FilterBanner. |
| **SidebarLayout** | `ui/components/SidebarLayout.svelte:163` | `.main-content { margin: 0 auto }` | Centering margin -- this is reasonable for layout components. Not a leak. |

**Recommendation:** Remove `margin-bottom` from SearchInput, Collapsible, and FilterBanner. Consumers should use `gap` on their container or wrap in a spacing element.

---

## 4. Font Property Inventory

### 4a. font-family

All shared components that declare `font-family` use tokens correctly:

| File | Element | Value |
|------|---------|-------|
| `base.css` | `body` | `var(--font-body)` |
| `base.css` | `code, pre` | `var(--font-mono)` |
| `Button.svelte` | `.btn` | `var(--font-body)` |
| `Input.svelte` | `.input` | `var(--font-body)` |
| `Select.svelte` | `.select` | `var(--font-body)` |
| `SearchInput.svelte` | `.search-input` | `var(--font-body)` |
| `ListItem.svelte` | `.list-item` | `var(--font-mono)` |
| `Collapsible.svelte` | `.collapsible-title` | `var(--font-mono)` |
| `ThemeSwitcher.svelte` | `.theme-btn` | `var(--font-mono)` |
| Yggdrasil `+page.svelte` | `.tab-btn` | `var(--font-mono)` |

App components that declare `font-family` all use tokens. No hardcoded font stacks were found outside of `tokens.css` definitions and one intentional override in KvasirView (the `FONT_CSS` map for user-selectable viewer fonts -- this is a feature, not a violation).

### 4b. font-weight

Used values across the codebase:

| Value | Count | Files |
|-------|-------|-------|
| `bold` | 4 | SidebarLayout, StatCard, MarkdownPreview (th), SvalinnView |
| `700` | 9 | HlidskjalfView (title, stat, code, kind), QualityReport, TrafficReport, SchemaInspector |
| `800` | 4 | HlidskjalfView, SvalinnView, KvasirView, RatatoskrView (app-name) |
| `600` | 4 | Panel, ThemeSwitcher, SchemaInspector, TableViewer |
| `500` | 4 | Badge, FormatControls, Yggdrasil tab-btn |
| `400` | 1 | SchemaInspector (section-nullable) |
| `300` | 4 | HlidskjalfView, SvalinnView, KvasirView, RatatoskrView (separator/subtitle) |

**Inconsistency:** `bold` and `700` are used interchangeably. They are CSS-equivalent but mixing named and numeric weights is a style inconsistency. The token system does not define weight tokens (e.g., `--weight-bold`, `--weight-semibold`). Consider adding them.

### 4c. font-size token compliance

| Token Used | Count | Example Files |
|------------|-------|---------------|
| `var(--text-xs)` | ~30 | HlidskjalfView, QualityReport, TrafficReport, KvasirView, SvalinnView |
| `var(--text-sm)` | ~25 | All views, Button, Input, Select, SearchInput, ListItem, TreeNode |
| `var(--text-lg)` | 4 | All view headers (h1) |
| `var(--text-xl)` | 1 | HlidskjalfView stat-value |
| `var(--text-2xl)` | 1 | StatCard |
| `var(--text-3xl)` | 1 | MarkdownPreview h1 |
| `var(--text-base)` | 1 | RatatoskrView stats h3 |
| Hardcoded | ~20 | SchemaInspector (13), TrafficReport (3), ThemeSwitcher (1), SidebarLayout (1), Yggdrasil (2) |

**Compliance rate:** Approximately 75% of font-size declarations use tokens. SchemaInspector alone accounts for most violations.

### 4d. line-height

| File | Element | Value |
|------|---------|-------|
| `base.css` | `body` | `1.5` |
| `SidebarLayout.svelte` | `.sidebar-close` | `1` |
| `yggdrasil/+page.svelte` | `.tab-btn` | `1` |
| `yggdrasil/+page.svelte` | `.tab-char` | `0.9rem` |
| HlidskjalfView | (none declared -- inherits body 1.5) | |
| QualityReport | `.edu-row` | `1.5` (redundant with body) |
| TrafficReport | `.section-text` | `1.4` |
| KvasirView | `.code-viewer pre` | `1.6` |
| MarkdownPreview | `.markdown-preview` | `1.7` |
| SchemaInspector | `.section-body` | `1.6` |
| JsonlViewer | `.code-viewer pre` | `1.6` |
| TableViewer | `table` | `1.4` |
| SvalinnView | `.up-btn` | `1.4` |
| KvasirView | `.up-btn` | `1.4` |
| JsonlViewer | `.nav-btn` | `1` |

No `line-height` tokens are defined. The values range from `1` to `1.7`. This is a gap in the token system -- consider adding `--leading-tight: 1`, `--leading-normal: 1.5`, `--leading-relaxed: 1.7` tokens.

---

## 5. z-index Map

Only 4 z-index values exist in the entire codebase:

| File | Element | z-index | Purpose |
|------|---------|---------|---------|
| `ui/components/SidebarLayout.svelte` | `.sidebar` | **100** | Fixed sidebar overlay |
| `ui/components/SidebarLayout.svelte` | `.resize-handle` | **101** | Resize handle above sidebar |
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | `.col-source` | **1** | Hover pill above row content |
| `kvasir/src/lib/SchemaInspector.svelte` | `.controls` | **10** | Sticky controls bar |
| `kvasir/src/lib/TableViewer.svelte` | `thead` | **1** | Sticky table header |

**Assessment:** z-index usage is minimal and well-coordinated. The sidebar (100/101) is clearly above page content z-indexes (1/10). No conflicts. No tokens needed at this scale, but if the codebase grows, consider defining `--z-overlay: 100`, `--z-sticky: 10`, `--z-above: 1`.

---

## 6. Specificity Issues

### 6a. :global() Usage

Only one `:global()` selector exists outside of MarkdownPreview:

| File | Selector | Purpose | Risk |
|------|----------|---------|------|
| `yggdrasil/src/routes/+page.svelte:160` | `.view-pane :global(main)` | Override SidebarLayout/RatatoskrView `main` height from 100vh to 100% | **Fragile.** This reaches into child components' DOM. If a view changes its root element from `<main>` to `<div>`, this breaks silently. |
| `kvasir/src/lib/KvasirView.svelte:826-843` | `.viewer-settings :global(.code-viewer pre)`, `:global(table)`, `:global(.markdown-preview)`, `:global(.section-body)` | Pass font-size/font-family CSS custom properties to child components | **Intentional.** These propagate viewer settings to sub-components. Well-scoped to `.viewer-settings` container. |
| `kvasir/src/lib/MarkdownPreview.svelte:18-119` | 16 `:global()` selectors | Style `@html` rendered markdown content | **Correct.** `@html` content is not scoped by Svelte, so `:global()` is required. All are scoped to `.markdown-preview` parent. |

### 6b. !important

Zero `!important` declarations. Excellent.

### 6c. Deep nesting

The deepest selectors are 2-3 levels, which is fine:
- `.event-row:hover .col-expand-hint` (HlidskjalfView)
- `.event-base:hover .col-source` (HlidskjalfView)
- `.severity-error .group-code` (QualityReport)
- `.stat.high .stat-value` (HlidskjalfView)
- `.content-section.thinking .section-text` (TrafficReport)
- `.code-viewer.wrap79 .line-content` (KvasirView, JsonlViewer)

No problematic specificity escalation patterns found.

---

## 7. Media Queries / Responsive CSS

**Zero media queries in the entire codebase.** No `@media` declarations exist in any `.svelte` or `.css` file.

This is architecturally consistent -- these are Tauri desktop apps with controlled window sizes. However:

- No adaptation for narrow windows (e.g., when the user resizes the Tauri window small)
- SidebarLayout has a drag-resize handle for the sidebar, providing manual responsiveness
- The `max-width: 1200px` on SidebarLayout's `.main-content` provides some width constraint

**Assessment:** Acceptable for current Tauri-only deployment. Would need attention if any web deployment is considered.

---

## 8. Duplicated CSS Patterns

Several CSS patterns are duplicated identically across multiple components:

### 8a. App Header Pattern (4 files)

The header with `.app-name`, `.separator`/`.sep`, `.subtitle` is duplicated in:
- `HlidskjalfView.svelte` (lines 481-506)
- `SvalinnView.svelte` (lines 534-564)
- `KvasirView.svelte` (lines 681-710)
- `RatatoskrView.svelte` (lines 413-443)

All four use identical styles: `.app-name { font-weight: 800; letter-spacing: 0.12em }`, `.separator { font-weight: 300; opacity: 0.5; color: var(--text-secondary) }`, `.subtitle { font-weight: 300; font-size: var(--text-sm); color: var(--text-secondary); letter-spacing: 0.04em }`.

**Recommendation:** Extract an `AppHeader` shared component.

### 8b. Up Button Pattern (2 files)

`.up-btn` styles are identical in SvalinnView and KvasirView.

### 8c. Code Viewer Pattern (2 files)

`.code-viewer`, `.line-number`, `.line-content`, and wrap variants are duplicated identically between KvasirView and JsonlViewer.

**Recommendation:** Extract a `CodeBlock` shared component.

### 8d. Empty State Pattern (4 files)

`.empty-state` with `text-align: center; padding: 4rem var(--space-2xl/3xl); color: var(--text-secondary)` appears in SvalinnView, KvasirView, RatatoskrView, and JsonlViewer. The padding varies between `4rem var(--space-2xl)` and `4rem var(--space-3xl)` -- inconsistent AND uses a hardcoded `4rem`.

### 8e. Error Banner Pattern (2 files)

`.error-banner` is duplicated in KvasirView and RatatoskrView with identical styles.

### 8f. Directory Input Pattern (2 files)

`.directory-input` and `.directory-row` are duplicated in SvalinnView and KvasirView.

---

## 9. Theme Awareness Gaps

### 9a. Workspace Colors Not Theme-Aware

`HlidskjalfView.svelte` defines `WORKSPACE_HUES` as 6 hardcoded HSL values:
```
hsl(210, 70%, 55%), hsl(35, 85%, 55%), hsl(0, 70%, 55%),
hsl(270, 60%, 60%), hsl(160, 60%, 45%), hsl(320, 60%, 55%)
```

These are mid-saturation colors designed for dark backgrounds. On the light theme (`--bg-primary: #ffffff`), these colors will have reduced contrast. On warm-dark, the blue hue (210) clashes with the warm palette.

**Recommendation:** Define `--workspace-color-1` through `--workspace-color-6` in `tokens.css` with theme-specific overrides.

### 9b. D3 Graph Colors

RatatoskrView reads CSS custom properties at runtime via `getComputedStyle()` for node/edge colors. This correctly responds to theme changes. Good pattern.

---

## 10. Recommendations

### Critical (architectural)

1. **Fix the 100vh conflict.** Change HlidskjalfView `.watchtower` and RatatoskrView `main` to use `height: 100%` instead of `100vh`. Add a wrapping `div` with `height: 100vh` in each standalone `+page.svelte` if needed. Remove the `:global(main)` hack from Yggdrasil.

2. **Remove external margins from shared components.** SearchInput, Collapsible, and FilterBanner should not have `margin-bottom`. Let consumers control spacing.

### High (token compliance)

3. **Refactor SchemaInspector font sizes to use tokens.** Either use existing tokens (`--text-xs`, `--text-sm`) or add 1-2 new tokens if the sizes between xs and sm are needed (e.g., `--text-2xs: 0.8rem`).

4. **Make workspace colors theme-aware.** Move them to CSS custom properties in `tokens.css` with per-theme overrides.

5. **Add missing tokens:**
   - `--space-2xs: 0.125rem` (2px) -- for the many `1px 6px` and `2px 10px` patterns
   - `--text-2xs: ~0.8rem` (if SchemaInspector's intermediate sizes are justified)
   - `--leading-tight: 1`, `--leading-normal: 1.5`, `--leading-relaxed: 1.7` for line-height
   - `--content-max-width: 1200px` (SidebarLayout)
   - `--tab-strip-width: 28px` (Yggdrasil)

### Medium (consistency)

6. **Standardize font-weight usage.** Pick either named (`bold`) or numeric (`700`) and use it consistently. Consider adding weight tokens.

7. **Extract duplicated CSS patterns into shared components:**
   - `AppHeader` (4 views)
   - `CodeBlock` (KvasirView + JsonlViewer)
   - `EmptyState` (4 files)

8. **Replace hardcoded `4rem` in empty-state padding** with a spacing token (define `--space-4xl: 4rem` or use `--space-3xl` consistently).

### Low (cleanup)

9. **Replace exact-match hardcoded values with their tokens:**
   - `1.5rem` in SidebarLayout -> `var(--text-xl)`
   - `0.75rem` in Yggdrasil -> `var(--text-xs)`
   - `gap: 4px` in HlidskjalfView -> `var(--space-xs)`
   - `margin-left: 4px` in SchemaInspector -> `var(--space-xs)`

10. **Add `box-sizing: border-box`** to SearchInput (it declares `box-sizing: border-box` explicitly, but this is redundant with the `*` reset in `base.css`). Remove the redundant declaration.
