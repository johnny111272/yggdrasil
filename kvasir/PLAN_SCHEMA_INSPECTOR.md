# Kvasir: JSON Schema Inspector Feature

## Goal

When viewing a `.schema.json` file in Kvasir, provide an "Inspect" tab that renders
the same structured analysis as `tools/inspect_schema.py` — on the fly, with no
sidecar generation required.

## Source Reference

`/Users/johnny/.ai/spaces/bragi/tools/inspect_schema.py` (~1260 lines Python)

The Python script reads a JSON Schema file and produces an interactive HTML report
showing per-field requirement status, type summaries, conditional rules, and
extension metadata (x-semantic, x-constraint). The output has:

- Schema header (title, description, version, field/section counts)
- Collapsible section blocks (one per top-level property)
- Per-field lines: name, type, requirement status, extension badges
- Expandable detail panels for semantic/constraint/excludes metadata
- Cross-group conditional annotations
- Toggle controls for showing/hiding descriptions, extensions, conditionals

## Architecture Decision

**Port analysis to TypeScript, render with Svelte components.**

Rationale:
- Schema is already loaded as JSON string in the frontend
- Tree walking / string analysis maps naturally to JS
- Svelte components replace raw HTML string concatenation
- No Python/uv dependency, no subprocess calls, fully self-contained
- Reactive — toggle state is native Svelte reactivity

NOT doing:
- Rust backend command (JSON tree walking is more natural in JS than serde)
- Python subprocess (fragile dependency, not portable)
- iframe with generated HTML (loses Kvasir theming, adds complexity)

## Implementation Plan

### Phase 1: Analysis Module (`src/lib/schema-inspect.ts`)

Port the pure analysis functions from Python to TypeScript.
These are all `dict -> dict` tree walks with no IO.

Functions to port (grouped by concern):

**$ref Resolution**
- `resolveAllRefs(schema)` — pre-resolve all `$defs` pointers
- `deepResolve(node, defs)` — recursive resolver

**Type Analysis**
- `unwrapNullable(schema)` — detect oneOf nullable wrapper
- `typeSummary(prop)` — compact type string (string, array[X], enum[...], etc.)

**Conditional Extraction**
- `describeIfClause(ifClause)` — human-readable condition description
- `extractConditionals(schemaObj)` — if/then/else → field conditions map
- `unwrapNestedPath(obj)` — nested properties chain → dotted path + leaf
- `extractCrossGroupConditionals(schema)` — root-level cross-group conditions
- `fieldRequirement(name, required, conditionals, minProps)` — status + css class

**Extension Metadata**
- `collectExtensions(fieldSchema)` — gather x-semantic, x-constraint
- `collectNotBlock(fieldSchema)` — exclusion patterns
- `notBlockPatterns(notBlock)` — extract pattern strings

**Section Analysis**
- `analyzeObject(schemaObj, crossGroupConds, pathPrefix)` — recursive tree walk
  returning structured data (not HTML) for each field
- `analyzeSchema(rawSchema)` — top-level: resolve refs, extract sections, stats

Output is a structured TypeScript interface tree:
```typescript
interface InspectedSchema {
  title: string;
  description: string;
  version: string;
  sections: InspectedSection[];
  crossGroupConds: Record<string, string[]>;
  stats: { fields: number; sections: number; semantic: [number, number] };
}
interface InspectedSection {
  name: string;
  requirement: { label: string; cssClass: string };
  fields: InspectedField[];
}
interface InspectedField {
  name: string;
  type: string;
  requirement: { label: string; cssClass: string };
  description?: string;
  extensions: { semantic?: any; constraint?: any; notBlock?: any; format?: string };
  children?: InspectedField[];   // nested objects
  alternatives?: InspectedAlt[]; // oneOf array items
}
```

### Phase 2: Svelte Rendering (`src/lib/SchemaInspector.svelte`)

A standalone Svelte component that receives `InspectedSchema` and renders it.

Structure:
- **Controls bar**: toggles for descriptions, semantic, constraints, conditionals
- **Header**: title, description, stats
- **Section blocks**: one per top-level property, collapsible
- **Field lines**: recursive rendering with indentation
- **Extension badges**: inline colored markers
- **Extension panels**: expandable detail (semantic checks, constraint rules, excludes)

Styling follows the existing sidecar theme (Tokyo Night color scheme) but uses
Kvasir's CSS variables for consistency with the rest of the app.

### Phase 3: Integration into Kvasir

- Detect `.schema.json` files (check extension chain, not just `.json`)
- Add "Inspect" tab alongside Code/Data tabs
- When tab is active, parse the JSON, run `analyzeSchema()`, render component
- Memoize analysis result (recompute only when file changes)

### Phase 4: Polish

- Verify against all 13 agent-builder schemas + 7 include-fragment schemas
- Verify cross-group conditionals render correctly
- Verify extension panels toggle correctly
- Test with external JSON Schema files (non-Verdandi)

## File Layout

```
src/
  lib/
    schema-inspect.ts       # Pure analysis functions
    SchemaInspector.svelte  # Rendering component
  routes/
    +page.svelte            # Integration (Inspect tab)
```

## Risk Notes

- The Python script handles edge cases accumulated over many iterations.
  The TypeScript port must handle the same cases. Test against real schemas.
- Cross-group conditionals (deeply nested if/then at root level) are the
  most complex analysis path. Port carefully.
- Extension metadata (x-semantic, x-constraint) is Verdandi-specific but the
  inspector should gracefully skip these for generic JSON Schemas.
