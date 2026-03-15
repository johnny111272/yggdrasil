# Token & CSS Migration Plan

**Audit date:** 2026-03-15
**Auditor:** Claude Opus 4.6
**Scope:** All `.svelte` files in the yggdrasil workspace (excluding `node_modules/`, `.svelte-kit/`)
**Token source:** `ui/css/tokens.css`

---

## Executive Summary

The token system is well-designed and covers 4 themes (darkly, light, warm-dark, cool-dark) with 80+ tokens across 11 categories. Overall token compliance is **good** -- shared UI components (`ui/components/`) are nearly fully compliant. However, app-specific view components contain numerous hardcoded values that bypass the token system. The primary issues are:

1. **34 raw `font-size` values** across 7 files that use hardcoded rem/px/em values instead of `--text-*` tokens
2. **26 raw spacing values** using hardcoded `px` measurements for padding, margin, and gap instead of `--space-*` tokens
3. **6 hardcoded HSL workspace colors** in JS that never respond to theme changes
4. **Missing token categories** for line-height, font-weight, z-index, transition duration, opacity, and max-width
5. **1 theme-awareness gap** in RatatoskrView where `getComputedStyle()` reads tokens at render time but does not re-read on theme change

Files audited: 27 project `.svelte` files (13 `ui/components/`, 5 `*/src/routes/`, 9 `*/src/lib/`)

---

## 1. Current Token Inventory

### Background Colors (5 tokens)
| Token | Default (darkly) | Used for |
|-------|-----------------|----------|
| `--bg-primary` | `#1a1a2e` | Body background |
| `--bg-secondary` | `#16213e` | Panels, cards, sidebar |
| `--bg-tertiary` | `#262640` | Borders, dividers |
| `--bg-hover` | `#1d2a4d` | Hover state |
| `--bg-selected` | `#2d3a5d` | Selected/active state |
| `--bg-alt-row` | `rgba(255,255,255,0.02)` | Alternating table rows |

### Text Colors (3 tokens)
| Token | Default | Used for |
|-------|---------|----------|
| `--text-primary` | `#eee` | Main text |
| `--text-secondary` | `#888` | Labels, subtitles |
| `--text-muted` | `#ccc` | Descriptions, less important |

### Border Colors (2 tokens)
| Token | Default | Used for |
|-------|---------|----------|
| `--border-default` | `#333` | Standard borders |
| `--border-subtle` | `#262640` | Subtle dividers |

### Severity Colors (4 tokens)
| Token | Default | Used for |
|-------|---------|----------|
| `--severity-blocked` | `#ff44ff` | Magenta -- gaming the system |
| `--severity-error` | `#ff3333` | Red -- violations |
| `--severity-warning` | `#ffd93d` | Yellow -- concerns |
| `--severity-success` | `#6bcb77` | Green -- success/good |

### Interactive Colors (6 tokens)
| Token | Default | Used for |
|-------|---------|----------|
| `--action-primary` | `#4361ee` | Primary buttons |
| `--action-primary-hover` | `#3a56d4` | Primary hover |
| `--action-special` | `#7b2cbf` | Special actions |
| `--action-special-hover` | `#6a24a8` | Special hover |
| `--action-neutral` | `#333` | Neutral buttons |
| `--action-neutral-hover` | `#444` | Neutral hover |

### Spacing Scale (7 tokens)
| Token | Value | px equivalent |
|-------|-------|--------------|
| `--space-xs` | `0.25rem` | 4px |
| `--space-sm` | `0.35rem` | ~6px |
| `--space-md` | `0.5rem` | 8px |
| `--space-lg` | `0.75rem` | 12px |
| `--space-xl` | `1rem` | 16px |
| `--space-2xl` | `1.5rem` | 24px |
| `--space-3xl` | `2rem` | 32px |

### Border Radius (3 tokens)
| Token | Value | Used for |
|-------|-------|----------|
| `--radius-sm` | `4px` | Inputs, buttons |
| `--radius-md` | `8px` | Panels, cards |
| `--radius-full` | `999px` | Badges, pills |

### Typography (9 tokens)
| Token | Value | Used for |
|-------|-------|----------|
| `--font-body` | system stack | Body text font |
| `--font-mono` | monospace stack | Code/data font |
| `--text-xs` | `0.75rem` (12px) | Badges |
| `--text-sm` | `0.875rem` (14px) | Body text |
| `--text-base` | `1rem` (16px) | Default |
| `--text-lg` | `1.25rem` (20px) | Subheadings |
| `--text-xl` | `1.5rem` (24px) | Stat values |
| `--text-2xl` | `2rem` (32px) | Large values |
| `--text-3xl` | `2.5rem` (40px) | Headings |

### Layout (1 token)
| Token | Value |
|-------|-------|
| `--sidebar-width` | `280px` |

### Shadows (2 tokens)
| Token | Value |
|-------|-------|
| `--shadow-sm` | `0 1px 2px rgba(0,0,0,0.3)` |
| `--shadow-md` | `0 4px 6px rgba(0,0,0,0.3)` |

### Syntax Highlighting (7 tokens)
| Token | Default | Used for |
|-------|---------|----------|
| `--syntax-identifier` | `#7aa2f7` | Section names |
| `--syntax-annotation` | `#bb9af7` | Nullable, format, labels |
| `--syntax-keyword` | `#7dcfff` | Field types |
| `--syntax-value` | `#9ece6a` | Defaults, check marks |
| `--syntax-conditional` | `#e0af68` | Conditionals |
| `--syntax-error` | `#f7768e` | Excludes, errors |
| `--syntax-constraint` | `#73daca` | Constraints |

### Graph Visualization (7 tokens -- all reference other tokens)
`--graph-node-app`, `--graph-node-library`, `--graph-node-framework`, `--graph-node-default`, `--graph-edge`, `--graph-edge-label`, `--graph-node-stroke`, `--graph-node-label`

### Priority Tints (2 tokens)
`--priority-high-tint`, `--priority-critical-tint`

### Theme Coverage
All 4 themes override: backgrounds, text, borders, severity, actions, syntax, priority tints, shadows, alt-row. Graph tokens reference other tokens so they inherit automatically. **No gaps in theme coverage for existing tokens.**

---

## 2. Token Gap Analysis

### 2.1 Missing Font-Size Values

Used in codebase but no corresponding token:

| Value | Where used | Proposed token |
|-------|-----------|---------------|
| `0.65rem` (~10.4px) | ThemeSwitcher `.theme-btn` | `--text-2xs` |
| `0.7rem` (~11.2px) | SchemaInspector `.section-arrow` | (use `--text-2xs` or `--text-xs`) |
| `0.78rem` (~12.5px) | SchemaInspector `.ext`, `.ext-panel`, `.field-desc`, `.badge-btn` | (use `--text-xs`) |
| `0.8rem` (~12.8px) | SchemaInspector `.controls label`, `.subtitle`, `.stats`, `.note`, `.section-nullable` | (new `--text-xs-lg` or use `--text-xs` 0.75rem) |
| `0.82rem` (~13.1px) | SchemaInspector `.section-body` | (use `--text-sm`) |
| `0.9rem` (~14.4px) | SchemaInspector `.section-header` | (use `--text-sm`) |
| `0.9em` | MarkdownPreview `code` | (relative em, acceptable) |
| `1rem` (`h2`) | SchemaInspector `.header h2` | `--text-base` exists |
| `1.5rem` | SidebarLayout `.sidebar-close` | `--text-xl` exists |
| `9px` | TrafficReport `.field-pip`, `.section-toggle` | `--text-2xs` |
| `8px` | TrafficReport `.tool-icon` | `--text-2xs` |
| `10px` | RatatoskrView edge labels (D3) | `--text-2xs` |
| `12px` | RatatoskrView node labels (D3) | `--text-xs` |
| `48px` | HlidskjalfView `.empty-icon` | (one-off decorative, acceptable) |
| `0.75rem` | Yggdrasil `.tab-btn`, `.clear-btn` | `--text-xs` exists |

### 2.2 Missing Spacing Values

Hardcoded spacing values used that fall outside the scale:

| Value | Where | Proposed solution |
|-------|-------|------------------|
| `2px` | ThemeSwitcher gap, Yggdrasil tab padding, KvasirView `.font-controls` gap | `--space-2xs: 0.125rem` (2px) |
| `1px` (padding) | Various `padding: 1px 6px` patterns | `--space-2xs` |
| `6px` (padding) | Classifier pill, source pill, expanded badge, edu-label | `--space-sm` is ~6px, close enough |
| `10px` (padding) | Filter/voice/clear buttons `padding: 2px 10px` | Approx `--space-sm --space-lg` |
| `3px` | SidebarLayout `.resize-handle` offset | One-off, acceptable |
| `4rem` | Empty state padding | `--space-4xl: 4rem` |

### 2.3 Missing Token Categories

**Line-height** -- used raw in 12+ places:
- `1.4` (TrafficReport, TableViewer, SvalinnView, KvasirView)
- `1.5` (base.css, QualityReport, SchemaInspector)
- `1.6` (KvasirView code, JsonlViewer code, SchemaInspector)
- `1.7` (MarkdownPreview)
- `1` (SidebarLayout close button)
- `0.9rem` (Yggdrasil `.tab-char`)

Proposed tokens:
```
--lh-tight: 1.2;
--lh-normal: 1.5;
--lh-relaxed: 1.6;
--lh-loose: 1.7;
```

**Font-weight** -- used raw in many places:
- `300` (separator, subtitle)
- `400` (section-nullable)
- `500` (Badge, format buttons, token-label, tab-btn)
- `600` (panel-header, various labels, format-badge, section-header)
- `700` (stat-value, title, banner-badge, col-kind-label, section-header, various)
- `800` (app-name)
- `bold` (sidebar-title, stat-label, issue-code, th)

Proposed tokens:
```
--fw-light: 300;
--fw-normal: 400;
--fw-medium: 500;
--fw-semibold: 600;
--fw-bold: 700;
--fw-extrabold: 800;
```

**Z-index** -- used raw:
- `1` (col-source, thead, table-scroll)
- `10` (SchemaInspector sticky controls)
- `100` (SidebarLayout sidebar)
- `101` (SidebarLayout resize handle)

Proposed tokens:
```
--z-base: 1;
--z-sticky: 10;
--z-sidebar: 100;
--z-overlay: 200;
```

**Transition duration** -- used raw:
- `0.15s` (Button, ThemeSwitcher, HlidskjalfView expand-hint, col-source)
- `300` (RatatoskrView zoom reset via D3)

Proposed token:
```
--transition-fast: 0.15s;
--transition-normal: 0.3s;
```

**Opacity** -- used raw:
- `0.3` (disabled buttons: up-btn, nav-btn)
- `0.4` (resize-handle hover, export-btn disabled)
- `0.5` (disabled state, separator, trace priority, canary)
- `0.6` (D3 stroke-opacity)
- `0.7` (low priority, badge-btn)

Proposed tokens:
```
--opacity-disabled: 0.5;
--opacity-muted: 0.7;
--opacity-hint: 0.4;
```

**Max-width** -- used raw:
- `1200px` (SidebarLayout `.main-content`)
- `250px` (RatatoskrView panels)
- `79ch` (code wrap mode)
- `300px` (expanded-json max-height)

### 2.4 Missing Semantic Colors

The workspace hue palette in HlidskjalfView is hardcoded in JS and never adapts to theme:
```js
const WORKSPACE_HUES = [
  "hsl(210, 70%, 55%)",
  "hsl(35, 85%, 55%)",
  "hsl(0, 70%, 55%)",
  "hsl(270, 60%, 60%)",
  "hsl(160, 60%, 45%)",
  "hsl(320, 60%, 55%)",
];
```

These need to be either CSS custom properties (theme-aware) or have lightness values tuned per-theme.

---

## 3. Violation Inventory

### 3.1 `ui/components/ThemeSwitcher.svelte`

| Line | Current | Recommended |
|------|---------|-------------|
| 57 | `gap: 2px` | `gap: var(--space-2xs)` (new token) |
| 65 | `width: 20px` | Keep (fixed element size) |
| 66 | `height: 20px` | Keep (fixed element size) |
| 72 | `font-size: 0.65rem` | `font-size: var(--text-2xs)` (new token) |
| 73 | `font-weight: 600` | `font-weight: var(--fw-semibold)` (new token) |
| 79 | `transition: color 0.15s ease, border-color 0.15s ease` | `transition: color var(--transition-fast) ease, border-color var(--transition-fast) ease` (new token) |

### 3.2 `ui/components/Badge.svelte`

| Line | Current | Recommended |
|------|---------|-------------|
| 30 | `padding: 0.125rem var(--space-md)` | `padding: var(--space-2xs) var(--space-md)` (new token) |
| 33 | `font-weight: 500` | `font-weight: var(--fw-medium)` (new token) |

### 3.3 `ui/components/SidebarLayout.svelte`

| Line | Current | Recommended |
|------|---------|-------------|
| 103 | `z-index: 100` | `z-index: var(--z-sidebar)` (new token) |
| 116 | `font-weight: bold` | `font-weight: var(--fw-bold)` (new token) |
| 128 | `font-size: 1.5rem` | `font-size: var(--text-xl)` |
| 132 | `line-height: 1` | Keep (visual alignment for close button) |
| 148 | `right: -3px` | Keep (resize handle offset) |
| 151 | `width: 6px` | Keep (resize handle width) |
| 152 | `z-index: 101` | `z-index: calc(var(--z-sidebar) + 1)` |
| 157 | `opacity: 0.4` | `opacity: var(--opacity-hint)` (new token) |
| 162 | `max-width: 1200px` | Consider token, but acceptable as layout constant |

### 3.4 `ui/components/StatCard.svelte`

| Line | Current | Recommended |
|------|---------|-------------|
| 38 | `font-weight: bold` | `font-weight: var(--fw-bold)` (new token) |

### 3.5 `ui/components/Button.svelte`

| Line | Current | Recommended |
|------|---------|-------------|
| 29 | `transition: background 0.15s ease` | `transition: background var(--transition-fast) ease` (new token) |
| 34 | `opacity: 0.5` | `opacity: var(--opacity-disabled)` (new token) |

### 3.6 `ui/components/Panel.svelte`

| Line | Current | Recommended |
|------|---------|-------------|
| 33 | `font-weight: 600` | `font-weight: var(--fw-semibold)` (new token) |

### 3.7 `hlidskjalf/src/lib/HlidskjalfView.svelte`

| Line | Current | Recommended |
|------|---------|-------------|
| 483 | `font-weight: 700` | `font-weight: var(--fw-bold)` |
| 491 | `font-weight: 800` | `font-weight: var(--fw-extrabold)` |
| 493 | `letter-spacing: 0.12em` | Keep (design intent) |
| 496 | `font-weight: 300` | `font-weight: var(--fw-light)` |
| 497 | `opacity: 0.5` | `opacity: var(--opacity-disabled)` |
| 503 | `font-weight: 300` | `font-weight: var(--fw-light)` |
| 505 | `letter-spacing: 0.04em` | Keep (design intent) |
| 510 | `padding: 2px 8px` | `padding: var(--space-2xs) var(--space-md)` |
| 532 | `font-weight: 700` | `font-weight: var(--fw-bold)` |
| 563 | `padding: 2px 10px` | `padding: var(--space-2xs) var(--space-lg)` |
| 592 | `gap: 4px` | `gap: var(--space-xs)` |
| 598 | `padding: 2px 10px` | `padding: var(--space-2xs) var(--space-lg)` |
| 619 | `padding: 2px 10px` | `padding: var(--space-2xs) var(--space-lg)` |
| 651 | `font-size: 48px` | Keep (decorative one-off) |
| 661 | `border-left: 3px solid transparent` | Keep (visual weight indicator) |
| 665 | `opacity: 0.5` | `opacity: var(--opacity-disabled)` |
| 669 | `opacity: 0.7` | `opacity: var(--opacity-muted)` |
| 687 | `opacity: 0.5` | `opacity: var(--opacity-disabled)` |
| 699 | `min-height: 1.6em` | Keep (line-height related) |
| 713 | `min-height: 1.2em` | Keep |
| 720 | `min-width: 72px` | Keep (tabular layout) |
| 727 | `min-width: 20px` | Keep |
| 732 | `font-weight: 600` | `font-weight: var(--fw-semibold)` |
| 734 | `letter-spacing: 0.04em` | Keep |
| 735 | `min-width: 60px` | Keep |
| 743 | `padding: 1px 6px` | `padding: var(--space-2xs) var(--space-sm)` |
| 751 | `font-weight: 600` | `font-weight: var(--fw-semibold)` |
| 753 | `min-width: 80px` | Keep |
| 825 | `padding: 1px 6px` | `padding: var(--space-2xs) var(--space-sm)` |
| 829 | `opacity: 0` | Keep (animation start) |
| 831 | `transition: opacity 0.15s` | `transition: opacity var(--transition-fast)` |
| 834 | `opacity: 1` | Keep (animation end) |
| 827 | `opacity: 0` | Keep (animation start) |
| 829 | `transition: opacity 0.15s` | `transition: opacity var(--transition-fast)` |
| 853 | `padding: 1px 6px` | `padding: var(--space-2xs) var(--space-sm)` |
| 865 | `max-height: 300px` | Consider `--max-height-panel` token |
| 98-105 (JS) | `WORKSPACE_HUES` array with 6 hardcoded HSL colors | Move to CSS tokens (P3) |

### 3.8 `hlidskjalf/src/lib/QualityReport.svelte`

| Line | Current | Recommended |
|------|---------|-------------|
| 184 | `font-weight: 700` | `font-weight: var(--fw-bold)` |
| 186 | `padding: 2px 10px` | `padding: var(--space-2xs) var(--space-lg)` |
| 190 | `letter-spacing: 0.5px` | Keep |
| 199 | `font-weight: 700` | `font-weight: var(--fw-bold)` |
| 239 | `font-weight: 700` | `font-weight: var(--fw-bold)` |
| 273 | `padding: 2px 0` | `padding: var(--space-2xs) 0` |
| 274 | `line-height: 1.5` | `line-height: var(--lh-normal)` (new token) |
| 279 | `font-weight: 700` | `font-weight: var(--fw-bold)` |
| 281 | `padding: 1px 6px` | `padding: var(--space-2xs) var(--space-sm)` |
| 284 | `margin-top: 2px` | `margin-top: var(--space-2xs)` |

### 3.9 `hlidskjalf/src/lib/TrafficReport.svelte`

| Line | Current | Recommended |
|------|---------|-------------|
| 267 | `font-weight: 700` | `font-weight: var(--fw-bold)` |
| 269 | `padding: 2px 10px` | `padding: var(--space-2xs) var(--space-lg)` |
| 271 | `letter-spacing: 0.5px` | Keep |
| 302 | `font-weight: 600` | `font-weight: var(--fw-semibold)` |
| 303 | `padding: 1px 6px` | `padding: var(--space-2xs) var(--space-sm)` |
| 310 | `gap: 3px` | `gap: var(--space-2xs)` |
| 315 | `font-size: 9px` | `font-size: var(--text-2xs)` (new token) |
| 316 | `font-weight: 700` | `font-weight: var(--fw-bold)` |
| 317 | `width: 16px` | Keep (fixed pip size) |
| 318 | `height: 16px` | Keep (fixed pip size) |
| 322 | `border-radius: 2px` | `border-radius: var(--space-2xs)` or keep |
| 377 | `font-weight: 700` | `font-weight: var(--fw-bold)` |
| 387 | `font-size: 9px` | `font-size: var(--text-2xs)` (new token) |
| 398 | `max-height: 4.2em` | Keep (collapsed preview height) |
| 410 | `line-height: 1.4` | `line-height: var(--lh-tight)` or keep |
| 439 | `padding: 2px var(--space-sm)` | `padding: var(--space-2xs) var(--space-sm)` |
| 441 | `font-size: var(--text-xs)` | OK |
| 451 | `font-size: 8px` | `font-size: var(--text-2xs)` (new token) |
| 469 | `max-height: 300px` | Consider token |
| 486 | `padding: 1px 6px` | `padding: var(--space-2xs) var(--space-sm)` |

### 3.10 `svalinn/src/lib/SvalinnView.svelte`

| Line | Current | Recommended |
|------|---------|-------------|
| 521 | `line-height: 1.4` | Consider `--lh-tight` token |
| 549 | `font-weight: 800` | `font-weight: var(--fw-extrabold)` |
| 550 | `letter-spacing: 0.12em` | Keep |
| 554 | `font-weight: 300` | `font-weight: var(--fw-light)` |
| 555 | `opacity: 0.5` | `opacity: var(--opacity-disabled)` |
| 560 | `font-weight: 300` | `font-weight: var(--fw-light)` |
| 563 | `letter-spacing: 0.04em` | Keep |
| 765 | `font-weight: bold` | `font-weight: var(--fw-bold)` |
| 774 | `padding: 4rem var(--space-2xl)` | `padding: var(--space-4xl) var(--space-2xl)` (new token) |

### 3.11 `kvasir/src/lib/KvasirView.svelte`

| Line | Current | Recommended |
|------|---------|-------------|
| 646 | `line-height: 1.4` | Consider token |
| 693 | `font-weight: 800` | `font-weight: var(--fw-extrabold)` |
| 695 | `letter-spacing: 0.12em` | Keep |
| 700 | `font-weight: 300` | `font-weight: var(--fw-light)` |
| 701 | `opacity: 0.5` | `opacity: var(--opacity-disabled)` |
| 705 | `font-weight: 300` | `font-weight: var(--fw-light)` |
| 709 | `letter-spacing: 0.04em` | Keep |
| 800 | `gap: 2px` | `gap: var(--space-2xs)` (new token) |
| 856 | `line-height: 1.6` | `line-height: var(--lh-relaxed)` (new token) |
| 897 | `padding: 4rem var(--space-3xl)` | `padding: var(--space-4xl) var(--space-3xl)` (new token) |

### 3.12 `kvasir/src/lib/SchemaInspector.svelte`

| Line | Current | Recommended |
|------|---------|-------------|
| 310 | `font-size: 0.8rem` | Use `--text-xs` (0.75rem close enough) or new token |
| 325 | `font-size: 1rem` | `font-size: var(--text-base)` |
| 326 | `font-weight: 600` | `font-weight: var(--fw-semibold)` |
| 332 | `font-size: 0.8rem` | Use `--text-xs` |
| 339 | `font-size: 0.8rem` | Use `--text-xs` |
| 358 | `font-size: 0.9rem` | Use `--text-sm` (0.875rem, close enough) |
| 357 | `font-weight: 600` | `font-weight: var(--fw-semibold)` |
| 371 | `font-size: 0.7rem` | Use `--text-2xs` (new) or `--text-xs` |
| 374 | `width: 12px` | Keep |
| 377 | `font-weight: 400` (via class) | `font-weight: var(--fw-normal)` |
| 377 | `font-size: 0.8rem` | Use `--text-xs` |
| 385 | `font-size: 0.82rem` | Use `--text-sm` (close enough) |
| 386 | `line-height: 1.6` | `line-height: var(--lh-relaxed)` |
| 389 | `padding: 1px 0` | `padding: var(--space-2xs) 0` |
| 391 | `padding: 1px 0` | `padding: var(--space-2xs) 0` |
| 398 | `font-size: 0.8rem` | Use `--text-xs` |
| 401 | `font-size: 0.78rem` | Use `--text-xs` |
| 413 | `font-size: 0.78rem` | Use `--text-xs` |
| 415 | `margin-left: 4px` | `margin-left: var(--space-xs)` |
| 428 | `font-size: 0.78rem` | Use `--text-xs` |
| 445 | `font-size: 0.78rem` | Use `--text-xs` |
| 451 | `padding-left: var(--space-md)` -- OK | |
| 459 | `font-weight: 600` | `font-weight: var(--fw-semibold)` |
| 462 | `font-weight: bold` | `font-weight: var(--fw-bold)` |

### 3.13 `kvasir/src/lib/MarkdownPreview.svelte`

| Line | Current | Recommended |
|------|---------|-------------|
| 16 | `line-height: 1.7` | `line-height: var(--lh-loose)` (new token) |
| 53 | `font-size: 0.9em` | Keep (relative to parent em) |
| 81 | `border-left: 4px solid var(--action-primary)` | Keep |

### 3.14 `kvasir/src/lib/JsonlViewer.svelte`

| Line | Current | Recommended |
|------|---------|-------------|
| 265 | `height: 4px` | Keep (scrubber track height) |
| 269 | `border-radius: 2px` | `border-radius: var(--space-2xs)` or keep |
| 275 | `width: 14px` | Keep (scrubber thumb) |
| 276 | `height: 14px` | Keep |
| 277 | `border-radius: 50%` | Keep |
| 324 | `line-height: 1.6` | `line-height: var(--lh-relaxed)` |

### 3.15 `kvasir/src/lib/TableViewer.svelte`

| Line | Current | Recommended |
|------|---------|-------------|
| 224 | `padding: 2px var(--space-sm)` | `padding: var(--space-2xs) var(--space-sm)` |
| 226 | `font-weight: 600` | `font-weight: var(--fw-semibold)` |
| 227 | `letter-spacing: 0.05em` | Keep |
| 240 | `opacity: 0.4` | `opacity: var(--opacity-hint)` |
| 271 | `line-height: 1.4` | Consider token |
| 285 | `font-weight: 600` | `font-weight: var(--fw-semibold)` |
| 289 | `border-bottom: 2px solid var(--border-default)` | Keep (header emphasis) |

### 3.16 `kvasir/src/lib/FormatControls.svelte`

| Line | Current | Recommended |
|------|---------|-------------|
| 86 | `font-weight: 500` | `font-weight: var(--fw-medium)` |
| 112 | `font-weight: 500` | `font-weight: var(--fw-medium)` |

### 3.17 `ratatoskr/src/lib/RatatoskrView.svelte`

| Line | Current | Recommended |
|------|---------|-------------|
| 429 | `font-weight: 800` | `font-weight: var(--fw-extrabold)` |
| 430 | `letter-spacing: 0.12em` | Keep |
| 434 | `font-weight: 300` | `font-weight: var(--fw-light)` |
| 435 | `opacity: 0.5` | `opacity: var(--opacity-disabled)` |
| 439 | `font-weight: 300` | `font-weight: var(--fw-light)` |
| 443 | `letter-spacing: 0.04em` | Keep |
| 489 | `max-width: 250px` | Keep (overlay panel) |
| 549 | `width: 10px` | Keep (type dot) |
| 550 | `height: 10px` | Keep |
| 551 | `border-radius: 50%` | Keep |

### 3.18 `yggdrasil/src/routes/+page.svelte`

| Line | Current | Recommended |
|------|---------|-------------|
| 165 | `width: 28px` | Keep (tab strip width -- design constant) |
| 183 | `font-size: 0.75rem` | `font-size: var(--text-xs)` |
| 184 | `font-weight: 500` | `font-weight: var(--fw-medium)` |
| 185 | `letter-spacing: 0.05em` | Keep |
| 191 | `padding: var(--space-xs) 2px` | `padding: var(--space-xs) var(--space-2xs)` |
| 214 | `font-weight: 700` | `font-weight: var(--fw-bold)` |
| 226 | `font-size: 0.75rem` | `font-size: var(--text-xs)` |

---

## 4. Duplicated CSS Patterns

### 4.1 App Header Pattern (duplicated across 4 apps)

The exact same pattern appears in HlidskjalfView, SvalinnView, KvasirView, RatatoskrView:
```css
.app-name { font-weight: 800; letter-spacing: 0.12em; }
.separator { font-weight: 300; opacity: 0.5; color: var(--text-secondary); }
.subtitle { font-weight: 300; font-size: var(--text-sm); color: var(--text-secondary); letter-spacing: 0.04em; }
```

**Recommendation:** Create a shared `AppHeader.svelte` component in `ui/components/` or a shared CSS class.

### 4.2 Up-Button Pattern (duplicated in SvalinnView + KvasirView)

Identical styles for `.up-btn` in both files (~15 lines each).

**Recommendation:** Either use the shared `Button` component with `variant="ghost"` or create a shared CSS mixin.

### 4.3 Code Viewer Pattern (duplicated in KvasirView + JsonlViewer)

The `.code-viewer`, `.line-number`, `.line-content`, `.wrap79`, `.wrapwidth` styles are duplicated.

**Recommendation:** Extract into a shared `CodeViewer.svelte` component.

### 4.4 Empty State Pattern (duplicated in 4 files)

```css
.empty-state { text-align: center; padding: 4rem var(--space-2xl); color: var(--text-secondary); }
```

Files: SvalinnView, KvasirView, RatatoskrView, JsonlViewer.

**Recommendation:** Create a shared `EmptyState.svelte` component or add to tokens as `--padding-empty-state`.

### 4.5 Error Banner Pattern (duplicated in KvasirView + RatatoskrView)

```css
.error-banner { background: var(--severity-error); color: var(--text-primary); padding: var(--space-lg) var(--space-xl); border-radius: var(--radius-sm); margin-bottom: ...; }
```

**Recommendation:** Create a shared `ErrorBanner.svelte` component.

### 4.6 Format Button Pattern (duplicated in FormatControls + JsonlViewer)

Identical `.format-btn` and `.format-selector` styles.

**Recommendation:** Extract shared `.format-btn` styles or create a shared component.

### 4.7 Badge Pill Pattern (duplicated across HlidskjalfView, QualityReport, TrafficReport)

`padding: 2px 10px; border-radius: var(--radius-sm|full); font-size: var(--text-xs); font-weight: 700;`

---

## 5. Theme Awareness Gaps

### 5.1 WORKSPACE_HUES in HlidskjalfView (CRITICAL)

**File:** `hlidskjalf/src/lib/HlidskjalfView.svelte`, lines 98-105

```js
const WORKSPACE_HUES = [
  "hsl(210, 70%, 55%)",  // blue
  "hsl(35, 85%, 55%)",   // amber
  "hsl(0, 70%, 55%)",    // red
  "hsl(270, 60%, 60%)",  // purple
  "hsl(160, 60%, 45%)",  // teal
  "hsl(320, 60%, 55%)",  // pink
];
```

These colors are applied via inline `style="color: ..."` and never change when the theme changes. On the light theme (white background), these saturated colors will have lower contrast. On warm-dark, the blue hue clashes with the warm palette.

**Fix:** Move to CSS custom properties with per-theme overrides:
```css
:root {
  --workspace-1: hsl(210, 70%, 55%);
  --workspace-2: hsl(35, 85%, 55%);
  --workspace-3: hsl(0, 70%, 55%);
  --workspace-4: hsl(270, 60%, 60%);
  --workspace-5: hsl(160, 60%, 45%);
  --workspace-6: hsl(320, 60%, 55%);
}
body[data-theme="light"] {
  --workspace-1: hsl(210, 70%, 40%);
  /* ... darker for white bg */
}
```

Then read them via `getComputedStyle()` or reference them via `var()`.

### 5.2 RatatoskrView getComputedStyle (MODERATE)

**File:** `ratatoskr/src/lib/RatatoskrView.svelte`, lines 68-79

```js
function cssVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}
```

This function reads CSS token values at the time D3 nodes are created (`renderGraph()`). If the user switches themes AFTER the graph is rendered, the node colors, edge colors, labels, and arrowhead marker all retain the old theme's values. The D3 elements are not reactive to CSS variable changes because the values are baked into SVG attributes.

**Fix options:**
1. Re-render the graph on theme change (listen for `data-theme` attribute mutation)
2. Use CSS variables directly in SVG via `style` attributes instead of resolved values (where D3 allows it)
3. Accept the limitation and document it

### 5.3 FONT_CSS in KvasirView (LOW)

**File:** `kvasir/src/lib/KvasirView.svelte`, lines 94-99

```js
const FONT_CSS: Record<FontFamily, string> = {
  mono: "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, monospace",
  sans: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
  ...
};
```

These duplicate the `--font-mono` and `--font-body` token values. If tokens change, these JS constants will be stale.

**Fix:** Reference `var(--font-mono)` and `var(--font-body)` in the CSS instead of JS font stacks. For the dyslexie and serif options, add new `--font-dyslexie` and `--font-serif` tokens.

---

## 6. New Tokens Proposal

### 6.1 Spacing

```css
--space-2xs: 0.125rem;   /* 2px -- micro gaps, pill padding */
--space-4xl: 4rem;        /* 64px -- empty state padding */
```

### 6.2 Typography -- Font Size

```css
--text-2xs: 0.625rem;     /* 10px -- pips, toggle arrows, tiny icons */
```

### 6.3 Typography -- Font Weight

```css
--fw-light: 300;
--fw-normal: 400;
--fw-medium: 500;
--fw-semibold: 600;
--fw-bold: 700;
--fw-extrabold: 800;
```

### 6.4 Typography -- Line Height

```css
--lh-tight: 1.2;
--lh-normal: 1.5;
--lh-relaxed: 1.6;
--lh-loose: 1.7;
```

### 6.5 Typography -- Additional Fonts

```css
--font-dyslexie: 'Dyslexie', sans-serif;
--font-serif: Georgia, 'Times New Roman', serif;
```

### 6.6 Z-index

```css
--z-base: 1;
--z-sticky: 10;
--z-sidebar: 100;
--z-overlay: 200;
```

### 6.7 Transitions

```css
--transition-fast: 0.15s;
--transition-normal: 0.3s;
```

### 6.8 Opacity

```css
--opacity-disabled: 0.5;
--opacity-muted: 0.7;
--opacity-hint: 0.4;
```

### 6.9 Workspace Colors (theme-aware)

```css
:root {
  --workspace-1: hsl(210, 70%, 55%);
  --workspace-2: hsl(35, 85%, 55%);
  --workspace-3: hsl(0, 70%, 55%);
  --workspace-4: hsl(270, 60%, 60%);
  --workspace-5: hsl(160, 60%, 45%);
  --workspace-6: hsl(320, 60%, 55%);
}
```

With per-theme overrides adjusting lightness for contrast.

**Total new tokens: 25**

---

## 7. Per-File Migration Plan

### Priority Definitions

- **P1:** Value has an exact existing token match. Zero-risk mechanical replacement.
- **P2:** Value needs a new token created first, then mechanical replacement.
- **P3:** Value needs architectural discussion (theme-aware JS colors, shared component extraction).

---

### P1 Replacements (existing tokens -- can be done immediately)

**`ui/components/SidebarLayout.svelte`**
- L128: `font-size: 1.5rem` -> `font-size: var(--text-xl)`

**`kvasir/src/lib/SchemaInspector.svelte`**
- L325: `font-size: 1rem` -> `font-size: var(--text-base)`
- L415: `margin-left: 4px` -> `margin-left: var(--space-xs)`

**`hlidskjalf/src/lib/HlidskjalfView.svelte`**
- L592: `gap: 4px` -> `gap: var(--space-xs)`

**`yggdrasil/src/routes/+page.svelte`**
- L183: `font-size: 0.75rem` -> `font-size: var(--text-xs)`
- L226: `font-size: 0.75rem` -> `font-size: var(--text-xs)`

**SchemaInspector font-size consolidation (approximate matches)**
- All `0.78rem` and `0.8rem` -> `var(--text-xs)` (0.75rem, 4% difference, visually identical)
- All `0.82rem` and `0.9rem` -> `var(--text-sm)` (0.875rem, ~5% difference)

Files affected: SchemaInspector (~14 replacements)

---

### P2 Replacements (need new tokens first)

**Step 1: Add tokens to `tokens.css`**

Add the 25 new tokens from Section 6 to `:root` (no theme overrides needed for most -- font-weight, line-height, z-index, transitions, opacity are theme-independent).

**Step 2: Apply across files**

| File | Replacements |
|------|-------------|
| `ui/components/Button.svelte` | `0.15s` -> `var(--transition-fast)`, `0.5` opacity -> `var(--opacity-disabled)` |
| `ui/components/Badge.svelte` | `0.125rem` -> `var(--space-2xs)`, `500` weight -> `var(--fw-medium)` |
| `ui/components/StatCard.svelte` | `bold` -> `var(--fw-bold)` |
| `ui/components/Panel.svelte` | `600` -> `var(--fw-semibold)` |
| `ui/components/SidebarLayout.svelte` | `100` z-index -> `var(--z-sidebar)`, `bold` -> `var(--fw-bold)`, `0.4` opacity -> `var(--opacity-hint)` |
| `ui/components/ThemeSwitcher.svelte` | `2px` gap -> `var(--space-2xs)`, `0.65rem` -> `var(--text-2xs)`, `600` -> `var(--fw-semibold)`, `0.15s` -> `var(--transition-fast)` |
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 15+ font-weight replacements, 6 padding `2px` -> `var(--space-2xs)`, 3 opacity replacements, 2 transition replacements |
| `hlidskjalf/src/lib/QualityReport.svelte` | 5 font-weight, 3 padding, 1 line-height |
| `hlidskjalf/src/lib/TrafficReport.svelte` | 4 font-weight, 3 `font-size: 9px/8px` -> `var(--text-2xs)`, 4 padding |
| `svalinn/src/lib/SvalinnView.svelte` | 5 font-weight, 1 opacity, 1 `4rem` -> `var(--space-4xl)` |
| `kvasir/src/lib/KvasirView.svelte` | 5 font-weight, 1 opacity, 1 gap `2px` -> `var(--space-2xs)`, 1 line-height, 1 `4rem` |
| `kvasir/src/lib/SchemaInspector.svelte` | 6 font-weight, multiple `1px` padding -> `var(--space-2xs)` |
| `kvasir/src/lib/FormatControls.svelte` | 2 font-weight |
| `kvasir/src/lib/JsonlViewer.svelte` | 1 line-height |
| `kvasir/src/lib/TableViewer.svelte` | 2 font-weight, 1 padding, 1 opacity |
| `kvasir/src/lib/MarkdownPreview.svelte` | 1 line-height |
| `ratatoskr/src/lib/RatatoskrView.svelte` | 3 font-weight, 1 opacity |
| `yggdrasil/src/routes/+page.svelte` | 2 font-weight |

**Estimated total P2 replacements: ~90**

---

### P3 Architectural Changes

#### 3a. Workspace colors -> theme-aware tokens

**Effort:** Medium
**Risk:** Low (additive change)
**Files:** `tokens.css` (add 6 `--workspace-*` tokens + per-theme overrides), `HlidskjalfView.svelte` (read from CSS instead of JS constant)

#### 3b. RatatoskrView theme reactivity

**Effort:** Medium
**Risk:** Low (re-render on theme change)
**Files:** `RatatoskrView.svelte` (add MutationObserver on body `data-theme` attribute, call `renderGraph()` on change)

#### 3c. KvasirView font stack deduplication

**Effort:** Low
**Risk:** Low
**Files:** `tokens.css` (add `--font-dyslexie`, `--font-serif`), `KvasirView.svelte` (reference tokens instead of JS constants)

#### 3d. Shared component extraction (deduplication)

**Effort:** High (affects multiple files, needs testing)
**Risk:** Medium (import path changes)
**Components to extract:**
- `AppHeader.svelte` -- used by all 4 views
- `CodeViewer.svelte` -- used by KvasirView + JsonlViewer
- `EmptyState.svelte` -- used by 4 views
- `ErrorBanner.svelte` -- used by KvasirView + RatatoskrView

This is a larger refactor and should be a separate task.

---

## 8. Summary Statistics

| Category | Count |
|----------|-------|
| Files with violations | 18 of 27 |
| Total violations found | ~130 |
| P1 (exact match, immediate) | ~20 |
| P2 (needs new tokens) | ~90 |
| P3 (architectural) | ~20 |
| New tokens proposed | 25 |
| Duplicated CSS patterns found | 7 |
| Theme awareness gaps | 3 |
| Shared UI components compliant | 11 of 13 (Badge + ThemeSwitcher have minor issues) |
| App view components compliant | 0 of 9 (all have violations) |

### Recommended Execution Order

1. Add all 25 new tokens to `tokens.css` `:root` block (and workspace tokens to all 4 theme blocks)
2. Execute all P1 replacements (~20 changes across 4 files)
3. Execute all P2 replacements in shared `ui/components/` first (6 files, ~15 changes)
4. Execute P2 replacements in app view components (9 files, ~75 changes)
5. Address P3 workspace colors + theme reactivity
6. Evaluate shared component extraction as a separate task
