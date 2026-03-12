# Terminology Audit

**Date:** 2026-03-11
**Scope:** Stale naming, incorrect terminology, and inconsistencies following the datagram protocol renames.

---

## Summary

| Category | Count |
|----------|-------|
| **BUG** (code using wrong name -- will break at runtime) | 4 |
| **STALE** (documentation/comment referencing old name -- misleading but not broken) | 21 |
| **CORRECT** (reference reviewed and confirmed accurate) | 6 |

---

## Findings

### 1. `--severity-info` CSS variable does not exist

**Category: BUG**

The CSS variable `--severity-info` is referenced in 4 Svelte source files but does not exist in `ui/css/tokens.css`. The actual variable is `--severity-success`. These references resolve to nothing at runtime, meaning the elements will have no color/background applied by these rules.

| File | Line | Reference |
|------|------|-----------|
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 50 | `return "var(--severity-info)"` (in `priorityColor` for "normal") |
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 344 | `.status.connected { background: var(--severity-info); }` |
| `hlidskjalf/src/lib/HlidskjalfView.svelte` | 439-440 | `.voice-btn.active { border-color: var(--severity-info); color: var(--severity-info); }` |
| `hlidskjalf/src/lib/QualityReport.svelte` | (3 refs) | `return "severity-info"`, `.group.severity-info`, `border-left-color: var(--severity-info)` |
| `svalinn/src/lib/SvalinnView.svelte` | (3 refs) | `return "var(--severity-info)"`, `color: var(--severity-info)` (2x) |
| `kvasir/src/lib/FormatControls.svelte` | (3 refs) | `border: 2px solid var(--severity-info)`, `color: var(--severity-info)` (2x) |

**Fix:** Replace all `--severity-info` with `--severity-success` in all 4 files. Alternatively, add `--severity-info` as an alias in `tokens.css`, but the canonical name is `--severity-success` per the design system.

---

### 2. `socket_emit` crate name in documentation

**Category: STALE**

The crate was renamed from `socket_emit` to `datagram`. The actual `Cargo.toml` files correctly use `datagram`, but 7 documentation files still reference the old name.

| File | Line | Stale Reference | Should Be |
|------|------|-----------------|-----------|
| `CLAUDE.md` | 47 | "nornir's `socket_emit` crate" | "nornir's `datagram` crate" |
| `CLAUDE.md` | 48 | "nornir `socket_emit/src/lib.rs`" | "nornir `datagram/src/lib.rs`" |
| `PLAN_UNIFIED_SHELL.md` | 70 | "deps: socket_emit, tokio..." | "deps: datagram, tokio..." |
| `PLAN_UNIFIED_SHELL.md` | 71 | "re-exports ... from socket_emit" | "re-exports ... from datagram" |
| `PLAN_UNIFIED_SHELL.md` | 178 | "nornir's `socket_emit` crate" | "nornir's `datagram` crate" |
| `PLAN_UNIFIED_SHELL.md` | 181 | `pub use socket_emit::{...}` | `pub use datagram::{...}` |
| `PLAN_UNIFIED_SHELL.md` | 257 | `socket_emit = { path = "../nornir/capability/socket_emit" }` | `datagram = { path = "../nornir/capability/datagram" }` |
| `AUDIT_GUIDE.md` | 59 | "nornir's `socket_emit` crate" | "nornir's `datagram` crate" |
| `HLIDSKJALF_DATAGRAM.md` | 222 | "in nornir's `socket_emit`" | "in nornir's `datagram`" |
| `HLIDSKJALF_DATAGRAM.md` | 223 | "imports from `socket_emit`" | "imports from `datagram`" |
| `DISPLAY_AND_FILTERING.md` | 26 | "the binary with the socket_emit call" | "the binary with the datagram call" |
| `DISPLAY_AND_FILTERING.md` | 52 | "the process with the socket_emit call" | "the process with the datagram call" |
| `YGGDRASIL_FEATURE_REQUESTS.md` | 100 | "like `socket_emit` in nornir" | "like `datagram` in nornir" |
| `schemas/datagram.schema.json` | 17 | "via socket_emit" | "via datagram" |
| `schemas/validate.datagram.schema.json` | 23 | "via socket_emit" | "via datagram" |
| `DATAGRAM_SPECIFICATION.md` | 20 | "via socket_emit" | "via datagram" |
| `DATAGRAM_SPECIFICATION.md` | 54 | "calls socket_emit" | "calls datagram" |

---

### 3. Stale `DatagramKind` enum variants in PLAN_UNIFIED_SHELL.md

**Category: STALE**

| File | Line | Stale Reference | Should Be |
|------|------|-----------------|-----------|
| `PLAN_UNIFIED_SHELL.md` | 185 | `DatagramKind`: Alert, **Report**, Canary, Notify, **Exchange** | `DatagramKind`: Alert, **Quality**, Canary, Notify, **Traffic** |

---

### 4. `"type"` field name used instead of `"kind"` in HLIDSKJALF_DATAGRAM.md

**Category: STALE**

HLIDSKJALF_DATAGRAM.md is the oldest datagram design doc and predates the rename from `type` to `kind`. Several sections use the old field name.

| File | Line | Stale Reference | Should Be |
|------|------|-----------------|-----------|
| `HLIDSKJALF_DATAGRAM.md` | 43 | `\| **type** \| Event class (alert, report, canary, notify) \|` | `\| **kind** \| Content type (alert, quality, canary, notify, traffic) \|` |
| `HLIDSKJALF_DATAGRAM.md` | 71 | `"type": "alert"` (in JSON example) | `"kind": "alert"` |
| `HLIDSKJALF_DATAGRAM.md` | 88 | "generates speech from the type-specific handler" | "generates speech from the kind-specific handler" |
| `HLIDSKJALF_DATAGRAM.md` | 227 | "type filter + priority threshold" | "kind filter + priority threshold" |

---

### 5. `"report"` kind value used instead of `"quality"` in HLIDSKJALF_DATAGRAM.md

**Category: STALE**

| File | Line | Stale Reference | Should Be |
|------|------|-----------------|-----------|
| `HLIDSKJALF_DATAGRAM.md` | 34 | `\| **report** \| Structured results from a scan \|` | `\| **quality** \| Code quality report \|` |
| `HLIDSKJALF_DATAGRAM.md` | 43 | "alert, report, canary, notify" (missing traffic) | "alert, quality, canary, notify, traffic" |

Additionally, the Event Classes table (line 31-36) is missing `traffic` entirely.

---

### 6. `"type": "exchange"` in JSONL_VIEWER_PLAN.md

**Category: STALE**

Double stale: uses old field name `type` and old enum value `exchange`.

| File | Line | Stale Reference | Should Be |
|------|------|-----------------|-----------|
| `JSONL_VIEWER_PLAN.md` | 19 | `"type": "exchange"` | `"kind": "traffic"` |

---

### 7. "hook intercept" terminology in HLIDSKJALF_DATAGRAM.md

**Category: STALE**

Several references to "hook intercepts" as a concept and "hook_intercept_llm_tool" as a source name. The hook intercept concept is from the pre-datagram era. Modern sources are `syn`, `bifrost`, `lockfile_monitor`, `send_alert`, etc.

| File | Line | Stale Reference | Should Be |
|------|------|-----------------|-----------|
| `HLIDSKJALF_DATAGRAM.md` | 26 | "hook intercepts" | "alerts" or "security events" |
| `HLIDSKJALF_DATAGRAM.md` | 33 | "Hook intercepts" as alert examples | "Security violations, denied actions" |
| `HLIDSKJALF_DATAGRAM.md` | 42 | source list includes "hook" | Should list "syn, bifrost, lockfile_monitor, send_alert" |
| `HLIDSKJALF_DATAGRAM.md` | 70 | `"source": "hook_intercept_llm_tool"` | Use a current source name (e.g., "syn") |
| `HLIDSKJALF_DATAGRAM.md` | 89 | "Hook intercepts carry file paths..." | Update to current payload descriptions |

---

### 8. "type enum" in CLAUDE.md recovery sources table

**Category: STALE**

| File | Line | Stale Reference | Should Be |
|------|------|-----------------|-----------|
| `CLAUDE.md` | 72 | "priority levels, type enum" | "priority levels, kind enum" |

---

### 9. "priority/type filtering" in documentation

**Category: STALE**

| File | Line | Stale Reference | Should Be |
|------|------|-----------------|-----------|
| `CONTEXT_MAP.md` | 171 | "priority/type filtering" | "priority/kind filtering" |
| `hlidskjalf/OUTLINE.md` | 14 | "Priority/type filtering" | "Priority/kind filtering" |

---

### 10. "datagram type" phrasing in documentation

**Category: STALE**

Several docs use "datagram type" to mean "datagram kind" in prose. While this is natural English ("type" as a generic noun), it introduces ambiguity given the field was explicitly renamed from `type` to `kind`.

| File | Line | Stale Reference | Should Be |
|------|------|-----------------|-----------|
| `NEURODIVERGENT_MODALITIES.md` | 76 | "from the event type and detail" | "from the event kind and detail" |
| `NEURODIVERGENT_MODALITIES.md` | 181 | "Every datagram type contributes" | "Every datagram kind contributes" |
| `DISPLAY_AND_FILTERING.md` | 266 | "Every datagram type contributes" | "Every datagram kind contributes" |

---

### 11. Stale RUST_BACKEND_AUDIT.md references

**Category: STALE**

The previous RUST_BACKEND_AUDIT.md was written before the datagram rename and contains multiple stale references. These are audit findings about code that no longer exists in that form.

| File | Line | Stale Reference | Issue |
|------|------|-----------------|-------|
| `audit/RUST_BACKEND_AUDIT.md` | 28 | `pub datagram_type: String` | Field is now `kind: DatagramKind` (typed enum, not String) |
| `audit/RUST_BACKEND_AUDIT.md` | 28 | `enum ["alert", "report", "canary", "notify"]` | Enum is now `["alert", "quality", "canary", "notify", "traffic"]` |
| `audit/RUST_BACKEND_AUDIT.md` | 426 | `DatagramType` | Type is now `DatagramKind` |
| `audit/RUST_BACKEND_AUDIT.md` | 225, 227 | `yggdrasil_shared` | Crate is now `common_core` |
| `audit/RUST_BACKEND_AUDIT.md` | 431 | `DatagramType` in yggdrasil_shared | Datagram now lives in nornir's `datagram` crate with `DatagramKind` |

These are historical audit findings and arguably should be left as-is (they describe the state at audit time), but they could mislead an agent that reads them as current state.

---

### 12. `WatchtowerEvent` reference in HLIDSKJALF_DATAGRAM.md

**Category: STALE**

| File | Line | Stale Reference | Issue |
|------|------|-----------------|-------|
| `HLIDSKJALF_DATAGRAM.md` | 93 | `WatchtowerEvent` | Pre-datagram type name. Migration section describes mapping from a type that no longer exists. |

---

### 13. Confirmed CORRECT references

**Category: CORRECT**

These were reviewed and confirmed to be accurate or intentionally preserved:

| Reference | Location | Why It's Correct |
|-----------|----------|------------------|
| `send_datagram --type <kind>` CLI flag | `DATAGRAM_SPECIFICATION.md` lines 141, 147 | The nornir binary `send_datagram` actually uses `--type` as its CLI flag (maps to `kind` in wire format). The flag name is a deliberate choice in the binary. |
| `HookEvent` struct in `hlidskjalf_core/src/lib.rs` | Lines 10-24, 34-61, 63-71 | Intentionally preserved for backward compatibility. Marked as legacy with comments. `From<HookEvent> for Datagram` conversion exists. |
| "gleipnir" in payload schema descriptions | `schemas/payloads/payload.quality.schema.json` | Gleipnir IS a detection library/tool name that appears in quality payloads. It's not a source (it never emits datagrams), but its name correctly appears in `tool` fields of quality report groups. |
| "bifrost traffic intercept pipeline" | `DISPLAY_AND_FILTERING.md` line 9 | Describes what bifrost does (intercepts API traffic), not used as a source name. |
| `hook-event` in `AUDIT_GUIDE.md` lines 64, 113 | N/A | These are in anti-pattern descriptions telling auditors what to look for. They describe the stale name so agents know to flag it. |
| "exchange diffs" in prose | Multiple docs | Used as a descriptive phrase ("API exchange diffs"), not as a DatagramKind enum value. The concept of exchange diffs is real; the kind is `traffic`. |

---

## Risk Assessment

### High Risk (will cause visual bugs)

**Finding 1: `--severity-info` CSS variable.** This is the only true runtime BUG. The variable does not exist in the design token system. All 12 references across 4 Svelte files will resolve to empty/inherited values, causing missing colors on connected status indicators, active voice buttons, normal-priority text coloring, and quality report severity borders.

### Medium Risk (will mislead future agents)

**Findings 2-10: Documentation drift.** These will not cause runtime bugs, but they are the exact kind of stale reference that causes LLMs to write incorrect code. An agent reading PLAN_UNIFIED_SHELL.md will see `pub use socket_emit::{...}` and write imports using the old crate name. An agent reading HLIDSKJALF_DATAGRAM.md will use `"type"` instead of `"kind"` in new datagram code.

### Low Risk (historical records)

**Findings 11-12: Previous audit references.** The RUST_BACKEND_AUDIT.md predates the rename. Its findings are historical and could carry a dated header. The WatchtowerEvent migration table in HLIDSKJALF_DATAGRAM.md is pure history.

---

## Recommended Fix Priority

1. **Immediate:** Fix `--severity-info` to `--severity-success` in all 4 Svelte files (12 replacements). This is a runtime visual bug.
2. **High:** Update CLAUDE.md, PLAN_UNIFIED_SHELL.md, AUDIT_GUIDE.md, and CONTEXT_MAP.md -- these are the most frequently read files and will propagate stale terminology into agent behavior.
3. **Medium:** Update HLIDSKJALF_DATAGRAM.md, DISPLAY_AND_FILTERING.md, DATAGRAM_SPECIFICATION.md, NEURODIVERGENT_MODALITIES.md, YGGDRASIL_FEATURE_REQUESTS.md, JSONL_VIEWER_PLAN.md, hlidskjalf/OUTLINE.md.
4. **Medium:** Update `schemas/datagram.schema.json` and `schemas/validate.datagram.schema.json` source field descriptions.
5. **Low:** Consider adding a dated header to RUST_BACKEND_AUDIT.md noting it describes pre-rename state.
