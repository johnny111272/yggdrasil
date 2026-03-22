# SHARED_UI_DESIGN — Layout Architecture Audit

**Status:** Active problem. Recurring layout breaks on every deploy.
**Last updated:** 2026-03-15

---

## The Problem

Every CSS fix for clipping/overflow creates a new break elsewhere. The root cause is not individual CSS properties — it is that layout responsibilities are split between shared components and per-app views with no contract defining who owns what.

---

## Current State: Who Owns Layout?

### Shared UI (`ui/`)

| Component | Sets Height | Sets Overflow | Sets Position | Notes |
|-----------|-------------|---------------|---------------|-------|
| SidebarLayout | `100vh` on `<main>` | `overflow-y: auto` on sidebar + main-content | `position: fixed` on sidebar | **Two architectural assumptions that break when embedded** |
| Panel | none | `overflow: auto` | none | Never used by any app |
| tokens.css | none | none | none | Pure design tokens, no layout |
| base.css | none | none | none | Minimal reset, no layout |

### Per-App Views

| App | Uses SidebarLayout? | Defines Own Layout? | Height Source |
|-----|---------------------|---------------------|---------------|
| Hlidskjalf | No | `.watchtower { height: 100vh; display: flex; flex-direction: column }` | Own CSS |
| Svalinn | Yes | Delegates to SidebarLayout | SidebarLayout |
| Kvasir | Yes | Delegates to SidebarLayout, plus `.viewer-settings`, `.code-viewer`, `.inspector-view` overflow rules | SidebarLayout + own CSS |
| Ratatoskr | No | `main { height: 100vh; display: flex }` | Own CSS |

### Yggdrasil Shell

```
.shell { height: 100vh; display: flex }
  .view-area { flex: 1; overflow: hidden; position: relative }
    .view-pane { position: absolute; inset: 0; overflow: auto }
      [App View Component]
        SidebarLayout <main> { height: 100vh }  <-- CONFLICT
```

---

## Two Architectural Bugs

### 1. `height: 100vh` in SidebarLayout

SidebarLayout sets `height: 100vh` on its `<main>` element. This works standalone (the view IS the viewport) but breaks in Yggdrasil where the view lives inside `.view-pane` (which is sized by its parent, not the viewport).

**Attempted fixes that failed:**
- `height: 100%` — breaks standalone because `html > body > sveltekit-wrappers` don't propagate height
- `height: 100%` + `html, body { height: 100% }` in tokens.css — partially works but SvelteKit's internal wrapper divs may not propagate
- `.view-pane :global(main) { height: 100% }` override in Yggdrasil — specificity/cascade issues

**The real fix:** SidebarLayout should not assume it knows its own height. The height must come from the container.

### 2. `position: fixed` on Sidebar

SidebarLayout's `.sidebar` uses `position: fixed` which positions relative to the **viewport**, not the containing element. In Yggdrasil, the sidebar escapes `.view-pane` and overlaps other views.

**The real fix:** Use `position: absolute` inside a `position: relative` parent — positions relative to the container, not the viewport.

---

## Unused Components

These exist in `ui/components/` but are imported by zero apps:

| Component | Why Unused |
|-----------|------------|
| Panel | Apps define their own content containers |
| Input | Apps use native `<input>` directly |
| Collapsible | Svalinn uses native `<details>` instead |
| ListItem | No app needs it |

Decision needed: remove or document as reserved.

---

## Spacing Leaks

Three components impose their own margin:
- `SearchInput` — `margin-bottom: var(--space-xl)` on wrapper
- `FilterBanner` — `margin-bottom: var(--space-xl)` on banner
- `Collapsible` — `margin-bottom: var(--space-md)` on collapsible

Components should not impose external spacing. The consumer decides margins via gap/padding on the parent container.

---

## What Needs to Happen

### 1. Fix the Height Chain

SidebarLayout must work in both contexts without per-deployment spot fixes.

**Option A: `dvh` unit**
Use `height: 100dvh` (dynamic viewport height) which accounts for browser chrome changes. Falls back gracefully. Still assumes the component IS the viewport — doesn't fix the Yggdrasil embedding problem.

**Option B: Container-relative height**
Remove explicit height from SidebarLayout. Let the consumer provide it:
- Standalone: the page component sets `height: 100vh` on a wrapper
- Yggdrasil: `.view-pane` provides the height via `inset: 0`

This is the correct separation: SidebarLayout fills its container, consumer decides container size.

**Option C: Prop-driven**
Add a `height` prop to SidebarLayout. Default to `100vh`, Yggdrasil passes `100%`.

### 2. Fix the Sidebar Positioning

Replace `position: fixed` with `position: absolute` on `.sidebar`. Requires the parent `<main>` to be `position: relative`.

This makes the sidebar position relative to its container, not the viewport. Works identically in standalone (container IS viewport-sized) and in Yggdrasil (container is `.view-pane`-sized).

### 3. Document the Layout Contract

Each shared component must document:
- What CSS constraints it requires from its parent
- What CSS constraints it provides to its children
- What it does NOT do (so consumers know what they must handle)

### 4. Consolidate Layout Patterns

The four apps use three different root layout patterns:
- SidebarLayout (Svalinn, Kvasir)
- Flex column with 100vh (Hlidskjalf)
- Flex with 100vh (Ratatoskr)

These should converge on shared patterns, even if Hlidskjalf and Ratatoskr don't use SidebarLayout.

---

## Component Usage Map

```
                    Hlidskjalf  Svalinn  Kvasir  Ratatoskr  Yggdrasil
SidebarLayout                      X        X
Button                             X        X        X
Badge                              X        X
TreeNode                           X        X
StatCard                           X
Select                             X
SearchInput                        X
FilterBanner                       X
ThemeSwitcher                                                   X
Panel
Input
Collapsible
ListItem
```

---

## Priority

1. Fix SidebarLayout height + sidebar positioning (architectural)
2. Remove or flag unused components (cleanup)
3. Fix spacing leaks (minor)
4. Document layout contract (prevention)
