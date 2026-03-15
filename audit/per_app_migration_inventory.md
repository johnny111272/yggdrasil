# Per-App Migration Inventory: Shared UI Compliance Audit

**Generated:** 2026-03-15
**Scope:** All 5 Yggdrasil apps vs. `@yggdrasil/ui` shared library (13 components + design tokens)

---

## Executive Summary

The shared UI library (`@yggdrasil/ui`) provides 13 components and a comprehensive design token system across 4 themes. Adoption varies wildly:

| App | Shared Imports | Raw HTML Elements | Hardcoded Values | Estimated Removable CSS Lines |
|-----|---------------|-------------------|------------------|-------------------------------|
| **Hlidskjalf** | 0 components | 25+ buttons, 3 stat displays, 1 checkbox, 1 empty state | 16 | ~350 |
| **Kvasir** | 3 (Button, SidebarLayout, TreeNode) | 18+ buttons, 4 inputs, 2 selects, 6+ empty states | 12 | ~200 |
| **Svalinn** | 6 (Button, SidebarLayout, TreeNode, StatCard, SearchInput, FilterBanner) | 4 buttons, 1 input, 2 selects, 1 details/summary | 3 | ~80 |
| **Ratatoskr** | 1 (Button) | 0 additional buttons, 1 error banner, 1 empty state | 3 | ~30 |
| **Yggdrasil** | 1 (ThemeSwitcher) | 4 tab buttons, 1 clear button | 0 | ~10 (tab strip is app-specific) |

**Total across all apps:** ~670 lines of CSS that duplicate shared component styling.

Hlidskjalf is the worst offender with zero shared component imports. Kvasir has partial adoption but massive local duplication in sub-components. Svalinn is the reference implementation for shared UI usage.

---

## 1. Hlidskjalf (Agent Watchtower)

### 1.1 Current Shared Component Imports

**None.** Zero imports from `@yggdrasil/ui`. This is the worst offender.

The `+layout.svelte` imports `@yggdrasil/ui/css/tokens.css` and `@yggdrasil/ui/css/base.css` (correct), but no components are used anywhere.

### 1.2 Local UI Code That Duplicates Shared Components

#### Buttons (should be `Button`)

| File | Lines | Current Code | Notes |
|------|-------|-------------|-------|
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 327-333 | `<button class="filter-btn">all</button>` | Filter toggle button, repeated for each kind |
| Same | 335-342 | `<button class="filter-btn">` per kind | Kind filter buttons (5-6 instances) |
| Same | 345-352 | `<button class="filter-btn" ... >expand</button>` | Expand/collapse toggle |
| Same | 357-364 | `<button class="voice-btn">` | Voice/speech toggle button |
| Same | 365 | `<button class="clear-btn" onclick={clearFeed}>clear</button>` | Clear feed button |
| Same | 402 | `<button class="col-workspace workspace-link">` | Workspace clickable link (styled as button) |
| Same | 409 | `<button class="col-workspace workspace-link">` | Another workspace link |
| Same | 411 | `<button class="detail-path-link">` | Clickable path segments in detail |
| `hlidskjalf/src/lib/QualityReport.svelte` | 131-135 | `<button class="location-link">` | File location links (multiple instances) |
| Same | 138-142 | `<button class="location-link">` | Another location link variant |
| Same | 148-150 | `<button class="location-link">` | Third location link variant |
| `hlidskjalf/src/lib/TrafficReport.svelte` | 176-181 | `<div class="section-header" onclick=...>` | Clickable section headers (div acting as button) |
| Same | 196-204 | `<div class="section-header" onclick=...>` | Tools section header |
| Same | 213-218 | `<div class="tool-summary" onclick=...>` | Tool entry toggles |

CSS for these buttons spans lines 560-631 (72 lines) in HlidskjalfView.svelte.

#### Stat Display (should be `StatCard`)

| File | Lines | Current Code |
|------|-------|-------------|
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 310-322 | Three inline stat blocks: total events, high count, critical count |

CSS for stats spans lines 525-546 (22 lines). These are functionally identical to `StatCard` but rendered inline in the header.

#### Checkbox Input (should be `Input` or dedicated toggle)

| File | Lines | Current Code |
|------|-------|-------------|
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 353-356 | `<label class="auto-scroll-toggle"><input type="checkbox">` |

CSS for auto-scroll toggle: lines 586-593 (8 lines).

#### Checkbox Controls (should be shared toggle component)

| File | Lines | Current Code |
|------|-------|-------------|
| `hlidskjalf/src/lib/SchemaInspector.svelte` | N/A (in Kvasir, not Hlidskjalf) | - |

#### Empty State (should be shared `EmptyState` component)

| File | Lines | Current Code |
|------|-------|-------------|
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 371-375 | `<div class="empty">` with icon and text |

CSS: lines 641-651 (11 lines).

#### Badge-like Elements (should be `Badge`)

| File | Lines | Current Code |
|------|-------|-------------|
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 423-427 | `<span class="expanded-badge">` (3 instances in expanded metadata) |
| `hlidskjalf/src/lib/TrafficReport.svelte` | 153 | `<span class="traffic-kind">` kind badge |
| Same | 155 | `<span class="injection-badge">injection</span>` |
| Same | 160-161 | `<span class="field-pip">` (multiple instances) |

#### Collapsible/Details (should be `Collapsible`)

| File | Lines | Current Code |
|------|-------|-------------|
| `hlidskjalf/src/lib/TrafficReport.svelte` | 172-190 | Manual expand/collapse pattern with `toggleSection()`, `isOpen` state, arrow icons |
| Same | 193-230 | Tool list expand/collapse with `toggleTool()` |
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 382-447 | Row expand/collapse with `toggleRow()`, `expandedRows` Set |

These implement custom collapsible behavior that partially overlaps with the `Collapsible` component, though the use case (inline row expansion) may warrant a separate component.

### 1.3 Hardcoded Values That Should Be Tokens

| File | Line | Current Value | Correct Token |
|------|------|--------------|---------------|
| `HlidskjalfView.svelte` | 510 | `padding: 2px 8px` | `padding: var(--space-xs) var(--space-md)` |
| Same | 563 | `padding: 2px 10px` | `padding: var(--space-xs) var(--space-lg)` (approx) |
| Same | 592 | `gap: 4px` | `gap: var(--space-xs)` |
| Same | 598 | `padding: 2px 10px` | `padding: var(--space-xs) var(--space-lg)` |
| Same | 619 | `padding: 2px 10px` | `padding: var(--space-xs) var(--space-lg)` |
| Same | 650 | `font-size: 48px` | `font-size: var(--text-3xl)` or `3rem` |
| Same | 699 | `min-height: 1.6em` | Could use spacing token |
| Same | 713 | `min-height: 1.2em` | Could use spacing token |
| Same | 721 | `min-width: 72px` | Hardcoded, could be a variable |
| Same | 735 | `min-width: 60px` | Hardcoded |
| Same | 743 | `padding: 1px 6px` | `padding: 1px var(--space-sm)` |
| Same | 753 | `min-width: 80px` | Hardcoded |
| Same | 825 | `padding: 1px 6px` | `padding: 1px var(--space-sm)` |
| Same | 853 | `padding: 1px 6px` | `padding: 1px var(--space-sm)` |
| Same | 865 | `max-height: 300px` | Hardcoded |
| `QualityReport.svelte` | 190 | `letter-spacing: 0.5px` | Hardcoded |
| Same | 274 | `padding: 2px 0` | `padding: var(--space-xs) 0` |
| Same | 282 | `padding: 1px 6px` | `padding: 1px var(--space-sm)` |
| Same | 284 | `margin-top: 2px` | `margin-top: var(--space-xs)` |
| `TrafficReport.svelte` | 269 | `padding: 2px 10px` | `padding: var(--space-xs) var(--space-lg)` |
| Same | 316 | `font-size: 9px` | Too small for tokens, but should be `var(--text-xs)` or smaller |
| Same | 318 | `width: 16px; height: 16px` | Hardcoded |
| Same | 322 | `border-radius: 2px` | `border-radius: var(--radius-sm)` |
| Same | 387 | `font-size: 9px` | Same issue |
| Same | 441 | `font-size: var(--text-xs)` but `.tool-icon` line 452: `font-size: 8px` | `8px` is hardcoded |
| Same | 469 | `max-height: 300px` | Hardcoded |

### 1.4 Features Trapped in This App That Should Be Shared

1. **Auto-scroll toggle** (lines 353-356, 124, 202-207): A scrollable feed with auto-scroll-to-bottom behavior. This pattern could be a shared `AutoScrollFeed` or an option on a shared `ScrollArea` component.

2. **Filter toggle bar** (lines 326-366): A row of toggleable filter buttons with "all/none" toggle, spacer, and auxiliary controls. This is more sophisticated than the existing `FilterBanner` but could become a shared `FilterBar` component.

3. **Priority color system** (lines 59-68, 655-689): Priority-based visual weight via CSS classes (`priority-critical`, `priority-high`, etc.) with border-left accents and background tints. The tokens exist (`--priority-high-tint`, `--priority-critical-tint`) but the class system is local.

4. **Status indicator pill** (lines 305-307, 508-518): Connected/disconnected status badge. Could be a shared `StatusBadge` component.

5. **Empty state with icon** (lines 371-375, 641-651): Centered empty state with large emoji icon and descriptive text. Multiple apps implement this pattern.

6. **Error banner** (not present in Hlidskjalf, but present in Kvasir/Ratatoskr): Hlidskjalf shows errors inline, but the pattern exists elsewhere and should be unified.

### 1.5 Migration Checklist

1. **Import shared Button**: Replace raw `<button class="filter-btn">` (HlidskjalfView:327-342) with `Button` variant="ghost" or a new "pill" variant. Requires adding a "pill" variant to Button (border-radius: full, compact padding).
2. **Import shared Button**: Replace `<button class="clear-btn">` (HlidskjalfView:365) with `Button` variant="ghost".
3. **Import shared Button**: Replace `<button class="voice-btn">` (HlidskjalfView:357-364) with `Button` variant="ghost".
4. **Create shared StatCard usage**: Replace inline stat blocks (HlidskjalfView:310-322) with `StatCard` components. The current header layout may need a compact variant of StatCard.
5. **Create shared EmptyState component**: Extract empty state pattern (HlidskjalfView:371-375) into shared library. Use it here and in all other apps.
6. **Create shared StatusBadge component**: Extract status pill (HlidskjalfView:305-307) into shared library.
7. **Replace hardcoded pixel values**: Fix all `2px`, `10px`, `4px`, `6px`, `48px` padding/margin values with design tokens (see table in 1.3).
8. **Consider shared FilterBar component**: The filter toggle row pattern (HlidskjalfView:326-366) is Hlidskjalf-specific for now but could become shared as more apps need filtering.
9. **Assess Collapsible usage**: The row expand/collapse pattern (HlidskjalfView:382-447) and section expand/collapse in TrafficReport (172-230) could potentially use `Collapsible`, but the inline-row use case may be too different.
10. **Replace Badge-like spans in TrafficReport**: `injection-badge`, `field-pip`, and `traffic-kind` badges could use `Badge` component (needs text content support, not just count).

### 1.6 Estimated Scope

- **CSS lines that duplicate shared components:** ~350 lines across HlidskjalfView.svelte (416 CSS lines), QualityReport.svelte (186 CSS lines), TrafficReport.svelte (242 CSS lines)
- **Buttons that should be shared:** 25+ raw button elements
- **After migration:** Approximately 200-250 CSS lines could be removed

---

## 2. Kvasir (Workspace Inspector)

### 2.1 Current Shared Component Imports

**KvasirView.svelte** (line 6):
```
import { Button, SidebarLayout, TreeNode } from "@yggdrasil/ui";
```

**JsonlViewer.svelte** (line 3):
```
import { Button } from "@yggdrasil/ui";
```

Total: 3 unique shared components used (Button, SidebarLayout, TreeNode). Missing: Badge, Input, Select, StatCard, SearchInput, FilterBanner, Collapsible, Panel, ListItem, ThemeSwitcher.

### 2.2 Local UI Code That Duplicates Shared Components

#### Buttons Not Using `Button` Component

| File | Lines | Current Code | Notes |
|------|-------|-------------|-------|
| `KvasirView.svelte` | 414 | `<button class="up-btn">` | Up-navigation button in sidebar header |
| Same | 474-488 | `<button class="tab">` (x6+) | Tab buttons for Code/Preview/Data/Inspect |
| Same | 476-479 | `<button class="tab font-btn">` (x3) | Font size controls |
| Same | 481-488 | `<button class="tab wrap-toggle">` | Wrap mode toggle |
| Same | 517-523 | Same tab/font/wrap buttons repeated for table view |
| Same | 555-602 | Same tab/font/wrap buttons repeated for regular file view |
| `FormatControls.svelte` | 30-37 | `<button class="format-btn">` (x5) | Format selector buttons (JSON/YAML/TOML/TOON/RON) |
| `JsonlViewer.svelte` | 168-175 | `<button class="nav-btn">` (x4) | Navigation buttons (first/prev/next/last) |
| Same | 193-199 | `<button class="format-btn">` (x5) | Format selector (duplicated from FormatControls) |
| `TableViewer.svelte` | 149 | `<button class="export-btn">Export CSV</button>` | Export button |
| `SchemaInspector.svelte` | 90-92 | `<button class="section-header">` | Section toggle buttons |
| Same | 142-143 | `<button class="ext ... badge-btn">` | Extension badge toggle buttons (x4 types) |
| Same | 219-229 | Multiple `<button class="badge-btn">` | More extension toggles |

#### Inputs Not Using `Input` Component

| File | Lines | Current Code |
|------|-------|-------------|
| `KvasirView.svelte` | 442-448 | `<input type="text" class="directory-input">` | Directory path input |
| `TableViewer.svelte` | 139-144 | `<input type="text" class="table-filter">` | Row filter input |
| `SchemaInspector.svelte` | 47-65 | `<input type="checkbox">` (x5) | Control checkboxes |
| `JsonlViewer.svelte` | 181-189 | `<input type="range" class="jsonl-scrubber">` | Range slider (no shared equivalent) |

#### Selects Not Using `Select` Component

None found in Kvasir (format selection uses button groups instead).

#### Empty States (should be shared)

| File | Lines | Current Code |
|------|-------|-------------|
| `KvasirView.svelte` | 627-629 | `<section class="empty-state">Select a file...</section>` |
| Same | 630-632 | `<section class="empty-state">Select a directory...</section>` |
| `JsonlViewer.svelte` | 212-214 | `<section class="empty-state">Empty JSONL file</section>` |

#### Error Banner (should be shared)

| File | Lines | Current Code |
|------|-------|-------------|
| `KvasirView.svelte` | 457-459 | `<section class="error-banner">{error}</section>` |
| `TableViewer.svelte` | 157 | `<div class="table-status table-error">{error}</div>` |

#### Panel-like Containers (could use `Panel`)

| File | Lines | Current Code |
|------|-------|-------------|
| `KvasirView.svelte` | 464-472 | `<section class="file-info">` | File info panel with header |
| Same | 506-515 | Same for table files |
| Same | 537-551 | Same for regular files |
| `FormatControls.svelte` | 27-58 | `<section class="data-controls">` | Data controls panel |
| `JsonlViewer.svelte` | 166-204 | `<section class="jsonl-controls">` | JSONL navigation panel |

#### Code Viewer Duplication

The `.code-viewer` CSS pattern is duplicated identically between:
- `KvasirView.svelte` lines 845-888 (44 lines)
- `JsonlViewer.svelte` lines 313-355 (43 lines)

This is a direct copy-paste duplication. Both have identical `.code-viewer`, `.line-number`, `.line-content`, `.wrap79`, and `.wrapwidth` styles.

### 2.3 Hardcoded Values That Should Be Tokens

| File | Line | Current Value | Correct Token |
|------|------|--------------|---------------|
| `KvasirView.svelte` | 802 | `gap: 2px` | `gap: var(--space-xs)` |
| Same | 899 | `padding: 4rem var(--space-3xl)` | `4rem` is hardcoded, use `var(--space-3xl)` for both or add `--space-4xl` |
| `JsonlViewer.svelte` | 357 | `padding: 4rem var(--space-3xl)` | Same issue |
| `TableViewer.svelte` | 252 | `padding: var(--space-3xl)` | Fine, but 300px not a token issue |
| `SchemaInspector.svelte` | 310 | `font-size: 0.8rem` | Should be `var(--text-xs)` (0.75rem) or create `--text-xs-plus` |
| Same | 325 | `font-size: 1rem` | `var(--text-base)` |
| Same | 331 | `font-size: 0.8rem` | `var(--text-xs)` |
| Same | 339 | `font-size: 0.8rem` | `var(--text-xs)` |
| Same | 358 | `font-size: 0.9rem` | Between `--text-sm` (0.875rem) and `--text-base` (1rem) |
| Same | 371 | `font-size: 0.7rem` | Below token range |
| Same | 385 | `font-size: 0.82rem` | Between `--text-xs` and `--text-sm` |
| Same | 398 | `font-size: 0.8rem` | `var(--text-xs)` |
| Same | 401 | `font-size: 0.78rem` | Below token range |
| Same | 413-428 | `font-size: 0.78rem` (x5) | Below token range |
| `FormatControls.svelte` | none significant | Uses tokens well | - |

### 2.4 Features Trapped in This App That Should Be Shared

1. **Font size controls** (KvasirView lines 106-108, 476-479, 589-593): Up/down font size buttons with display. Repeated 3 times in KvasirView (JSONL view, table view, regular file view). Should be a shared `FontSizeControls` component.

2. **Font family cycling** (KvasirView lines 93-104): Mono/Dyslexie/Sans/Serif cycling. Accessibility feature that belongs in the shared library.

3. **Wrap mode cycling** (KvasirView lines 81-91): No-wrap/wrap-79/wrap-to-width. Used for code display. Should be shared with any code viewer.

4. **Code viewer with line numbers** (KvasirView lines 620-622, JsonlViewer lines 207-209): Highlighted code display with line numbers and wrap modes. Duplicated across two components. Should be a shared `CodeViewer` component.

5. **Format selector buttons** (FormatControls lines 29-38, JsonlViewer lines 192-199): JSON/YAML/TOML/TOON/RON format toggle bar. Duplicated between two components.

6. **Empty state** (KvasirView lines 627-632, JsonlViewer lines 212-214): Same pattern as Hlidskjalf. Should be shared.

7. **Error banner** (KvasirView lines 457-459): Red error banner with message. Same pattern exists in Ratatoskr. Should be shared.

8. **Up-button in sidebar** (KvasirView line 414): Same `<button class="up-btn">` as in SvalinnView. Duplicated CSS.

### 2.5 Migration Checklist

1. **Replace directory input** (KvasirView:442-448) with shared `Input` component.
2. **Replace table filter input** (TableViewer:139-144) with shared `Input` or `SearchInput`.
3. **Replace up-btn** (KvasirView:414) with shared `Button variant="ghost"`. Identical code in SvalinnView.
4. **Replace export-btn** (TableViewer:149) with shared `Button`.
5. **Replace nav-btn buttons** (JsonlViewer:168-175) with shared `Button`.
6. **Create shared CodeViewer component**: Extract `.code-viewer` + `.line-number` + `.line-content` + wrap mode support from KvasirView (845-888) and JsonlViewer (313-355). Saves ~87 lines of duplicated CSS.
7. **Create shared FontSizeControls component**: Extract font size up/down/display from KvasirView (476-479). Eliminates 3x duplication within KvasirView.
8. **Create shared FormatSelector component**: Extract format button bar from FormatControls (29-38) and JsonlViewer (192-199). Eliminates duplication.
9. **Create shared EmptyState component**: Extract from KvasirView (627-632) and JsonlViewer (212-214).
10. **Create shared ErrorBanner component**: Extract from KvasirView (457-459).
11. **Replace tab buttons** (KvasirView:555-602) with shared `Button` or create shared `TabBar` component. The tab pattern is repeated 3 times within KvasirView itself.
12. **Replace hardcoded font sizes in SchemaInspector**: All `0.78rem`, `0.8rem`, `0.82rem`, `0.9rem` values should map to `var(--text-xs)` or `var(--text-sm)`.
13. **Consider Panel for file-info sections** (KvasirView:464-472, 506-515, 537-551): These panel-like sections could use the shared `Panel` component.

### 2.6 Estimated Scope

- **CSS lines that duplicate shared components:** ~200 lines across KvasirView.svelte (263 CSS lines), JsonlViewer.svelte (145 CSS lines), TableViewer.svelte (127 CSS lines), FormatControls.svelte (79 CSS lines), SchemaInspector.svelte (176 CSS lines), MarkdownPreview.svelte (112 CSS lines)
- **Internal duplication:** CodeViewer CSS duplicated 2x (87 lines), format selector duplicated 2x, font controls duplicated 3x
- **After migration:** Approximately 150-180 CSS lines could be removed

---

## 3. Svalinn (Code Quality Viewer)

### 3.1 Current Shared Component Imports

**SvalinnView.svelte** (line 6):
```
import { Button, SidebarLayout, TreeNode, StatCard, SearchInput, FilterBanner } from "@yggdrasil/ui";
```

Total: 6 shared components. This is the best adoption rate. Missing: Badge (uses `.group-count` instead), Input (uses raw `<input>`), Select (uses raw `<select>`), Collapsible (uses raw `<details>`), Panel, ListItem.

### 3.2 Local UI Code That Duplicates Shared Components

#### Buttons Not Using `Button`

| File | Lines | Current Code | Notes |
|------|-------|-------------|-------|
| `SvalinnView.svelte` | 360 | `<button class="up-btn">` | Up-navigation (same as Kvasir) |
| Same | 421 | `<button class="view-btn">By File</button>` | View mode toggle (x3) |
| Same | 422 | `<button class="view-btn">By Error Type</button>` | |
| Same | 423 | `<button class="view-btn">By Tool</button>` | |

CSS for these: lines 513-637 (view-btn lines 622-637 = 16 lines, up-btn lines 513-532 = 20 lines).

#### Input Not Using `Input`

| File | Lines | Current Code |
|------|-------|-------------|
| `SvalinnView.svelte` | 387-392 | `<input type="text" bind:value={directory} class="directory-input">` |
| Same | 403 | `<input type="checkbox" bind:checked={includeTests}>` |

#### Selects Not Using `Select`

| File | Lines | Current Code |
|------|-------|-------------|
| `SvalinnView.svelte` | 429-435 | `<select bind:value={severityFilter}>` with 5 options |
| Same | 438-444 | `<select bind:value={toolFilter}>` with dynamic options |

CSS for selects: lines 650-656 (7 lines). This is a direct match for the `Select` shared component.

#### Details/Summary Not Using `Collapsible`

| File | Lines | Current Code |
|------|-------|-------------|
| `SvalinnView.svelte` | 460-503 | `<details class="group">` with `<summary>` | Issue group collapsible sections |

CSS for `.group` and `.group summary`: lines 663-691 (29 lines). The `Collapsible` component provides this exact pattern: title + badge count + expandable content.

#### Badge-like Elements Not Using `Badge`

| File | Lines | Current Code |
|------|-------|-------------|
| `SvalinnView.svelte` | 463 | `<span class="group-count">{issues.length}</span>` | Pill-shaped count badge |

CSS: lines 687-691 (5 lines). This is exactly what `Badge` does.

#### Empty State (should be shared)

| File | Lines | Current Code |
|------|-------|-------------|
| `SvalinnView.svelte` | 506-508 | `<section class="empty-state">` |

### 3.3 Hardcoded Values That Should Be Tokens

| File | Line | Current Value | Correct Token |
|------|------|--------------|---------------|
| `SvalinnView.svelte` | 774 | `padding: 4rem var(--space-2xl)` | `4rem` hardcoded |
| Same | 536 | `text-align: center` on header | Not a token issue, but inconsistent with other apps |

Svalinn is generally excellent at using tokens. Almost all spacing, colors, and typography use the design token system.

### 3.4 Features Trapped in This App That Should Be Shared

1. **View mode toggle bar** (lines 419-424): By File / By Error Type / By Tool toggle. This is a segmented control / button group pattern. Could be a shared `ButtonGroup` or `SegmentedControl` component.

2. **Up-button** (line 360): Identical to Kvasir's up-button. Same CSS, same behavior. Should be extracted to shared.

3. **Issue list with educational detail** (lines 466-498): The issue row + expandable signal/direction/canary detail pattern. This is Svalinn-specific data, but the expandable-row pattern could be shared.

### 3.5 Migration Checklist

1. **Replace directory input** (SvalinnView:387-392) with shared `Input` component.
2. **Replace severity select** (SvalinnView:429-435) with shared `Select` component.
3. **Replace tool select** (SvalinnView:438-444) with shared `Select` component.
4. **Replace `<details class="group">` blocks** (SvalinnView:460-503) with shared `Collapsible` component. The `Collapsible` already supports `title`, `badgeCount`, `badgeSeverity`, and `open` props.
5. **Replace `.group-count` span** (SvalinnView:463) with shared `Badge` component (already used internally by `Collapsible`).
6. **Replace view mode toggle buttons** (SvalinnView:421-423) with shared `Button` components or create a shared `ButtonGroup`.
7. **Replace up-btn** (SvalinnView:360) with shared `Button variant="ghost"`.
8. **Create shared EmptyState component**: Use for SvalinnView:506-508.
9. **Fix `4rem` hardcoded padding** in empty-state (line 774).

### 3.6 Estimated Scope

- **CSS lines that duplicate shared components:** ~80 lines (select: 7, details/group: 29, view-btn: 16, up-btn: 20, empty-state: 5, directory-input: 8)
- **After migration:** Approximately 60-70 CSS lines could be removed

---

## 4. Ratatoskr (Graph Viewer)

### 4.1 Current Shared Component Imports

**RatatoskrView.svelte** (line 4):
```
import { Button } from "@yggdrasil/ui";
```

Total: 1 shared component. Missing: Panel (stats-panel/node-panel), StatCard (graph statistics), Badge.

### 4.2 Local UI Code That Duplicates Shared Components

#### Panel-like Containers (should be `Panel`)

| File | Lines | Current Code |
|------|-------|-------------|
| `RatatoskrView.svelte` | 336-365 | `<aside class="stats-panel">` | Graph statistics panel |
| Same | 369-388 | `<aside class="node-panel">` | Selected node detail panel |

CSS: lines 481-513 (33 lines). Both panels have identical base styling. The shared `Panel` component provides `header` + content slot pattern that matches.

#### Error Banner (should be shared)

| File | Lines | Current Code |
|------|-------|-------------|
| `RatatoskrView.svelte` | 327-329 | `<section class="error-banner">{error}</section>` |

CSS: lines 458-464 (7 lines). Identical pattern to Kvasir's error-banner.

#### Empty State (should be shared)

| File | Lines | Current Code |
|------|-------|-------------|
| `RatatoskrView.svelte` | 391-393 | `<section class="empty-state">` |

CSS: lines 565-571 (7 lines).

### 4.3 Hardcoded Values That Should Be Tokens

| File | Line | Current Value | Correct Token |
|------|------|--------------|---------------|
| `RatatoskrView.svelte` | 472 | `min-height: 400px` | Hardcoded |
| Same | 489 | `max-width: 250px` | Hardcoded |

Ratatoskr generally uses tokens well for all other values.

### 4.4 Features Trapped in This App That Should Be Shared

1. **Floating overlay panel** (stats-panel, node-panel): Absolute-positioned panels overlaying content. This is a pattern not currently in the shared library. Could be a shared `FloatingPanel` component.

2. **Definition list (dl/dt/dd) styling** (lines 515-532): Grid-based key-value display. This pattern could be a shared `PropertyList` or `KeyValueTable` component.

3. **Type dot indicator** (lines 548-553): Small colored circle next to text. Could be part of `Badge` or a shared `ColorDot` component.

### 4.5 Migration Checklist

1. **Replace error-banner** (RatatoskrView:327-329) with shared `ErrorBanner` component (needs to be created first).
2. **Create shared EmptyState component**: Use for RatatoskrView:391-393.
3. **Consider Panel for stats-panel** (RatatoskrView:336-365) and **node-panel** (RatatoskrView:369-388). The floating/overlay positioning is app-specific, but the content structure matches `Panel`.
4. **Fix hardcoded min-height and max-width** values.

### 4.6 Estimated Scope

- **CSS lines that duplicate shared components:** ~30 lines (error-banner: 7, empty-state: 7, overlapping panel styling: 16)
- **After migration:** Approximately 20-25 CSS lines could be removed
- Ratatoskr is a relatively small UI surface area dominated by the D3 graph SVG

---

## 5. Yggdrasil (Unified Shell)

### 5.1 Current Shared Component Imports

**+page.svelte** (line 6):
```
import { ThemeSwitcher } from "@yggdrasil/ui";
```

Total: 1 shared component. But this is intentionally minimal -- Yggdrasil is a thin shell.

### 5.2 Local UI Code That Duplicates Shared Components

#### Tab Buttons (app-specific pattern)

| File | Lines | Current Code |
|------|-------|-------------|
| `yggdrasil/src/routes/+page.svelte` | 106-129 | `<button class="tab-btn">` (x4 tabs + 1 clear button) |

CSS: lines 176-226 (51 lines). These are vertical rotated character buttons in a narrow sidebar strip. This is a highly app-specific pattern that does NOT match the shared `Button` component -- the tab buttons display each character vertically, which is a custom layout.

### 5.3 Hardcoded Values That Should Be Tokens

| File | Line | Current Value | Correct Token |
|------|------|--------------|---------------|
| `+page.svelte` | 165 | `width: 28px` | Hardcoded tab strip width |
| Same | 183 | `font-size: 0.75rem` | `var(--text-xs)` |
| Same | 191 | `padding: var(--space-xs) 2px` | `2px` hardcoded |
| Same | 198 | `height: 0.9rem` | Hardcoded |
| Same | 227 | `font-size: 0.75rem` | `var(--text-xs)` |

### 5.4 Features Trapped in This App That Should Be Shared

None significant. The Yggdrasil shell is intentionally thin. The vertical tab strip is a unique UI pattern specific to this app. The `ThemeSwitcher` is already shared.

### 5.5 Migration Checklist

1. **Replace hardcoded `font-size: 0.75rem`** (lines 183, 227) with `var(--text-xs)`.
2. **Replace hardcoded `2px` padding** (line 191) with token equivalent.
3. No component migrations needed -- the tab strip is intentionally app-specific.

### 5.6 Estimated Scope

- **CSS lines that could reference tokens better:** ~10 lines
- **No component-level migrations** needed
- The tab strip CSS (51 lines) is app-specific and should stay

---

## Cross-App Migration Priorities

### Priority 1: Create Missing Shared Components (Unblocks All Apps)

These new components would benefit multiple apps immediately:

| New Component | Apps That Need It | Estimated Savings |
|---------------|-------------------|-------------------|
| **EmptyState** | Hlidskjalf, Kvasir (x3), Svalinn, Ratatoskr | 6 duplicate implementations, ~35 CSS lines |
| **ErrorBanner** | Kvasir, Ratatoskr | 2 duplicate implementations, ~14 CSS lines |
| **CodeViewer** | Kvasir (x2 internal duplication) | 87 duplicated CSS lines within Kvasir alone |
| **FontSizeControls** | Kvasir (x3 internal duplication) | ~30 lines duplicated within KvasirView |

### Priority 2: Migrate Existing Component Usage (Highest ROI)

These migrations use components that already exist:

| Migration | App | Shared Component | Lines Saved |
|-----------|-----|-----------------|-------------|
| Replace `<select>` elements | Svalinn (x2) | `Select` | ~7 CSS lines |
| Replace `<details>` groups | Svalinn (x1) | `Collapsible` + `Badge` | ~34 CSS lines |
| Replace directory `<input>` | Kvasir, Svalinn | `Input` | ~16 CSS lines |
| Replace `<button class="up-btn">` | Kvasir, Svalinn | `Button variant="ghost"` | ~40 CSS lines (20 x 2 apps) |
| Replace view-mode buttons | Svalinn | `Button` | ~16 CSS lines |
| Replace export/nav buttons | Kvasir (TableViewer, JsonlViewer) | `Button` | ~30 CSS lines |

### Priority 3: Extend Shared Components (Needs Design Work)

| Extension | What's Needed | Apps Affected |
|-----------|--------------|---------------|
| **Button "pill" variant** | `border-radius: full`, compact padding | Hlidskjalf (filter-btn, voice-btn, clear-btn) |
| **Badge text variant** | Support text content, not just count | Hlidskjalf (expanded-badge, injection-badge) |
| **Panel floating variant** | Absolute-positioned overlay mode | Ratatoskr (stats-panel, node-panel) |
| **Shared tab bar** | Horizontal button group with active indicator | Kvasir (code/preview/data/inspect tabs) |

### Priority 4: Hlidskjalf Full Migration (Most Work, Most Reward)

Hlidskjalf has zero shared component usage and ~350 lines of CSS that largely duplicates shared patterns. A full migration would:
- Import Button, Badge, StatCard
- Add new EmptyState, StatusBadge components
- Replace all 25+ raw buttons
- Replace 3 inline stat displays
- Standardize all hardcoded pixel values to tokens

---

## Dependency Order

Migrations must happen in this order to avoid breaking changes:

### Phase 1: Create New Shared Components

1. **Create `EmptyState`** in `ui/components/EmptyState.svelte`
   - Props: `icon?: string`, `message: string`
   - Used by: all 5 apps

2. **Create `ErrorBanner`** in `ui/components/ErrorBanner.svelte`
   - Props: `message: string`, `onDismiss?: () => void`
   - Used by: Kvasir, Ratatoskr (Svalinn and Hlidskjalf handle errors differently)

3. **Create `CodeViewer`** in `ui/components/CodeViewer.svelte`
   - Props: `content: string`, `highlighted: string`, `wrapMode: 'nowrap' | 'wrap79' | 'wrapwidth'`
   - Used by: Kvasir (KvasirView + JsonlViewer)

4. **Create `FontSizeControls`** in `ui/components/FontSizeControls.svelte`
   - Props: `size: number` (bindable), `min?: number`, `max?: number`, `fontFamily?: string` (bindable), `families?: string[]`
   - Used by: Kvasir (3 internal duplications)

### Phase 2: Extend Existing Components

5. **Add "pill" variant to `Button`**
   - `variant: "primary" | "special" | "neutral" | "ghost" | "pill"`
   - Pill: `border-radius: var(--radius-full)`, compact padding, transparent background
   - Unblocks: Hlidskjalf filter buttons

6. **Add text mode to `Badge`**
   - Currently only takes `count: number`. Add `text?: string` prop for text badges.
   - Unblocks: Hlidskjalf expanded-badge, injection-badge

### Phase 3: App Migrations (Can Happen in Parallel per App)

7. **Svalinn migration** (smallest delta, reference implementation)
   - Replace `<select>` with `Select`
   - Replace `<details>` with `Collapsible`
   - Replace `<input>` with `Input`
   - Replace `up-btn` and `view-btn` with `Button`
   - Add `EmptyState`

8. **Ratatoskr migration** (small surface area)
   - Add `ErrorBanner`
   - Add `EmptyState`
   - Consider `Panel` for overlay panels

9. **Kvasir migration** (medium complexity)
   - Replace `<input>` with `Input`
   - Replace raw buttons with `Button`
   - Replace duplicated CodeViewer with shared `CodeViewer`
   - Replace duplicated FontSizeControls with shared component
   - Add `EmptyState`, `ErrorBanner`

10. **Hlidskjalf migration** (largest, most buttons)
    - Import `Button` (pill variant for filters, ghost for actions)
    - Import `StatCard` for header stats
    - Import `Badge` (text variant) for metadata badges
    - Add `EmptyState`
    - Standardize all hardcoded values to tokens

### Phase 4: Update Exports

11. **Update `ui/index.js`** to export new components:
    ```
    export { default as EmptyState } from './components/EmptyState.svelte';
    export { default as ErrorBanner } from './components/ErrorBanner.svelte';
    export { default as CodeViewer } from './components/CodeViewer.svelte';
    export { default as FontSizeControls } from './components/FontSizeControls.svelte';
    ```

---

## Appendix: Shared Pattern Catalog

### Patterns Duplicated Across Multiple Apps

| Pattern | Hlidskjalf | Kvasir | Svalinn | Ratatoskr | Yggdrasil |
|---------|-----------|--------|---------|-----------|-----------|
| App header (h1 with app-name :: subtitle) | local | local | local | local | N/A |
| Directory input | N/A | local | local | N/A | N/A |
| Up-button in sidebar | N/A | local | local | N/A | N/A |
| Error banner | N/A | local | N/A | local | N/A |
| Empty state | local | local (x3) | local | local | N/A |
| Filter/toggle buttons | local | local | local | N/A | N/A |
| Code viewer with line numbers | N/A | local (x2) | N/A | N/A | N/A |
| Font size controls | N/A | local (x3) | N/A | N/A | N/A |

### App Header Pattern

All 4 app views use the same header pattern:
```html
<h1>
  <span class="app-name">APPNAME</span>
  <span class="separator">::</span>
  <span class="subtitle">Description</span>
</h1>
```

With identical CSS:
```css
.app-name { font-weight: 800; letter-spacing: 0.12em; }
.separator { font-weight: 300; opacity: 0.5; color: var(--text-secondary); }
.subtitle { font-weight: 300; font-size: var(--text-sm); color: var(--text-secondary); letter-spacing: 0.04em; }
```

This is duplicated in Hlidskjalf (490-506), Kvasir (693-709), Svalinn (547-564), and Ratatoskr (427-443). That is ~68 lines of identical CSS across 4 files. A shared `AppHeader` component would eliminate all of it.
