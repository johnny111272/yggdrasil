# Component & Import Audit

**Date:** 2026-03-15
**Auditor:** Claude Opus 4.6
**Scope:** All shared UI components, all app view components, all route pages/layouts, all CSS across the Yggdrasil workspace.

---

## Executive Summary

**What's good:**
- The `@yggdrasil/ui` design system is well-structured with 13 components, comprehensive CSS tokens (4 themes), and clean base styles.
- All 5 apps correctly import `tokens.css` and `base.css` via their `+layout.svelte`.
- View components correctly use `./` relative imports (not `$lib/`) for sibling components, preserving the unified shell contract.
- No `$lib/` imports exist in any view component -- the documented anti-pattern has been avoided.
- CSS token usage is excellent: no hardcoded hex colors in any app's `<style>` blocks. All colors reference `var(--*)` tokens.
- The `commands` prop contract is implemented consistently across all 4 view components.
- Yggdrasil's `+page.svelte` correctly maps all command names with 4-letter prefixes.

**What's broken or suboptimal:**
- **9 of 13 shared components are unused by any app.** Only `Button`, `SidebarLayout`, `TreeNode`, `StatCard`, `SearchInput`, `FilterBanner`, and `ThemeSwitcher` are imported. `Badge`, `Input`, `Select`, `Panel`, `Collapsible`, and `ListItem` have zero consumers.
- **Significant CSS duplication across apps.** The app header pattern (`.app-name` + `.separator` + `.subtitle`) is copy-pasted identically in 4 views. The `.up-btn`, `.directory-input`, `.empty-state`, `.error-banner`, and `.code-viewer` patterns are duplicated across 2-3 apps each.
- **Hlidskjalf imports zero shared components.** It builds its entire UI from scratch, including stat cards, filter buttons, badges, and collapsible sections that overlap with shared components.
- **Hardcoded font sizes** in SchemaInspector (12 instances of raw `rem` values like `0.78rem`, `0.82rem`, `0.9rem`) that don't use `--text-*` tokens.
- **Hardcoded spacing** in Hlidskjalf and traffic/quality reports -- numerous `1px`, `2px`, `6px`, `8px`, `10px` values instead of `--space-*` tokens.
- **KvasirView duplicates `--font-mono` token value** in its `FONT_CSS` JS constant instead of reading from the CSS variable.

---

## 1. Shared Component Inventory

### 1.1 Component Table

| # | Component | Props | Internal Deps | Consumers | Non-Consumers |
|---|-----------|-------|---------------|-----------|----------------|
| 1 | **Button** | `variant` ("primary"\|"special"\|"neutral"\|"ghost"), `disabled`, `onclick`, `children` | none | SvalinnView, KvasirView, JsonlViewer, RatatoskrView | **HlidskjalfView** |
| 2 | **Badge** | `count`, `severity` ("blocked"\|"error"\|"warning"\|"success"\|"neutral") | none | TreeNode (internal), Collapsible (internal) | All 5 apps (no direct import) |
| 3 | **Input** | `value` (bindable), `placeholder`, `type`, `disabled`, `oninput` | none | **None** | All 5 apps |
| 4 | **Select** | `value` (bindable), `disabled`, `onchange`, `children` | none | **None** | All 5 apps |
| 5 | **Panel** | `header`, `children` | none | **None** | All 5 apps |
| 6 | **StatCard** | `value`, `label`, `severity` | none | SvalinnView | HlidskjalfView, KvasirView, RatatoskrView, Yggdrasil |
| 7 | **TreeNode** | `node`, `depth`, `selected`, `badgeCount`, `badgeSeverity`, `onToggle`, `onSelect`, `onDblClickDir`, `getBadgeCount`, `getBadgeSeverity`, `getIcon` | Badge | SvalinnView, KvasirView | HlidskjalfView, RatatoskrView |
| 8 | **Collapsible** | `title`, `badgeCount`, `badgeSeverity`, `open`, `children` | Badge | **None** | All 5 apps |
| 9 | **ListItem** | `onclick`, `children` | none | **None** | All 5 apps |
| 10 | **SearchInput** | `value` (bindable), `placeholder`, `oninput` | none | SvalinnView | HlidskjalfView, KvasirView, RatatoskrView, Yggdrasil |
| 11 | **SidebarLayout** | `showSidebar`, `sidebarTitle`, `onCloseSidebar`, `sidebar` (snippet), `children` (snippet), `headerExtra` (snippet), `fullWidth`, `minWidth`, `maxWidth` | none | SvalinnView, KvasirView | HlidskjalfView, RatatoskrView |
| 12 | **FilterBanner** | `label`, `value`, `onClear` | none | SvalinnView | HlidskjalfView, KvasirView, RatatoskrView, Yggdrasil |
| 13 | **ThemeSwitcher** | `orientation` ("vertical"\|"horizontal") | none | Yggdrasil `+page.svelte` | All 4 standalone apps |

### 1.2 Unused Components (Zero App Consumers)

These 6 components are exported from `ui/index.js` but never imported by any app:

| Component | Observation |
|-----------|-------------|
| **Input** | Apps use raw `<input>` elements with custom styling (SvalinnView line 389, KvasirView line 447) |
| **Select** | SvalinnView uses raw `<select>` elements (lines 429, 440) |
| **Panel** | Apps build their own panel-like sections (`.controls`, `.file-info`, `.stats-panel`) |
| **Collapsible** | SvalinnView uses raw `<details>` with custom styling (line 460). Kvasir's SchemaInspector has its own collapsible sections. |
| **ListItem** | No app uses a generic list item -- they all build custom list rows. |

**Badge** is only consumed indirectly via `TreeNode` and `Collapsible` (internal `./` imports within `ui/components/`). No app directly imports Badge.

### 1.3 Prop Contract Consistency

All consumers use shared component props correctly:

- **Button:** All 4 consumers use `variant`, `onclick`, `disabled`. No consumer passes unknown props.
- **SidebarLayout:** Both consumers (Svalinn, Kvasir) use `showSidebar`, `sidebarTitle`, `onCloseSidebar`, `sidebar` snippet, `children` snippet, `headerExtra` snippet. Kvasir additionally uses `fullWidth`. Both pass the same structural pattern.
- **TreeNode:** Both consumers pass `node`, `selected`, `onToggle`, `onSelect`, `onDblClickDir`. Svalinn also passes `getBadgeCount` and `getBadgeSeverity`. Kvasir passes `getIcon`.
- **StatCard:** Only Svalinn uses it (5 instances with `value`, `label`, `severity`).
- **SearchInput:** Only Svalinn uses it (`bind:value`, `placeholder`).
- **FilterBanner:** Only Svalinn uses it (`label`, `value`, `onClear`).
- **ThemeSwitcher:** Only Yggdrasil uses it (default `orientation`).

**Observation on TreeNode:** The `badgeSeverity` prop type in `TreeNode.Props` accepts `"info"` as a severity value, but the `Badge` component it delegates to does not recognize `"info"` -- it falls through to the default case (`var(--action-primary)`). The tokens.css file has no `--severity-info`. This is a silent type mismatch with no runtime error but unexpected styling.

---

## 2. Duplicated Functionality

### 2.1 App Header Pattern (4 apps)

All four views duplicate the same header structure: `<span class="app-name">NAME</span> <span class="separator">::</span> <span class="subtitle">Description</span>`

| File | Lines |
|------|-------|
| `/Users/johnny/.ai/smidja/yggdrasil/hlidskjalf/src/lib/HlidskjalfView.svelte` | 304, CSS 490-506 |
| `/Users/johnny/.ai/smidja/yggdrasil/svalinn/src/lib/SvalinnView.svelte` | 378, CSS 548-564 |
| `/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/lib/KvasirView.svelte` | 436, CSS 693-710 |
| `/Users/johnny/.ai/smidja/yggdrasil/ratatoskr/src/lib/RatatoskrView.svelte` | 303, CSS 427-443 |

The CSS is near-identical across all 4:
```css
.app-name { font-weight: 800; letter-spacing: 0.12em; }
.separator { font-weight: 300; opacity: 0.5; color: var(--text-secondary); }
.subtitle { font-weight: 300; font-size: var(--text-sm); color: var(--text-secondary); letter-spacing: 0.04em; }
```

**Candidate for shared component:** `AppHeader` with `name` and `subtitle` props.

### 2.2 Up Button Pattern (2 apps)

Identical `.up-btn` styling in Svalinn and Kvasir:

| File | Lines |
|------|-------|
| `/Users/johnny/.ai/smidja/yggdrasil/svalinn/src/lib/SvalinnView.svelte` | CSS 513-532 |
| `/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/lib/KvasirView.svelte` | CSS 638-657 |

Both use `background: none; border: 1px solid var(--border-default); border-radius: var(--radius-sm); color: var(--text-secondary)` with identical hover/disabled states. This is essentially a `Button variant="ghost"` with a border, or could be a new variant.

### 2.3 Directory Input Pattern (2 apps)

Identical `.directory-input` styling in Svalinn and Kvasir:

| File | Lines |
|------|-------|
| `/Users/johnny/.ai/smidja/yggdrasil/svalinn/src/lib/SvalinnView.svelte` | CSS 579-587 |
| `/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/lib/KvasirView.svelte` | CSS 724-732 |

Both are `flex: 1` text inputs with token-based styling. The shared `Input` component exists but is not used.

### 2.4 Empty State Pattern (3 apps)

Similar `.empty-state` styling in Svalinn, Kvasir, and Ratatoskr:

| File | Lines |
|------|-------|
| `/Users/johnny/.ai/smidja/yggdrasil/svalinn/src/lib/SvalinnView.svelte` | CSS 772-776 |
| `/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/lib/KvasirView.svelte` | CSS 895-899 |
| `/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/lib/JsonlViewer.svelte` | CSS 357-361 |
| `/Users/johnny/.ai/smidja/yggdrasil/ratatoskr/src/lib/RatatoskrView.svelte` | CSS 565-571 |

All have `text-align: center; padding: 4rem; color: var(--text-secondary)`. **Candidate for shared component:** `EmptyState` with a `message` prop.

### 2.5 Error Banner Pattern (2 apps)

Identical `.error-banner` in Kvasir and Ratatoskr:

| File | Lines |
|------|-------|
| `/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/lib/KvasirView.svelte` | CSS 734-740 |
| `/Users/johnny/.ai/smidja/yggdrasil/ratatoskr/src/lib/RatatoskrView.svelte` | CSS 458-464 |

Both are `background: var(--severity-error); color: var(--text-primary); padding; border-radius`. **Candidate for shared component:** `ErrorBanner`.

### 2.6 Code Viewer Pattern (2 components in same app)

The `.code-viewer` CSS block is duplicated between KvasirView and JsonlViewer:

| File | Lines |
|------|-------|
| `/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/lib/KvasirView.svelte` | CSS 845-888 |
| `/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/lib/JsonlViewer.svelte` | CSS 313-355 |

Both share identical `.code-viewer`, `.code-viewer pre`, `.code-viewer code`, `.line-number`, `.line-content`, `.wrap79`, `.wrapwidth` rules. The HTML template for rendering highlighted code is also nearly identical (lines 621 in KvasirView, 208 in JsonlViewer). **Candidate for shared component:** `CodeBlock` with `content`, `highlighted`, `wrapMode` props.

### 2.7 Format Selector Pattern (2 components in same app)

The `.format-selector` + `.format-btn` CSS is duplicated between FormatControls and JsonlViewer:

| File | Lines |
|------|-------|
| `/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/lib/FormatControls.svelte` | CSS 73-99 |
| `/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/lib/JsonlViewer.svelte` | CSS 289-311 |

Both have the same flex layout, button styling, active state, and hover behavior. FormatControls additionally supports a `.source` border.

### 2.8 Hlidskjalf Inline Stat Cards

HlidskjalfView builds its own stat display in the header (lines 310-322) with `.stat`, `.stat-value`, `.stat-label` (CSS lines 525-546). These CSS classes overlap directly with the shared `StatCard` component's responsibility, but the HlidskjalfView version is inline in a header bar rather than standalone cards.

### 2.9 Filter Button Pattern

HlidskjalfView has `.filter-btn` (lines 560-580), `.voice-btn` (595-614), and `.clear-btn` (616-631) which share nearly identical styling: `font-family: var(--font-mono); font-size: var(--text-xs); padding: 2px 10px; border: 1px solid var(--bg-tertiary); border-radius: var(--radius-full); background: transparent; color: var(--text-secondary); cursor: pointer;` with `.active` and `:hover` states. This is a toggle-pill pattern not covered by any shared component.

---

## 3. Missing Shared Components

Patterns that appear in 2+ apps but have no shared component:

| Pattern | Apps | Proposal |
|---------|------|----------|
| **App header** (name :: subtitle) | Hlidskjalf, Svalinn, Kvasir, Ratatoskr | `AppHeader` component with `name`, `subtitle` props |
| **Empty state** (centered message) | Svalinn, Kvasir (x2), Ratatoskr | `EmptyState` with `icon?`, `message` props |
| **Error banner** | Kvasir, Ratatoskr | `ErrorBanner` with `message` prop |
| **Code viewer** (line numbers + syntax highlight + wrap modes) | KvasirView, JsonlViewer | `CodeBlock` with `content`, `highlighted`, `wrapMode` props |
| **Toggle pill** (filter/kind buttons) | Hlidskjalf (filter-btn, voice-btn, clear-btn), Svalinn (view-btn) | `TogglePill` with `label`, `active`, `onclick` props |
| **Format selector** (JSON/YAML/TOML/TOON/RON tabs) | FormatControls, JsonlViewer | Already in FormatControls but not shared at `@yggdrasil/ui` level |
| **Controls panel** (bg-secondary, padded, rounded) | Svalinn (.controls), Kvasir (.controls), Ratatoskr (.controls) | Already covered by `Panel` but Panel is unused |
| **Directory row** (input + buttons) | Svalinn, Kvasir | Could be a `DirectoryPicker` or just use shared `Input` |

---

## 4. Import Graph

### 4.1 CSS Imports (all via `+layout.svelte`)

```
All 5 apps:
  +layout.svelte
    -> @yggdrasil/ui/css/tokens.css
    -> @yggdrasil/ui/css/base.css
```

### 4.2 Component Imports from `@yggdrasil/ui`

```
SvalinnView.svelte
  -> { Button, SidebarLayout, TreeNode, StatCard, SearchInput, FilterBanner }

KvasirView.svelte
  -> { Button, SidebarLayout, TreeNode }

JsonlViewer.svelte (Kvasir child)
  -> { Button }

RatatoskrView.svelte
  -> { Button }

Yggdrasil +page.svelte
  -> { ThemeSwitcher }

HlidskjalfView.svelte
  -> (no shared component imports)
```

### 4.3 Relative (`./`) Imports in View Components

```
HlidskjalfView.svelte
  -> ./QualityReport.svelte
  -> ./TrafficReport.svelte

KvasirView.svelte
  -> ./MarkdownPreview.svelte
  -> ./SchemaInspector.svelte
  -> ./JsonlViewer.svelte
  -> ./TableViewer.svelte
  -> ./FormatControls.svelte
  -> ./schema-inspect (TS)
  -> ./kvasir-types (TS)

SchemaInspector.svelte
  -> ./schema-inspect (TS types)

JsonlViewer.svelte
  -> ./kvasir-types (TS types)

FormatControls.svelte
  -> ./kvasir-types (TS types)

TableViewer.svelte
  -> ./kvasir-types (TS types)

SvalinnView.svelte
  -> (no relative imports)

RatatoskrView.svelte
  -> (no relative imports)
```

### 4.4 Internal UI Library Imports

```
TreeNode.svelte -> ./Badge.svelte
Collapsible.svelte -> ./Badge.svelte
```

### 4.5 Vite Alias Imports (Yggdrasil only)

```
yggdrasil/src/routes/+page.svelte
  -> $hlidskjalf/HlidskjalfView.svelte
  -> $svalinn/SvalinnView.svelte
  -> $kvasir/KvasirView.svelte
  -> $ratatoskr/RatatoskrView.svelte
```

### 4.6 Route Page to View Component Mapping

| App | Route Page | View Import | Import Path |
|-----|------------|-------------|-------------|
| Hlidskjalf | `+page.svelte` | HlidskjalfView | `$lib/HlidskjalfView.svelte` |
| Svalinn | `+page.svelte` | SvalinnView | `$lib/SvalinnView.svelte` |
| Kvasir | `+page.svelte` | KvasirView | `$lib/KvasirView.svelte` |
| Ratatoskr | `+page.svelte` | RatatoskrView | `$lib/RatatoskrView.svelte` |
| Yggdrasil | `+page.svelte` | All 4 views | `$hlidskjalf/`, `$svalinn/`, `$kvasir/`, `$ratatoskr/` |

Note: Standalone `+page.svelte` files correctly use `$lib/` (which resolves to their own `src/lib/`). This is correct because these are route-level files, not view components. The `$lib/` restriction only applies to files within `src/lib/` that may be consumed via Vite aliases.

---

## 5. Hardcoded Values Audit

### 5.1 Hardcoded Colors

| File | Line | Value | Should Be |
|------|------|-------|-----------|
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 99-104 | `hsl(210, 70%, 55%)` etc. (6 workspace hues) | Acceptable -- dynamic workspace color assignment, not a themeable property. However, these don't adapt to light/warm/cool themes. |

No other hardcoded colors found in app CSS blocks. All use `var(--*)` tokens.

### 5.2 Hardcoded Font Sizes (not using `--text-*` tokens)

| File | Lines | Values |
|------|-------|--------|
| `kvasir/src/lib/SchemaInspector.svelte` | 310, 323, 331, 339, 358, 371, 377, 385, 398, 401, 413, 428, 446 | `0.78rem`, `0.8rem`, `0.82rem`, `0.9rem`, `1rem`, `0.7rem` |
| `hlidskjalf/src/lib/TrafficReport.svelte` | 315, 387, 451 | `9px`, `8px` |
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 651 | `48px` (empty icon) |
| `kvasir/src/lib/MarkdownPreview.svelte` | 53 | `0.9em` (inline code relative) |
| `yggdrasil/src/routes/+page.svelte` | 183, 224 | `0.75rem` (tab buttons) |
| `ui/components/ThemeSwitcher.svelte` | 72 | `0.65rem` |
| `ui/components/SidebarLayout.svelte` | 129 | `1.5rem` (close button) |

The SchemaInspector is the worst offender with 13 raw rem values. These should map to the existing `--text-xs` (0.75rem), `--text-sm` (0.875rem), `--text-base` (1rem) tokens, or the scale should be extended.

### 5.3 Hardcoded Spacing (pixel values instead of `--space-*` tokens)

High-frequency pattern in Hlidskjalf components -- many `1px 6px`, `2px 10px`, `2px 8px` values instead of `--space-xs`/`--space-sm`/`--space-md` tokens.

| File | Count | Common values |
|------|-------|---------------|
| `HlidskjalfView.svelte` | 10+ instances | `2px 8px`, `2px 10px`, `1px 6px`, `4px`, `48px` |
| `TrafficReport.svelte` | 6+ instances | `2px 10px`, `1px 6px`, `3px` |
| `QualityReport.svelte` | 5+ instances | `2px 10px`, `1px 6px`, `2px 0` |
| `SchemaInspector.svelte` | 3+ instances | `1px 0`, `4px` |
| `Yggdrasil +page.svelte` | 1 instance | `2px` |

### 5.4 Duplicate Font Stack in JS

`KvasirView.svelte` line 95 defines `mono: "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, monospace"` which duplicates the value of `--font-mono` from tokens.css. If the token value changes, this constant will silently diverge. The `sans` and `serif` stacks are custom alternatives for the font-family cycling feature, so those are intentional.

---

## 6. Recommendations

### 6.1 High Priority: Reduce Duplication

1. **Create `AppHeader` shared component.** Four apps duplicate the exact same header structure and CSS. Component should accept `name` and `subtitle` props.

2. **Create `EmptyState` shared component.** Three apps duplicate centered placeholder messages. Component should accept `icon?` and `message` props.

3. **Create `ErrorBanner` shared component.** Two apps duplicate error banner styling. Component should accept `message` prop, maybe `onDismiss?`.

4. **Extract `CodeBlock` from KvasirView/JsonlViewer.** The code viewer with line numbers, syntax highlighting, and wrap modes is duplicated within Kvasir's own components. Extract to a Kvasir-local component first (since no other app uses it), then evaluate promotion to shared library if Hlidskjalf or Ratatoskr later need code display.

### 6.2 Medium Priority: Fix Token Usage

5. **SchemaInspector: replace raw rem values with `--text-*` tokens.** Add `--text-2xs` (0.65rem) and `--text-xxs` (0.78rem) to tokens if the existing scale is insufficient, or round to nearest existing token.

6. **Hlidskjalf components: replace `px` spacing with `--space-*` tokens.** The `2px 10px` pattern appears 6+ times and maps roughly to `var(--space-xs) var(--space-lg)`. The `1px 6px` pattern maps roughly to `1px var(--space-sm)`.

7. **KvasirView: read `--font-mono` from CSS variable** instead of duplicating the font stack in the `FONT_CSS` constant. Use `getComputedStyle(document.documentElement).getPropertyValue('--font-mono')`.

### 6.3 Low Priority: Clean Up Unused Components

8. **Audit whether `Input`, `Select`, `Panel`, `Collapsible`, `ListItem` should be kept, updated, or removed.** They have zero consumers. Options:
   - **Adopt them:** Refactor apps to use `Input` instead of raw `<input>`, `Select` instead of raw `<select>`, etc. This would eliminate the `.directory-input` and bare `<select>` patterns.
   - **Remove them:** If they don't match the actual UI patterns apps need, remove them to avoid confusion.
   - **Panel** is close to what several apps build as `.controls` sections but lacks flexibility (no flexible padding, no flex layout). If adapted, it could replace duplicated panel patterns.
   - **Collapsible** is close to what SvalinnView builds as `<details class="group">` but has different visual treatment. Could be reconciled.

9. **Consider promoting `ThemeSwitcher` to standalone app layouts.** Currently only Yggdrasil exposes theme switching. Standalone apps use the default (darkly) theme with no way for users to switch.

### 6.4 Observations (No Action Required)

10. **Workspace hue colors in HlidskjalfView** (lines 98-105) are hardcoded HSL values. These serve a dynamic color-assignment purpose and are not theme-aware. If theme awareness is desired, they could be moved to tokens.css as `--workspace-hue-*` variables with theme-specific overrides.

11. **Badge severity type mismatch.** TreeNode's `badgeSeverity` prop accepts `"info"` but delegates to Badge which only handles `"blocked"`, `"error"`, `"warning"`, `"success"`, `"neutral"`. The `"info"` value falls through to the default, which coincidentally produces acceptable results (`var(--action-primary)`), but the type contract is inaccurate.

12. **Hlidskjalf is the only view with zero shared component imports.** This is architecturally acceptable since it has a fundamentally different layout (full-screen event feed with fixed header, no sidebar) compared to the sidebar-based views. However, it internally duplicates Badge-like, StatCard-like, and Collapsible-like patterns.

---

## 7. Component Count Summary

| Category | Count |
|----------|-------|
| Shared UI components (ui/components/) | 13 |
| Shared UI components actually imported by apps | 7 |
| Shared UI components with zero consumers | 6 |
| App-specific components (across all apps) | 8 |
| Route pages (+page.svelte) | 5 |
| Layout files (+layout.svelte) | 5 |
| Total .svelte files audited | 31 |
| Duplicated UI patterns across 2+ apps | 7 |
| Hardcoded font-size values | ~20 |
| Hardcoded spacing values (px) | ~25 |
| Hardcoded color values | 6 (workspace hues only, no CSS hex) |

---

## 8. Full File Inventory

### Shared UI Library (`/Users/johnny/.ai/smidja/yggdrasil/ui/`)

| File | Type | Size |
|------|------|------|
| `ui/package.json` | Config | Package definition |
| `ui/index.js` | Entry | Exports all 13 components |
| `ui/css/tokens.css` | CSS | Design tokens (4 themes, 238 lines) |
| `ui/css/base.css` | CSS | Reset + global styles (69 lines) |
| `ui/components/Button.svelte` | Component | 69 lines |
| `ui/components/Badge.svelte` | Component | 36 lines |
| `ui/components/Input.svelte` | Component | 47 lines |
| `ui/components/Select.svelte` | Component | 41 lines |
| `ui/components/Panel.svelte` | Component | 39 lines |
| `ui/components/StatCard.svelte` | Component | 45 lines |
| `ui/components/TreeNode.svelte` | Component | 132 lines |
| `ui/components/Collapsible.svelte` | Component | 57 lines |
| `ui/components/ListItem.svelte` | Component | 47 lines |
| `ui/components/SearchInput.svelte` | Component | 46 lines |
| `ui/components/SidebarLayout.svelte` | Component | 171 lines |
| `ui/components/FilterBanner.svelte` | Component | 42 lines |
| `ui/components/ThemeSwitcher.svelte` | Component | 91 lines |

### Hlidskjalf (`/Users/johnny/.ai/smidja/yggdrasil/hlidskjalf/src/`)

| File | Shared UI Imports | Relative Imports |
|------|-------------------|------------------|
| `routes/+layout.svelte` | tokens.css, base.css | -- |
| `routes/+page.svelte` | -- | `$lib/HlidskjalfView` |
| `lib/HlidskjalfView.svelte` | **(none)** | `./QualityReport`, `./TrafficReport` |
| `lib/QualityReport.svelte` | -- | -- |
| `lib/TrafficReport.svelte` | -- | -- |

### Svalinn (`/Users/johnny/.ai/smidja/yggdrasil/svalinn/src/`)

| File | Shared UI Imports | Relative Imports |
|------|-------------------|------------------|
| `routes/+layout.svelte` | tokens.css, base.css | -- |
| `routes/+page.svelte` | -- | `$lib/SvalinnView` |
| `lib/SvalinnView.svelte` | Button, SidebarLayout, TreeNode, StatCard, SearchInput, FilterBanner | -- |

### Kvasir (`/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/`)

| File | Shared UI Imports | Relative Imports |
|------|-------------------|------------------|
| `routes/+layout.svelte` | tokens.css, base.css | -- |
| `routes/+page.svelte` | -- | `$lib/KvasirView` |
| `lib/KvasirView.svelte` | Button, SidebarLayout, TreeNode | `./MarkdownPreview`, `./SchemaInspector`, `./JsonlViewer`, `./TableViewer`, `./FormatControls`, `./schema-inspect`, `./kvasir-types` |
| `lib/MarkdownPreview.svelte` | -- | -- |
| `lib/SchemaInspector.svelte` | -- | `./schema-inspect` (types) |
| `lib/JsonlViewer.svelte` | Button | `./kvasir-types` (types) |
| `lib/TableViewer.svelte` | -- | `./kvasir-types` (types) |
| `lib/FormatControls.svelte` | -- | `./kvasir-types` (types) |

### Ratatoskr (`/Users/johnny/.ai/smidja/yggdrasil/ratatoskr/src/`)

| File | Shared UI Imports | Relative Imports |
|------|-------------------|------------------|
| `routes/+layout.svelte` | tokens.css, base.css | -- |
| `routes/+page.svelte` | -- | `$lib/RatatoskrView` |
| `lib/RatatoskrView.svelte` | Button | -- |

### Yggdrasil (`/Users/johnny/.ai/smidja/yggdrasil/yggdrasil/src/`)

| File | Shared UI Imports | Alias Imports |
|------|-------------------|---------------|
| `routes/+layout.svelte` | tokens.css, base.css | -- |
| `routes/+page.svelte` | ThemeSwitcher | `$hlidskjalf/HlidskjalfView`, `$svalinn/SvalinnView`, `$kvasir/KvasirView`, `$ratatoskr/RatatoskrView` |
