# Feature Distribution Audit — Yggdrasil Platform

**Date:** 2026-03-15
**Scope:** All 5 apps (Hlidskjalf, Svalinn, Kvasir, Ratatoskr, Yggdrasil), shared UI library (`@yggdrasil/ui`)
**Method:** Full source read of every Svelte component, layout, route page, CSS file, and shared library export

---

## Executive Summary

The platform has a well-defined shared component library (`@yggdrasil/ui`) with 13 exported components and a CSS design token system. However, feature distribution is significantly uneven:

1. **Theme switching is Yggdrasil-only.** The `ThemeSwitcher` component exists in the shared library but is only rendered in Yggdrasil's tab strip. All 4 standalone apps are locked to the default "darkly" theme with no way to switch. The tokens.css file defines 4 complete themes, but standalone users cannot access 3 of them.

2. **Font/text controls are Kvasir-only.** Font size adjustment (10-24px), font family cycling (mono/dyslexie/sans/serif), and word wrap modes (no wrap, 79-char, fit-to-width) exist only in Kvasir. No other app offers any text customization.

3. **Sidebar layout is shared but unevenly adopted.** Svalinn and Kvasir use `SidebarLayout` with full resize capability. Hlidskjalf and Ratatoskr have custom full-height layouts with no sidebar.

4. **No state persistence** exists for any UI preference except theme choice (localStorage in ThemeSwitcher). Font size, font family, wrap mode, sidebar width, filter state, and view mode all reset on reload.

5. **Accessibility is minimal and inconsistent.** ARIA attributes appear on some interactive elements but coverage is incomplete. No skip navigation, no focus management, no screen reader announcements. Only `base.css` provides a `:focus-visible` outline.

6. **Code duplication** is moderate. The code viewer (syntax-highlighted `<pre>` with line numbers) is duplicated between `KvasirView.svelte` and `JsonlViewer.svelte` with identical CSS. Error banner styling is duplicated across Kvasir and Ratatoskr. The app title pattern (`.app-name :: .subtitle`) is re-implemented in every view with near-identical CSS.

---

## Feature Matrix

| Feature | Hlidskjalf | Svalinn | Kvasir | Ratatoskr | Yggdrasil | Location |
|---------|-----------|---------|--------|-----------|-----------|----------|
| **Theming** | | | | | | |
| tokens.css import | Yes (layout) | Yes (layout) | Yes (layout) | Yes (layout) | Yes (layout) | Shared |
| base.css import | Yes (layout) | Yes (layout) | Yes (layout) | Yes (layout) | Yes (layout) | Shared |
| Theme switcher UI | No | No | No | No | Yes | Yggdrasil-only |
| 4 themes (darkly/light/warm-dark/cool-dark) | Tokens exist | Tokens exist | Tokens exist | Tokens exist | Switchable | Shared CSS, local UI |
| Theme persistence (localStorage) | N/A | N/A | N/A | N/A | Yes | ThemeSwitcher |
| **Text/Font Controls** | | | | | | |
| Font size adjustment | No | No | Yes (10-24px) | No | No | Kvasir-only |
| Font family cycling | No | No | Yes (4 families) | No | No | Kvasir-only |
| Word wrap modes | No | No | Yes (3 modes) | No | No | Kvasir-only |
| State persistence | N/A | N/A | No | N/A | N/A | -- |
| **Layout** | | | | | | |
| SidebarLayout (shared) | No | Yes | Yes | No | No | Shared component |
| Sidebar resize | No | Yes (180-600px) | Yes (180-600px) | No | No | SidebarLayout |
| Sidebar show/hide | No | Yes | Yes | No | No | SidebarLayout |
| Sidebar close button | No | Yes | No (derived) | No | No | App-specific |
| Tab strip | No | No | No | No | Yes | Yggdrasil-only |
| Full-height layout | Yes (custom) | Yes (SidebarLayout) | Yes (SidebarLayout) | Yes (custom) | Yes (shell) | Mixed |
| **Navigation** | | | | | | |
| File tree (TreeNode) | No | Yes | Yes | No | No | Shared component |
| Directory navigation (up/zoom) | No | Yes | Yes | No | No | App-local |
| Directory picker dialog | No | Yes | Yes | No | No | App-local |
| Tab switching | No | No | Yes (code/data/preview/inspect/jsonl/table) | No | Yes (4 app tabs) | App-local |
| View mode switching | No | Yes (by_file/by_code/by_tool) | No | No | No | Svalinn-only |
| **Scrolling** | | | | | | |
| Auto-scroll toggle | Yes | No | No | No | No | Hlidskjalf-only |
| Scroll position preservation | No | No | No | No | No | None |
| Scroll container (`overflow-y: auto`) | Yes (.feed) | Yes (SidebarLayout) | Yes (SidebarLayout) | Yes (.main-content) | Yes (.view-pane) | Mixed |
| **Filtering** | | | | | | |
| Kind filter (toggle buttons) | Yes | No | No | No | No | Hlidskjalf-only |
| Priority filter | Yes (min level) | No | No | No | No | Hlidskjalf-only |
| Severity filter | No | Yes (dropdown) | No | No | No | Svalinn-only |
| Tool filter | No | Yes (dropdown) | No | No | No | Svalinn-only |
| Text search | No | Yes (SearchInput) | No | No | No | Svalinn-only |
| File filter (tree selection) | No | Yes | No | No | No | Svalinn-only |
| FilterBanner (active filter display) | No | Yes | No | No | No | Svalinn uses shared |
| Row filter (table) | No | No | Yes (TableViewer) | No | No | Kvasir-only |
| Dotfile toggle | No | No | Yes | No | No | Kvasir-only |
| **Content Rendering** | | | | | | |
| Syntax highlighting (hljs) | No | No | Yes | No | No | Kvasir-only |
| Markdown preview (marked) | No | No | Yes | No | No | Kvasir-only |
| JSON Schema inspector | No | No | Yes | No | No | Kvasir-only |
| Format conversion (JSON/YAML/TOML/TOON/RON) | No | No | Yes | No | No | Kvasir-only |
| JSONL viewer with scrubber | No | No | Yes | No | No | Kvasir-only |
| Table viewer (CSV/TSV/Parquet) | No | No | Yes | No | No | Kvasir-only |
| Quality report renderer | Yes | No | No | No | No | Hlidskjalf-only |
| Traffic report renderer | Yes | No | No | No | No | Hlidskjalf-only |
| JSON fallback (pre+stringify) | Yes | No | No | No | No | Hlidskjalf-only |
| D3 graph visualization | No | No | No | Yes | No | Ratatoskr-only |
| Code viewer (line numbers) | No | No | Yes (duplicated) | No | No | Kvasir-local |
| **Data Display Components** | | | | | | |
| StatCard | No | Yes | No | No | No | Shared component |
| Badge | No | Yes (via TreeNode) | No | No | No | Shared component |
| Collapsible | No | No | No | No | No | Shared (unused) |
| Panel | No | No | No | No | No | Shared (unused?) |
| **Real-time Features** | | | | | | |
| Tauri event listener (datagram) | Yes | No | No | No | No | Hlidskjalf-only |
| Connection status indicator | Yes | No | No | No | No | Hlidskjalf-only |
| Speech alerts | Yes | No | No | No | No | Hlidskjalf-only |
| Auto-expand toggle | Yes | No | No | No | No | Hlidskjalf-only |
| **Graph Features** | | | | | | |
| D3 force simulation | No | No | No | Yes | No | Ratatoskr-only |
| Zoom/pan (D3 zoom) | No | No | No | Yes | No | Ratatoskr-only |
| Node drag | No | No | No | Yes | No | Ratatoskr-only |
| Node selection panel | No | No | No | Yes | No | Ratatoskr-only |
| Stats panel toggle | No | No | No | Yes | No | Ratatoskr-only |
| **Error Handling** | | | | | | |
| Error banner | No | No | Yes | Yes | No | Duplicated CSS |
| Empty state message | Yes | Yes | Yes | Yes | No | Per-app, similar pattern |
| Loading state | No | Yes | Yes | Yes | No | Per-app |
| **Keyboard/Accessibility** | | | | | | |
| role="button" on clickables | Partial | Partial | No | No | No | Inconsistent |
| tabindex on clickables | Partial | Partial | No | No | No | Inconsistent |
| Enter/Space key handlers | Partial | Partial | Partial | No | No | Inconsistent |
| Arrow key navigation | No | No | Yes (JsonlViewer) | No | No | Kvasir-only |
| :focus-visible outline | Yes (base.css) | Yes (base.css) | Yes (base.css) | Yes (base.css) | Yes (base.css) | Shared |
| ARIA separator (resize) | N/A | Yes (SidebarLayout) | Yes (SidebarLayout) | N/A | N/A | Shared |
| Skip navigation | No | No | No | No | No | None |

---

## Per-Feature Deep Dive

### 1. Theming

**Where it lives:** Design tokens in `ui/css/tokens.css`. ThemeSwitcher component in `ui/components/ThemeSwitcher.svelte`. Rendered only in `yggdrasil/src/routes/+page.svelte` (line 119).

**How it works:** `tokens.css` defines CSS custom properties on `:root` (default "darkly" dark blue-purple theme) with 3 override blocks: `body[data-theme="light"]`, `body[data-theme="warm-dark"]`, `body[data-theme="cool-dark"]`. ThemeSwitcher reads/writes `localStorage("yggdrasil-theme")` and toggles `document.body.dataset.theme`.

**What's missing:** No standalone app renders ThemeSwitcher. All 4 standalone apps are permanently on the darkly theme. The standalone +page.svelte files for Hlidskjalf, Svalinn, Kvasir, and Ratatoskr do not import or render ThemeSwitcher. Since all 4 themes are fully defined with complete token coverage (backgrounds, text, severity colors, syntax highlighting, graph visualization, priority tints), adding the switcher to standalone apps would be trivially correct.

**All 4 themes are complete:** Each theme defines all ~55 CSS custom properties. No theme is missing tokens that another defines.

### 2. Text / Font Controls

**Where it lives:** Entirely within `kvasir/src/lib/KvasirView.svelte` (lines 69-109).

**How it works:**
- `viewerFontSize`: `$state(14)`, range 10-24, exposed via `fontSizeDown()`/`fontSizeUp()` buttons
- `viewerFontFamily`: `$state("mono")`, cycles through `["mono", "dyslexie", "sans", "serif"]` via `cycleFontFamily()`
- `wrapMode`: `$state("nowrap")`, cycles through `["nowrap", "wrap79", "wrapwidth"]` via `cycleWrap()`
- Applied via CSS custom properties `--vfs` and `--vff` on a `.viewer-settings` wrapper using `:global()` selectors

Font family CSS map:
- mono: system monospace
- dyslexie: 'Dyslexie', sans-serif (accessibility font)
- sans: system sans-serif
- serif: Georgia, Times New Roman

**State persistence:** None. All values reset to defaults on page reload (font size 14, mono, no wrap).

**What's missing:** These controls would benefit Svalinn (code display), Hlidskjalf (event feed text), and Ratatoskr (metadata display). The dyslexie font option is a significant accessibility feature that only Kvasir users can access.

### 3. Layout Controls

**SidebarLayout usage:**
- **Svalinn:** Uses SidebarLayout with show/hide toggle (`showTree` state), close button (`onCloseSidebar`), and a re-show button in the controls area. The sidebar shows a file tree of QA sidecars.
- **Kvasir:** Uses SidebarLayout with show/hide based on derived state (`showTree = treeRoot !== null`). Has a `fileView` mode that hides the sidebar and goes full-width. No explicit close button — sidebar disappears when entering file-focused view.
- **Hlidskjalf:** Custom full-height flex layout (`.watchtower` with header/filters/feed sections). No sidebar.
- **Ratatoskr:** Custom full-height flex layout (`main > .main-content`). No sidebar.
- **Yggdrasil:** Shell layout with `.view-area` (stacked view panes) and `.tab-strip` (28px vertical strip on the right).

**Sidebar resize:** Handled by SidebarLayout's pointer capture drag handler. Width state is `$state(280)`, clamped to `minWidth=180` / `maxWidth=600`. Not persisted.

**Panel component:** Defined in shared library but not used by any app's view component currently (based on source reading). The `Panel` component is a simple card with optional header.

### 4. Scrolling Behavior

**Hlidskjalf:** Has explicit auto-scroll feature. `.feed` div with `overflow-y: auto` and `bind:this={feedElement}`. When `autoScroll` is true and a new datagram arrives, `requestAnimationFrame` scrolls to bottom. Toggle control in the filter bar.

**Svalinn:** Scrolling handled by SidebarLayout's `.main-content` (`overflow-y: auto`) and `.sidebar-content` (`overflow-y: auto`). No auto-scroll or position preservation.

**Kvasir:** Same SidebarLayout scrolling. Code viewer uses `overflow-x: auto` on `<pre>`. When opening a file with a line number (via `openLine` prop), uses `requestAnimationFrame` + `scrollIntoView({ block: "center" })` to jump to the target line. The `$effect` that does this is on `openFile` changes.

**Ratatoskr:** `.main-content` with `overflow: auto`. Graph container has `overflow: hidden` (graph pans via D3 zoom transform instead).

**Yggdrasil:** `.view-pane` divs with `overflow: auto`. Each view preserves its own scroll position because panes use visibility toggle rather than conditional rendering.

**Consistency issue:** Hlidskjalf is the only app with explicit scroll management. No app preserves scroll position across reloads.

### 5. Keyboard Shortcuts / Accessibility

**Global keyboard shortcuts:**
- **Kvasir JsonlViewer only:** Arrow keys for JSONL entry navigation (Up=prev, Down=next, Left=first, Right=last). This is a global `window.addEventListener("keydown")` which could conflict with other functionality when JsonlViewer is mounted but not visible.

**Element-level keyboard access:**
- **Hlidskjalf HlidskjalfView:** Event rows with payloads have `role="button"`, `tabindex`, Enter/Space handlers for expand/collapse.
- **Hlidskjalf TrafficReport:** Section headers and tool summaries have `role="button"`, `tabindex`, Enter/Space handlers.
- **Svalinn SvalinnView:** Issue rows have `role="button"`, `tabindex`, Enter handler for opening in editor.
- **Kvasir KvasirView:** Directory input has `onkeydown` for Enter to trigger loadTree.
- **Ratatoskr RatatoskrView:** No keyboard accessibility at all. No role, tabindex, or keyboard handlers on any element.

**Shared components:**
- `TreeNode`: `role="button"`, `tabindex="0"`, Enter key handler. Good.
- `ListItem`: `role="button"`, `tabindex="0"`, Enter key handler. Good.
- `SidebarLayout` resize handle: `role="separator"`, `aria-orientation="vertical"`. Good.
- `Button`: No ARIA attributes (native `<button>` element, so acceptable).

**Missing across all apps:**
- No skip navigation links
- No landmark roles (`<main>`, `<nav>`, `<aside>` exist but without ARIA labels to differentiate)
- No `aria-live` regions for dynamic content (Hlidskjalf event feed would benefit)
- No focus management when views switch in Yggdrasil
- No `aria-label` on icon-only buttons (Yggdrasil tab strip letter buttons, Hlidskjalf filter/clear buttons)

### 6. State Management Patterns

All apps use Svelte 5 runes consistently:

**`$state()` for reactive local state:**
Every view component manages its own state entirely via `$state()`. No shared stores, no context API, no cross-component state. Examples:
- Hlidskjalf: `datagrams`, `connected`, `autoScroll`, `enabledKinds`, `filterPriorityMin`, `speechMinPriority`, `expandedRows`, `autoExpand`
- Svalinn: `directory`, `scanResult`, `loading`, `viewMode`, `severityFilter`, `toolFilter`, `searchQuery`, `showTree`, `treeRoot`, `selectedFile`
- Kvasir: `directory`, `treeRoot`, `selectedFile`, `fileContent`, `activeTab`, `dataFormats`, `selectedFormat`, `viewerFontSize`, `viewerFontFamily`, `wrapMode`
- Ratatoskr: `graphData`, `stats`, `selectedNode`, `error`, `loading`, `showStats`

**`$derived()` / `$derived.by()` for computed values:**
Used consistently:
- Hlidskjalf: `filteredDatagrams`, `datagramKinds`, `stats`, `allEnabled`
- Svalinn: `baseFilteredIssues`, `filteredIssues`, `groupedIssues`, `filteredDataByPath`, `filteredStats`
- Kvasir: `displayContent`, `highlightedContent`, `renderedMarkdown`
- Ratatoskr: None

**`$effect()` for side effects:**
- Kvasir: Watches `openFile` prop changes, triggers file loading
- Kvasir JsonlViewer: Watches `path` and `refreshKey` changes
- Kvasir TableViewer: Watches `path` and `refreshKey` changes

**`$bindable()` for two-way binding:**
- Shared Input/Select components use `$bindable()` for value
- Kvasir FormatControls uses `$bindable()` for `selectedFormat`

**Props pattern:** All view components accept a `commands` prop with default values mapping bare names to bare names (standalone mode). Yggdrasil overrides with prefixed names. This is the documented view component contract and it works correctly.

**No state persistence:** Zero use of localStorage or sessionStorage in any view component or app-specific code. Only ThemeSwitcher persists state.

### 7. Error Handling in UI

**Error banners:**
- **Kvasir:** `<section class="error-banner">{error}</section>` rendered when `error` is truthy. Red background (`--severity-error`), white text.
- **Ratatoskr:** Identical pattern and identical CSS: `<section class="error-banner">{error}</section>` with identical styling.
- **Hlidskjalf:** No error banner. Connection failure is shown via the status badge changing to "disconnected" with a red background.
- **Svalinn:** No error banner. Scan failures silently result in `scanResult` remaining null.

**Loading states:**
- **Svalinn:** Button text changes: "Scanning..." / "Running..." via ternary on `loading`/`sagaRunning` state.
- **Kvasir:** Button text changes: "Loading..." via ternary on `loading`.
- **Ratatoskr:** Button text changes: "Loading..." via ternary on `loading`.
- **Hlidskjalf:** No loading state. Monitor starts on mount. Connection status badge serves as indicator.

**Empty states:**
- **Hlidskjalf:** Centered div with icon and message: watching icon when connected, hourglass when connecting.
- **Svalinn:** "Select a directory to view .qa sidecars generated by Saga" centered text.
- **Kvasir:** Two empty states: "Select a file from the tree to view its contents" and "Select a directory to browse files."
- **Ratatoskr:** "Load a graph JSON file or generate a sample graph" centered text.
- **Pattern:** All use a `.empty-state` class with centered text and muted color. CSS is duplicated per-app.

**Silent failure pattern:** Svalinn wraps most invocations in try/catch with empty catch blocks. Comments like `// scan failure surfaced via empty scanResult` indicate the pattern is intentional but provides no user feedback on failure.

### 8. Content Display

**Code viewer with line numbers:**
Duplicated between `KvasirView.svelte` and `JsonlViewer.svelte`. Both use:
```svelte
<section class="code-viewer">
  <pre><code>{#each ... as line, i}
    <span class="line-number">{i+1}</span>
    <span class="line-content">{@html highlighted}</span>
  {/each}</code></pre>
</section>
```
CSS is identical: `.code-viewer`, `.code-viewer pre`, `.line-number`, `.line-content`, `.wrap79`, `.wrapwidth` classes.

**Syntax highlighting:** `highlight.js` is imported only in Kvasir (KvasirView + JsonlViewer). Language mapping function `getHljsLanguage()` exists in KvasirView and is passed as a prop to JsonlViewer.

**Markdown rendering:** `marked` library used only in Kvasir. `MarkdownPreview.svelte` renders `{@html content}` with global styles for headings, code, lists, blockquotes, tables.

**JSON display:** Hlidskjalf uses `JSON.stringify(payload, null, 2)` in a `<pre>` tag for unknown payload types. Ratatoskr uses the same for node metadata.

**Table display:** Kvasir's `TableViewer` is a full-featured sortable, filterable table with sticky headers and alternating row colors. No other app has table display.

**Tree display:** `TreeNode` (shared) is used by both Svalinn and Kvasir. Svalinn extends it with badge counts and severity colors. Kvasir extends it with file type icons.

### 9. Navigation Patterns

**Sidebar tree navigation:** Svalinn and Kvasir both implement directory-up navigation and double-click-to-zoom using `parentDir()` and `zoomToDirectory()` functions. These functions are duplicated with nearly identical logic.

**Tab strip (Yggdrasil):** Vertical letter strip on the right edge. Uses lazy mounting (`mounted` Set) with explicit `selectTab()` that adds to the set. View panes are positioned absolutely and toggled via `visibility: hidden` / `pointer-events: none`. This preserves scroll and state. Has a "clear dormant" button.

**Content tabs (Kvasir):** Horizontal tab bar for Code/Preview/Data/Inspect/JSONL/Table. Which tabs appear depends on file type flags. Active tab stored in `activeTab` state.

**View mode buttons (Svalinn):** "By File" / "By Error Type" / "By Tool" toggle buttons for grouping issues.

**Breadcrumbs:** None in any app.

### 10. Standalone vs Embedded Divergences

**File open behavior — SIGNIFICANT DIVERGENCE:**
- **Standalone Hlidskjalf:** `onOpenFile` prop is set by `+page.svelte` to call `invoke("open_default", { path })`, which runs `open <path>` via macOS command. Opens files in the system default application.
- **Embedded Hlidskjalf (Yggdrasil):** `onOpenFile` is set to switch to Kvasir tab and pass the path as `openFilePath`. Opens files in the integrated Kvasir viewer.
- This is an intentional and well-designed divergence.

**ThemeSwitcher availability — FEATURE GAP:**
- **Standalone apps:** Cannot switch themes. Permanently locked to darkly.
- **Yggdrasil:** ThemeSwitcher in tab strip allows all 4 themes.

**Kvasir file opening from external:**
- **Standalone Kvasir:** `+page.svelte` listens for `"open-file"` Tauri events and `get_pending_file` command.
- **Yggdrasil:** Same mechanism exists in Yggdrasil's `+page.svelte`, switches to Kvasir tab on file open.

**Command names — CORRECTLY HANDLED:**
All 4 view components accept `commands` prop with defaults that map to bare names. Yggdrasil overrides with prefixed names. This is the documented contract and is correctly implemented.

**Layout containment in Yggdrasil:**
- Hlidskjalf's `.watchtower { height: 100vh }` works in both standalone and embedded (Yggdrasil sets `.view-pane :global(main) { height: 100% }`).
- Svalinn and Kvasir use `SidebarLayout` which sets `main { height: 100vh }`, contained by the same global rule.
- Ratatoskr's `main { height: 100vh }` similarly works.

**No other behavioral divergences found.** The `commands` prop contract successfully isolates all app behavior.

---

## State Management Patterns Summary

| Pattern | Usage | Location |
|---------|-------|----------|
| `$state()` rune | All reactive UI state | Every view component |
| `$derived()` / `$derived.by()` | Computed/filtered views of state | Hlidskjalf, Svalinn, Kvasir |
| `$effect()` | Side effects on state changes | Kvasir (file opening, JSONL loading) |
| `$bindable()` | Two-way binding for child components | Shared Input, Select, FormatControls |
| `$props()` | Component inputs | All components |
| localStorage | Theme persistence only | ThemeSwitcher |
| Svelte stores | None | -- |
| Context API | None | -- |
| Cross-component state | None | -- |
| URL-based state | None | -- |

---

## Shared UI Component Usage

| Component | Hlidskjalf | Svalinn | Kvasir | Ratatoskr | Yggdrasil |
|-----------|-----------|---------|--------|-----------|-----------|
| SidebarLayout | -- | Yes | Yes | -- | -- |
| Button | -- | Yes | Yes (2) | Yes | -- |
| Badge | -- | Via TreeNode | -- | -- | -- |
| Input | -- | -- | -- | -- | -- |
| Select | -- | -- | -- | -- | -- |
| Panel | -- | -- | -- | -- | -- |
| StatCard | -- | Yes | -- | -- | -- |
| TreeNode | -- | Yes | Yes | -- | -- |
| Collapsible | -- | -- | -- | -- | -- |
| ListItem | -- | -- | -- | -- | -- |
| SearchInput | -- | Yes | -- | -- | -- |
| FilterBanner | -- | Yes | -- | -- | -- |
| ThemeSwitcher | -- | -- | -- | -- | Yes |

**Notable: Hlidskjalf imports zero shared UI components.** Its view is entirely self-contained with custom HTML and CSS. This means any improvements to shared components (Button styling, etc.) do not reach Hlidskjalf.

**Unused shared components:** `Input`, `Select`, `Panel`, `Collapsible`, `ListItem` are defined and exported but not used by any view component. Apps use native `<input>`, `<select>`, and custom containers instead. `Collapsible` could replace the `<details>` elements in Svalinn's issue groups. `Panel` could replace custom card containers in multiple apps.

---

## Recommendations for Centralization

### Priority 1 — Immediate Value, Low Risk

**R1: Add ThemeSwitcher to standalone apps.**
Each standalone app's `+page.svelte` or `+layout.svelte` should render `<ThemeSwitcher orientation="horizontal" />` somewhere accessible (e.g., a corner widget or header area). The component is self-contained, reads from localStorage, and requires zero props. All 4 themes are fully defined in tokens.css.

**R2: Persist font/wrap preferences in localStorage.**
Add `localStorage.getItem/setItem` calls to Kvasir's font size, font family, and wrap mode state. Zero risk, immediate quality-of-life improvement.

### Priority 2 — Moderate Value, Low-Medium Risk

**R3: Extract a shared `CodeViewer` component.**
The code viewer pattern (syntax-highlighted `<pre>` with line numbers, wrap modes, font controls) is duplicated between KvasirView and JsonlViewer. Extract to a shared component in `ui/components/`. Props: `content`, `language`, `wrapMode`, `fontSize`, `fontFamily`. Consumers: Kvasir (2 places), potentially Hlidskjalf (JSON payload display), potentially Svalinn (file content preview).

**R4: Extract a shared `AppHeader` component.**
Every view has an identical title pattern: `<h1><span class="app-name">NAME</span> <span class="separator">::</span> <span class="subtitle">Description</span></h1>`. The CSS is duplicated 4 times with only the text differing. Create `<AppHeader name="KVASIR" subtitle="Workspace Inspector" />`.

**R5: Extract a shared `ErrorBanner` component.**
Kvasir and Ratatoskr have identical error banner markup and CSS. Create `<ErrorBanner message={error} />` in the shared library.

**R6: Extract shared `EmptyState` component.**
All 4 apps have similar empty state patterns. Create `<EmptyState icon="eye" message="Watching for events..." />`.

### Priority 3 — High Value, Medium Risk

**R7: Extract font/text controls as a shared component.**
Create `<ViewerControls fontSize={$bindable(14)} fontFamily={$bindable("mono")} wrapMode={$bindable("nowrap")} />`. This would allow Svalinn to offer text customization for issue display, and Hlidskjalf to offer it for the event feed. The dyslexie font option is a meaningful accessibility feature that should be platform-wide.

**R8: Add `aria-live="polite"` to Hlidskjalf's feed.**
The event feed is dynamic content. Screen readers need to be informed of new events. Add `aria-live="polite"` to the feed container, or use `aria-live="assertive"` for critical-priority datagrams.

**R9: Add `aria-label` to icon-only buttons.**
Yggdrasil tab strip buttons, Hlidskjalf filter/clear buttons, and navigation arrows all lack accessible labels.

### Priority 4 — High Value, Higher Risk

**R10: Adopt shared components more consistently.**
Replace native `<input>` with `Input`, `<select>` with `Select`, and custom card containers with `Panel` or `Collapsible` where appropriate. This would improve visual consistency and reduce per-app CSS. Risk: may affect layout if shared component dimensions differ from current inline elements.

**R11: Consider URL-based state for Kvasir.**
Kvasir has the most complex navigation state (directory, selected file, active tab). URL-based state via SvelteKit's `$page` would enable back/forward navigation and deep linking. This would be a significant architectural change.

**R12: Add keyboard navigation to Yggdrasil tab strip.**
The tab strip has no keyboard navigation. Add Cmd+1/2/3/4 shortcuts for tab switching, and arrow key navigation within the strip.

---

## Duplicated CSS Catalog

| CSS Pattern | Apps | Lines (approx) | Candidate for extraction |
|-------------|------|----------------|--------------------------|
| `.app-name` + `.separator` + `.subtitle` header | All 4 views | ~30 lines each | Yes - `AppHeader` |
| `.error-banner` | Kvasir, Ratatoskr | ~7 lines each | Yes - `ErrorBanner` |
| `.empty-state` | All 4 views | ~5 lines each | Yes - `EmptyState` |
| `.code-viewer` + line numbers + wrap modes | KvasirView, JsonlViewer | ~45 lines each | Yes - `CodeViewer` |
| `.controls` panel (bg-secondary, padding, radius) | Svalinn, Kvasir, Ratatoskr | ~5 lines each | Marginal |
| `.directory-row` + `.directory-input` | Svalinn, Kvasir | ~15 lines each | Yes if shared tree nav |
| `.up-btn` | Svalinn, Kvasir | ~15 lines each | Yes - `UpButton` or headerExtra slot |
| `.format-btn` + `.format-selector` | KvasirView, FormatControls, JsonlViewer | ~15 lines each | Yes - shared format selector |
