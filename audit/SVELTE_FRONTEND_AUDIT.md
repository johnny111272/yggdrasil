# Svelte Frontend Audit -- Yggdrasil

**Date:** 2026-03-08
**Scope:** All Svelte frontend code across 5 apps and the shared UI library
**Files examined:** 35 Svelte components, 5 vite configs, 5 svelte configs, 5 package.json files, 1 TypeScript module, 2 CSS files

---

## Critical (breaks in unified shell or is functionally wrong)

### C-1. Port collisions between standalone apps

Three apps share port 1421 and two share 1422. Running any two of these concurrently in dev mode will fail with `strictPort: true`.

| App | Dev Port | HMR Port |
|-----|----------|----------|
| Svalinn | **1420** | **1421** |
| Hlidskjalf | **1421** | **1422** |
| Kvasir | **1421** | **1422** |
| Ratatoskr | **1422** | **1423** |
| Yggdrasil | 1425 | 1426 |

Hlidskjalf and Kvasir are identical (1421/1422). Svalinn's HMR port (1421) collides with Hlidskjalf/Kvasir's dev port. Ratatoskr's dev port (1422) collides with Hlidskjalf/Kvasir's HMR port.

**Files:**
- `/Users/johnny/.ai/smidja/yggdrasil/hlidskjalf/vite.config.js` line 17: `port: 1421`
- `/Users/johnny/.ai/smidja/yggdrasil/kvasir/vite.config.js` line 17: `port: 1421`
- `/Users/johnny/.ai/smidja/yggdrasil/svalinn/vite.config.js` line 17: `port: 1420`, line 21: `port: 1421`
- `/Users/johnny/.ai/smidja/yggdrasil/ratatoskr/vite.config.js` line 17: `port: 1422`

**Impact:** Cannot run multiple standalone apps simultaneously during development. This is a real-world scenario when testing inter-app behavior or comparing views side by side.

### C-2. Hlidskjalf has phantom dependencies: `highlight.js` and `marked`

`/Users/johnny/.ai/smidja/yggdrasil/hlidskjalf/package.json` lists `highlight.js` and `marked` as dependencies. Neither is imported anywhere in Hlidskjalf's source code. They are used by Kvasir, not Hlidskjalf.

```json
"highlight.js": "^11.11.1",
"marked": "^17.0.1"
```

This wastes disk space (each app has isolated `node_modules/`) and misleads developers about what Hlidskjalf actually uses.

### C-3. Kvasir renders unsanitized HTML via `{@html}`

`/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/lib/KvasirView.svelte` line 491:
```svelte
{@html renderedMarkdown}
```

And line 495:
```svelte
{@html highlighted}
```

`renderedMarkdown` comes from `marked(fileContent.content)` where `fileContent` is loaded from arbitrary files on disk via Tauri. While this is a local desktop app (not a web app), `marked()` does not sanitize by default. A crafted `.md` file containing `<script>` tags or `<img onerror=...>` payloads would execute in the WebView context, which has Tauri IPC access. This is an XSS vector that could invoke Tauri commands.

The `{@html highlighted}` usage is less dangerous since `highlight.js` HTML-encodes its output, but it still bypasses Svelte's template escaping.

### C-4. Ratatoskr renders all views on mount, not on demand

`/Users/johnny/.ai/smidja/yggdrasil/yggdrasil/src/routes/+page.svelte` lines 19-49: All four view components are rendered simultaneously and toggled via CSS `visibility`. This means:

- Ratatoskr's `onMount` fires immediately and calls `loadSampleGraph()` plus sets up a resize listener, even if the user never navigates to the Ratatoskr tab.
- Hlidskjalf starts its Unix socket listener immediately.
- All four apps compete for resources simultaneously.

The `visibility: hidden` approach preserves DOM state when switching tabs (good), but the unconditional mount of all views is wasteful and causes unnecessary background activity.

---

## Structural (patterns that will cause pain as the project grows)

### S-1. Hlidskjalf uses zero shared UI components

HlidskjalfView.svelte imports only `GleipnirReport.svelte` (its own sibling) and Tauri APIs. It hand-rolls:
- Stat cards (`.stat`, `.stat-value`, `.stat-label` at lines 338-359) -- the shared `StatCard` component does this
- Filter buttons (`.filter-btn` at lines 373-393) -- these could use shared `Button` with a variant
- The entire layout structure, ignoring `SidebarLayout`

The shared UI library exists and the other three apps use it. Hlidskjalf does not. This creates an inconsistency where Hlidskjalf's UI will visually drift from the other apps over time.

### S-2. Duplicated tree node implementations

Kvasir implements its own tree node rendering inline via a `{#snippet renderNode}` (KvasirView.svelte lines 345-363), duplicating the shared `TreeNode` component that Svalinn uses. Both implementations have the same structure: icon + name, click handler, recursive children. Kvasir's version is less complete -- it lacks badge counts and keyboard navigation that `TreeNode` provides.

Kvasir's inline tree node CSS (lines 533-562) also duplicates the shared `TreeNode` component's styles nearly verbatim.

### S-3. Kvasir is a 900-line monolith

`/Users/johnny/.ai/smidja/yggdrasil/kvasir/src/lib/KvasirView.svelte` is 898 lines. It contains:
- File tree browsing logic
- File loading and content display
- Syntax highlighting via highlight.js
- Markdown rendering via marked
- Format conversion (JSON/YAML/TOML/TOON) with token statistics
- Schema inspection delegation
- Tab management (code/data/preview/inspect)

This is too many responsibilities for one component. The code viewer, markdown preview, format converter, and tree browser should each be separate components.

### S-4. Svalinn duplicates filter logic three times

`/Users/johnny/.ai/smidja/yggdrasil/svalinn/src/lib/SvalinnView.svelte` has three separate `$derived.by()` blocks that apply the same filter chain (`severityFilter`, `toolFilter`, `searchQuery`):
- `filteredIssues` (lines 193-216) -- for the main issue list
- `filteredDataByPath` (lines 273-303) -- for tree badge counts
- `filteredStats` (lines 336-367) -- for the stat cards

Each independently re-filters `scanResult.issues` with the same conditions. The filter logic should be extracted once and the three derived values should build on a single filtered base.

### S-5. No error state rendering for most Tauri command failures

Every app catches Tauri command errors with `console.error()` and moves on. The user sees nothing.

| App | Error handling pattern |
|-----|----------------------|
| Hlidskjalf | `console.error("Failed to start listener:", err)` -- line 122. User sees "disconnected" status forever. No indication of what failed. |
| Svalinn | `console.error("Scan failed:", e)` -- lines 100, 125, 149, 179, 189. All silent. User stares at an empty screen with no feedback. |
| Kvasir | Sets `error = String(e)` and renders an error banner -- line 117, line 390. **This is the only app that does error handling correctly.** |
| Ratatoskr | Sets `error = String(e)` and renders an error banner -- line 95, line 112, line 129. Also correct. |

Hlidskjalf and Svalinn are the worst offenders. Svalinn has 5 separate `try/catch` blocks that all swallow errors to console.

### S-6. Reactive state mutation by reference in Svalinn

`/Users/johnny/.ai/smidja/yggdrasil/svalinn/src/lib/SvalinnView.svelte` line 155:
```typescript
treeRoot = treeRoot;
```

This self-assignment exists to "trigger reactivity" after mutating nested properties of `treeRoot` (lines 134-153 mutate `node.expanded`, `node.loading`, `node.children` directly). In Svelte 5 with `$state()`, deep mutations on plain objects do trigger reactivity without this hack because `$state` creates a proxy. But the mutations on lines 135-153 happen on a node found by `findNode()` traversal, which returns a reference into the deep proxy. The self-assignment is unnecessary in Svelte 5 but is harmless. It is confusing and should be removed with a comment explaining why.

### S-7. Missing `onDestroy` / cleanup in several components

- Hlidskjalf's `onMount` returns an unlisten function, but the socket listener started via `invoke(commands.start_listener)` is never stopped. If the component unmounts (tab switch that destroys DOM, or Yggdrasil switches to lazy mounting), the Rust-side listener continues running.
- Ratatoskr creates a D3 simulation with `d3.forceSimulation()` and a window resize listener. The resize listener is cleaned up, but the simulation is never stopped on unmount (`.stop()` is never called). D3 simulations run an internal timer that continues ticking after the component is destroyed.
- Ratatoskr stores `simulation` as a module-level `let` (line 66), not as `$state`. This is correct for a D3 simulation reference, but means Svelte cannot track it. The `svgElement` on line 65 is also not `$state` -- `bind:this` in Svelte 5 should use `$state` for the element reference.

### S-8. Yggdrasil's Vite aliases do not inform SvelteKit about external sources

`/Users/johnny/.ai/smidja/yggdrasil/yggdrasil/vite.config.js` sets up `resolve.alias` for `$hlidskjalf`, `$svalinn`, `$kvasir`, `$ratatoskr`, but does not configure SvelteKit's `kit.alias` in `svelte.config.js`. SvelteKit may not properly handle HMR, TypeScript resolution, or preprocessor application for files resolved via Vite-only aliases. The svelte config should mirror these aliases.

---

## Per-App Findings

### Shared UI Library (ui/)

**Strengths:**
- Clean Svelte 5 runes usage throughout -- `$props()`, `$state()`, `$derived()`, `$bindable()`, `{#snippet}`, `{@render}`. No Svelte 4 patterns.
- Well-typed: every component has a TypeScript `interface Props`.
- Consistent use of design tokens from `tokens.css`.
- `SidebarLayout` has a well-implemented resize handle with pointer capture.

**Issues:**

1. **`SearchInput` duplicates `Input`**. `SearchInput` (47 lines) is a styled `<input type="search">` with a wrapper div. `Input` (48 lines) already accepts `type="search"`. They have different styling (SearchInput uses `--bg-secondary`, Input uses `--bg-primary`), but this should be a variant, not a separate component.

2. **`Input` focus style removes the outline entirely** (line 38-39: `outline: none`). This conflicts with `base.css` line 66-68 which defines a `:focus-visible` outline. The `Input` component's `:focus` rule overrides the global `:focus-visible` rule, removing keyboard focus visibility. This is an accessibility regression.

3. **`Select` focus style also removes outline** (line 33: `outline: none`). Same problem as Input.

4. **`SidebarLayout.main-content` has `max-width: 1200px`** (line 161). This is a layout opinion baked into the shared component. Hlidskjalf needs full-width for its event feed. This forces Hlidskjalf to not use `SidebarLayout` at all (see S-1) or override it.

5. **`TreeNode` uses emoji for file icons** (lines 48-50: folder/file emojis). Emoji rendering varies by OS version and is not accessible to screen readers. The icons carry semantic meaning (file vs directory vs loading) but have no `aria-label`.

6. **`Badge` has poor contrast for "neutral" variant**. The default neutral color is `var(--action-primary)` (#4361ee) with white text on a small pill badge. At `--text-xs` (12px), this may fail WCAG AA for small text.

7. **`ListItem` renders as `<li>` but there is no guarantee the parent provides `<ul>` or `<ol>`**. A `<li>` outside a list container is invalid HTML.

8. **`FilterBanner` is not keyboard accessible**. The "Clear filter" button is a `<button>` (good), but the section element has no `role` attribute and no landmark semantics.

9. **No `index.d.ts` or type declarations**. The `ui/package.json` has `"svelte": "./index.js"` but no TypeScript entry point. Consumers relying on type checking will not get prop type information from the shared library.

### Hlidskjalf

**Component:** `HlidskjalfView.svelte` (563 lines)
**Sibling:** `GleipnirReport.svelte` (294 lines)

**Strengths:**
- Correct `commands` prop contract with defaults for standalone use.
- Correct `./` import for `GleipnirReport`.
- Good use of `$derived` for filtered events and stats.
- Canary events get compact rendering -- good information density decision.
- Speech threshold cycling is a creative UX pattern for accessibility profiles.

**Issues:**

1. **No shared UI component usage at all**. Uses raw `<button>`, `<input>`, `<span>` everywhere. Has its own stat cards, filter buttons, and layout. This is the most mature app and it developed its own design language independent of the shared library.

2. **Events array grows unbounded**. Line 103: `events = [...events, ev]`. There is no cap. A long-running session will accumulate thousands of events in memory, each with a full datagram payload. The `filtered_events` derived value re-filters the entire array on every new event. This will degrade performance over time.

3. **`speech_labels` is declared but never used** (line 140). Dead code.

4. **`speech_threshold` cycling logic is opaque**. Lines 143-148: the cycle goes 3 -> 2 -> 4 -> 5 -> 3. These magic numbers correspond to priority levels but are not named constants. The `speech_label()` function (lines 150-156) duplicates this mapping. Both should reference the same priority constant map.

5. **The `feed_element` non-null assertion** at lines 112-113: `feed_element!.scrollTop = feed_element!.scrollHeight`. The `!` suppresses TypeScript's null check. If `feed_element` is undefined when an event arrives (e.g., component is unmounting), this throws.

6. **No keyboard navigation for event rows**. The filter buttons are `<button>` elements (correct), but individual event rows have no `tabindex` or keyboard interaction. If a user wants to inspect an event's payload details, they must click.

7. **Two hardcoded `rgba()` colors** in styles (lines 477, 481): `rgba(255, 51, 51, 0.08)` and `rgba(255, 153, 0, 0.05)`. These are transparent tints of the severity colors but do not reference the design tokens. They will not update if the color palette changes.

**GleipnirReport.svelte:**
- Well-structured. Good use of typed props.
- All colors reference design tokens.
- Location rendering handles both `line` and `lines` (single vs multi-line violations).
- No interactivity -- locations are not clickable to open in editor. This is a missed opportunity.

### Svalinn

**Component:** `SvalinnView.svelte` (753 lines)

**Strengths:**
- Best adopter of the shared UI library: uses `Button`, `SidebarLayout`, `TreeNode`, `StatCard`, `SearchInput`, `FilterBanner`.
- Correct `commands` prop contract.
- Correct `./` imports (no `$lib/` in the view component -- no sibling components to import).
- Good filtering: severity, tool, search, and file tree selection all compose correctly.
- Issue details (signal/direction/canary) are displayed inline.

**Issues:**

1. **753 lines is too long.** The component handles directory selection, scanning, tree loading, tree toggling, saga running, editor opening, filtering (3x), grouping, and all the rendering. The controls section, stats section, filter section, and results section should each be factored out.

2. **`error` is caught but never shown to the user** (lines 99-101, 124-126, 148-149, 178-180, 188-190). Five `console.error()` calls, zero error UI. Compare Kvasir which renders an error banner.

3. **The `viewMode` type annotation is wrong for Svelte 5**. Line 71:
   ```typescript
   let viewMode: "by_file" | "by_code" | "by_tool" = $state("by_file");
   ```
   This works, but the type annotation is on the variable, not on the `$state()` call. This is fine in practice but means the inferred type of `viewMode` is the narrowed literal `"by_file"` from Svelte's perspective, while the annotation widens it. This is a stylistic inconsistency with how other state is declared (no explicit type annotation elsewhere).

4. **Svalinn's `select` elements use raw `<select>` instead of the shared `Select` component** (lines 441-458). The raw selects have their own inline styling (lines 625-631). This is inconsistent with using `Button` and `SearchInput` from the shared library.

5. **The `commands` prop includes `open_in_editor` and `run_saga`**, but these commands are not in the CONTEXT_MAP's documented Svalinn commands (`scan`, `read_report`). Either the documentation is stale or the commands diverged.

6. **`treeRoot = treeRoot` self-assignment** (line 155) as discussed in S-6.

### Kvasir

**Component:** `KvasirView.svelte` (898 lines)
**Siblings:** `SchemaInspector.svelte` (464 lines), `schema-inspect.ts` (821 lines)

**Strengths:**
- Correct `commands` prop contract.
- Correct `./` imports for `SchemaInspector.svelte` and `schema-inspect.ts`.
- Error handling is visible to the user via an error banner (lines 390-393).
- Tab system (code/data/preview/inspect) is well-implemented.
- Token statistics for format conversion are a unique and useful feature.
- `schema-inspect.ts` is well-structured pure logic with no side effects -- good separation.

**Issues:**

1. **898 lines is the longest component in the codebase.** See S-3 above. The code viewer, markdown preview, and format converter are all inlined.

2. **KvasirView duplicates TreeNode** (see S-2). Kvasir implements its own tree rendering with a `{#snippet renderNode}` block (lines 345-363) and duplicates the `.tree-node`, `.tree-icon`, `.tree-name` CSS classes (lines 533-562). The shared `TreeNode` component exists and Svalinn uses it.

3. **`showTree` is derived but can be false when it should still allow re-opening**. Line 74: `let showTree = $derived(treeRoot !== null)`. This is read-only (`$derived` cannot be set). Compare Svalinn which uses `let showTree = $state(true)` and has a toggle button. Kvasir cannot close/reopen the sidebar because the derived value only depends on whether `treeRoot` exists.

4. **SchemaInspector uses 22 hardcoded hex colors**. The entire component uses a Tokyo Night color palette (`#7aa2f7`, `#bb9af7`, `#7dcfff`, `#9ece6a`, `#e0af68`, `#f7768e`, `#73daca`) that is completely disconnected from the design token system. If the app theme changes, SchemaInspector will not change with it. This is a parallel color system.

5. **SchemaInspector has pixel-based font sizes** (e.g., `0.8rem`, `0.82rem`, `0.78rem`, `0.9rem` at lines 311, 339, 359, 386, 389, 397, 400, etc.). The design tokens define a font size scale (`--text-xs`, `--text-sm`, etc.) but SchemaInspector uses its own arbitrary sizes. `0.82rem` and `0.78rem` are not on any standard scale.

6. **`{@html renderedMarkdown}` XSS vector** (see C-3).

7. **`{@html highlighted}` is used for syntax highlighting** (line 495). While highlight.js HTML-encodes its input, the line is inside a dense single-line template expression that is hard to read and audit:
   ```svelte
   <pre><code>{#each displayContent.split('\n') as line, i}{@const highlighted = highlightedContent.split('\n')[i] || ''}<span class="line-number">{i + 1}</span><span class="line-content">{@html highlighted}</span>
   {/each}</code></pre>
   ```
   This entire `<pre><code>` block is a single line. It is unreadable.

8. **`onMount` and `onDestroy` are not used**. Kvasir has no lifecycle hooks -- it is entirely event-driven via user interactions. This is fine, but the component imports `onMount` from svelte (line 5) and never uses it. Dead import.

9. **`commands` prop includes 5 commands** (`list_directory`, `read_file`, `open_in_editor`, `convert_all_formats`, `is_data_file`), but the CONTEXT_MAP documents only 3 (`list_dir`, `read_file`, `convert`). The actual commands have different names than documented.

### Ratatoskr

**Component:** `RatatoskrView.svelte` (560 lines)

**Strengths:**
- Correct `commands` prop contract.
- No `$lib/` imports (no siblings needed).
- Good D3 integration with zoom, drag, force simulation.
- Error handling renders a visible banner.
- Uses the shared `Button` component.

**Issues:**

1. **`svgElement` is not `$state`** (line 65). In Svelte 5, `bind:this` should target a `$state()` variable. Using a plain `let` means Svelte may not properly track the element reference for reactivity. Currently this is not a problem because `svgElement` is only used imperatively in `renderGraph()`, but it is technically incorrect.

2. **D3 simulation is never cleaned up** (see S-7). The simulation runs an internal timer via `d3.timer()`. When the component unmounts (e.g., Yggdrasil destroys inactive tab DOM), the timer continues running, ticking against a detached SVG. `simulation.stop()` is never called.

3. **7 hardcoded hex colors in the script section** (lines 70-73, 175, 193, 206, 226, 238). D3's declarative API makes it tempting to inline colors, but these should reference the design tokens or be extracted to a configuration object that could be themed.

4. **`renderGraph()` is called from `onMount` and on window resize without debouncing**. Line 295: `window.addEventListener("resize", handleResize)` calls `renderGraph()` synchronously on every resize event. D3 force simulations are expensive to restart. A debounce is needed.

5. **Ratatoskr does not use `SidebarLayout`** but has its own stats panel and node panel as floating overlays. This is architecturally different from the other apps and may be intentional for a graph visualization, but it means Ratatoskr looks and feels different from the rest of the platform.

6. **`selected` import from `@tauri-apps/plugin-dialog` returns `string | null` in Tauri v2**, but line 104 checks `typeof selected === "string"`. The `open()` function in Tauri v2 plugin-dialog returns `string | string[] | null` depending on the `multiple` option. With `multiple: false`, it returns `string | null`. The type guard is correct but overly defensive for the configuration used.

7. **`save` is imported but never validated**. Line 3: `import { open, save } from "@tauri-apps/plugin-dialog"`. The `save()` function is used in `saveGraph()` but `commands.save_graph` is invoked without checking if the command actually exists (it may not be registered in the Tauri backend yet).

### Yggdrasil (unified shell)

**Component:** `+page.svelte` (144 lines)

**Strengths:**
- Correct command prefixing for all four apps.
- Clean Vite alias configuration.
- The tab strip is minimal and functional.
- All four views receive their `commands` prop with correct prefixed names.

**Issues:**

1. **All views mount simultaneously** (see C-4). Every view component runs its `onMount` immediately. Hlidskjalf starts a socket listener, Ratatoskr loads a sample graph and starts a D3 simulation, all before the user ever looks at those tabs.

2. **No keyboard navigation for the tab strip**. The tab buttons are `<button>` elements (correct), but there is no `aria-role="tablist"` / `aria-role="tab"` / `aria-role="tabpanel"` structure. Screen readers cannot determine that this is a tab interface. There are no arrow key handlers for tab switching, which is the expected keyboard pattern for tab UIs (WAI-ARIA Tabs Pattern).

3. **Tab labels use character-by-character rendering** (lines 60-62):
   ```svelte
   {#each tab.label.split("") as char}
     <span class="tab-char">{char}</span>
   {/each}
   ```
   Each character of the tab label is wrapped in its own `<span>`. This means "Hlidskjalf" produces 10 separate DOM nodes. A screen reader will not read this as a word -- it will spell it out letter by letter. This breaks accessibility and is unnecessary for the visual effect (vertical text can be achieved with `writing-mode: vertical-lr` in CSS).

4. **Tab strip is 22px wide** (line 97). This is extremely narrow. The Norse mythology names are long (Hlidskjalf is 10 characters, Ratatoskr is 9). The vertical character-by-character rendering makes each tab tall but the touch/click target is only 22px wide, which fails mobile/accessibility touch target guidelines (minimum 44px).

5. **View panes use `visibility: hidden` but remain in the DOM**. This means all four views are competing for layout, even when hidden. Memory usage scales with 4x the component trees. For a desktop app this is acceptable, but it becomes a problem as view complexity grows.

6. **Yggdrasil's `svelte.config.js` does not declare Vite aliases as SvelteKit aliases** (see S-8). The `$hlidskjalf`, `$svalinn`, `$kvasir`, `$ratatoskr` aliases are only in `vite.config.js`. SvelteKit's type generation and preprocessor do not know about these paths.

---

## Cross-Cutting Concerns

### Accessibility

Accessibility is poor across the entire platform. Specific failures:

- **No ARIA landmark roles.** None of the five apps define `role="main"`, `role="navigation"`, `role="complementary"`, etc. The only ARIA attribute in the entire codebase is `aria-orientation="vertical"` on SidebarLayout's resize handle.
- **No skip links.** No way to bypass the sidebar/tab strip to reach main content.
- **Keyboard focus management is inconsistent.** `TreeNode` and `ListItem` have `role="button"` and `tabindex="0"` (correct). Most other interactive elements do not.
- **`Input` and `Select` destroy focus visibility** by setting `outline: none` on `:focus`, overriding the global `:focus-visible` rule.
- **Color contrast.** `--text-secondary` is `#888` on `--bg-primary` (#1a1a2e). The contrast ratio is approximately 3.8:1, which fails WCAG AA (4.5:1 for normal text).
- **No `prefers-reduced-motion` support.** Ratatoskr's D3 simulation animates continuously. No media query disables animations.
- **Yggdrasil tab strip is inaccessible** (see Yggdrasil issues 2, 3, 4 above).

### Type Safety

Type safety is generally good. Every component uses TypeScript with `lang="ts"`. All props have interface definitions. There are no `any` types. Specific issues:

- `schema-inspect.ts` uses `SchemaNode = Record<string, unknown>` extensively, which requires many `as` casts. This is inevitable for JSON Schema analysis but means runtime type errors are possible.
- Ratatoskr's `GraphEdge` has `source: string | GraphNode` because D3 mutates the edge objects during simulation. This is correct D3 practice but ugly TypeScript.
- The `commands` prop typing is inline (anonymous object types) in every view component instead of being a shared type. If a command name changes, there is no compile-time connection between the Rust command registration and the TypeScript command map.

### Dependency Consistency

All five apps share the same versions for core dependencies:
- `svelte: ^5.0.0`
- `vite: ^6.0.3`
- `@sveltejs/kit: ^2.9.0`
- `@sveltejs/adapter-static: ^3.0.6`
- `@tauri-apps/api: ^2.10.1`
- `@tauri-apps/plugin-dialog: ^2.6.0`
- `@tauri-apps/plugin-opener: ^2`

This is correct. The semantic version ranges mean actual installed versions may differ between apps since each has isolated `node_modules/`. A lockfile per app would guarantee consistency; there is no evidence of lockfiles.

App-specific dependencies:
- Hlidskjalf: `highlight.js`, `marked` (unused -- phantom deps)
- Kvasir: `highlight.js`, `marked` (used)
- Ratatoskr: `d3`, `@types/d3` (used)
- Yggdrasil: `highlight.js`, `marked`, `d3`, `@types/d3` (needed for aliased view imports)

### CSS Architecture

The design token system in `tokens.css` is solid and covers colors, spacing, typography, radius, and shadows. `base.css` provides a clean reset. The problem is adoption:

- Hlidskjalf and GleipnirReport use tokens consistently (except 2 rgba values).
- Svalinn uses tokens consistently.
- Kvasir uses tokens in KvasirView but SchemaInspector has 22 hardcoded colors.
- Ratatoskr has 7 hardcoded colors in script and D3 calls.
- The tab strip in Yggdrasil uses tokens correctly.

There are no component-level CSS variables or CSS custom properties being composed. Each component's `<style>` block is fully scoped (Svelte default), which is fine for isolation but means shared patterns are duplicated rather than abstracted.

### Svalinn Description is Empty

`/Users/johnny/.ai/smidja/yggdrasil/svalinn/package.json` line 4: `"description": ""`. Every other app has a description.

### Hlidskjalf Description is Wrong

`/Users/johnny/.ai/smidja/yggdrasil/hlidskjalf/package.json` line 4: `"description": "File viewer with syntax highlighting"`. Hlidskjalf is an agent watchtower / real-time monitor, not a file viewer. This description belongs to Kvasir.

---

## Recommendations (prioritized)

### 1. Fix port collisions (C-1)

Assign unique, non-overlapping port ranges:

| App | Dev Port | HMR Port |
|-----|----------|----------|
| Svalinn | 1420 | 1421 |
| Hlidskjalf | 1430 | 1431 |
| Kvasir | 1440 | 1441 |
| Ratatoskr | 1450 | 1451 |
| Yggdrasil | 1460 | 1461 |

This takes 5 minutes and unblocks concurrent development.

### 2. Remove phantom dependencies from Hlidskjalf (C-2)

Delete `highlight.js` and `marked` from `hlidskjalf/package.json`. Run `npm install` to clean up `node_modules/`.

### 3. Sanitize markdown in Kvasir (C-3)

Configure `marked` with `sanitize: true` or use DOMPurify before rendering via `{@html}`. This is a real XSS vector in a desktop app with IPC access.

### 4. Add lazy mounting to Yggdrasil (C-4)

Replace the `visibility: hidden` CSS approach with Svelte `{#if}` blocks that only render the active view. Use `{#key}` to preserve state if needed. This prevents all four apps from running simultaneously on load.

Alternatively, add a `visible` or `active` prop to each view component so they can pause their background activity when not shown.

### 5. Refactor Hlidskjalf to use shared UI components (S-1)

At minimum, use `StatCard` for the stat display in the header. Consider using `Button` for filter/clear/voice buttons. This brings Hlidskjalf into visual consistency with the other apps.

### 6. Refactor Kvasir to use shared `TreeNode` (S-2)

Delete the inline tree rendering snippet and CSS. Use the shared `TreeNode` component. If `TreeNode` is missing features Kvasir needs (e.g., file-type icons), extend `TreeNode` rather than duplicating it.

### 7. Add error banners to Hlidskjalf and Svalinn (S-5)

Follow the pattern established by Kvasir and Ratatoskr: maintain an `error` state variable, set it on catch, render a visible banner. At minimum, replace `console.error()` with user-visible feedback.

### 8. Add basic ARIA structure

- Add `role="tablist"` to the Yggdrasil tab strip, `role="tab"` to each tab button, `role="tabpanel"` to each view pane.
- Add `aria-label` to emoji icons in `TreeNode`.
- Fix the character-by-character tab labels -- use `writing-mode: vertical-lr` instead.
- Remove `outline: none` from `Input` and `Select` focus styles.

### 9. Extract SchemaInspector colors to design tokens

Add a semantic color section to `tokens.css` for code/schema visualization (type colors, requirement colors, etc.) and update SchemaInspector to use them. The Tokyo Night palette is aesthetically fine -- it just needs to live in the token system so it can be updated centrally.

### 10. Cap the Hlidskjalf event buffer

Add a maximum event count (e.g., 5000) and drop oldest events when the cap is reached. Alternatively, implement virtual scrolling. An unbounded array that grows on every socket event will eventually cause performance degradation.

### 11. Fix package.json metadata

- Set Hlidskjalf description to "Real-time agent monitor".
- Set Svalinn description to "Code quality viewer".

### 12. Break up Kvasir and Svalinn monoliths

Factor out from KvasirView:
- `CodeViewer.svelte` -- syntax highlighting, line numbers
- `MarkdownPreview.svelte` -- markdown rendering
- `FormatConverter.svelte` -- format selector, token statistics
- `FileTree.svelte` -- tree rendering (or just use shared `TreeNode`)

Factor out from SvalinnView:
- Extract the triplicated filter chain into a single derived base
- `IssueGroup.svelte` -- the collapsible issue group rendering
- `IssueRow.svelte` -- individual issue rendering with signal/direction/canary

---

*End of audit.*
