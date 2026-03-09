# Hlidskjalf Display and Filtering Design

## Purpose

Design specification for how Hlidskjalf displays exchange-pair diffs from Bifrost and provides intelligent filtering across concurrent sessions. This covers the feed rendering, the filter bar, session management, and the interaction model.

## Context

Bifrost intercepts Claude Code API traffic and diffs consecutive main conversation exchanges. Each diff produces a datagram sent to Hlidskjalf via the Unix socket. The diff has three dimensions: messages, system blocks, and tools. Hlidskjalf must display these diffs alongside existing event types (alerts, reports, canaries, notifications) while letting the user manage noise from multiple concurrent sessions.

---

## Exchange Diff Datagrams

### Priority Determination

The datagram priority is set by the **highest-priority dimension that changed**:

| What changed | Priority | Rationale |
|-------------|----------|-----------|
| Messages only (no system or tool changes) | low | Context anchor, no actionable signal |
| Tool definitions added/removed/modified | normal | Operational awareness — available capabilities changed |
| System prompt content added/modified | high | The instructions governing the session changed |
| Compaction detected | critical | Everything dropped, new summary replaces all context |

A single datagram may have changes in multiple dimensions. The priority reflects the most significant one.

### Datagram Fields

| Field | Value |
|-------|-------|
| `type` | `"exchange"` |
| `source` | `"bifrost"` |
| `priority` | Determined by highest-priority dimension (see above) |
| `workspace` | Workspace name from Bifrost session resolution |
| `detail` | Human-readable summary, e.g. "system prompt updated, 2 tools added" |
| `speech` | Generated for high+ priority, absent for low/normal |
| `payload` | Structured diff data (see below) |

### Payload Structure

```json
{
  "type": "exchange_diff",
  "session_id": "uuid",
  "exchange_index": 42,
  "is_compaction": false,
  "dimensions": {
    "messages": {
      "changed": true,
      "turns_added": 3,
      "summary": "user asked about X, assistant responded with Y"
    },
    "system": {
      "changed": true,
      "blocks_added": ["full text of new system block..."],
      "blocks_removed": [],
      "blocks_modified": [
        {
          "before_snippet": "first 200 chars...",
          "after_snippet": "first 200 chars...",
          "full_before": "...",
          "full_after": "..."
        }
      ]
    },
    "tools": {
      "changed": true,
      "added": ["tool_name_1"],
      "removed": [],
      "modified": ["tool_name_2"]
    }
  }
}
```

Payload details will evolve as we see real diffs. This is a starting shape.

---

## Feed Rendering

### Layered Disclosure

The rendering hierarchy surfaces the most important signal first. Everything else is available on demand.

**System changes — visible immediately.** New system block text is displayed directly in the feed row. If the text exceeds a threshold (e.g. 3 lines or 200 characters), it truncates with a visual indicator. Click the text to toggle between truncated and full view.

**Tool changes — summary line.** A compact summary: "tools updated — 1 added, 1 modified." Click to expand and see which tools, with full description diffs available on further expansion.

**Messages — collapsed row.** A "messages" row shows turn count but content is hidden. Click to expand and read the interchange. This serves as the context anchor when multiple sessions are running — you re-read the interchange to understand what was happening when a system prompt changed.

**Compaction — critical alert rendering.** Distinct from regular exchange diffs. Full-width banner treatment (similar to GleipnirReport). Shows that context was compressed, the workspace affected, and the timestamp. The compaction summary content (if captured in the response) would be the primary display.

### Message-Only Exchanges

Exchange diffs where only messages changed (no system or tool deltas) arrive as `priority: "low"`. They are:

- Logged to the rolling event log with everything else
- Hidden in the feed by default (the priority minimum filter excludes them)
- Visible when the user drops the priority threshold to "low" or "trace"
- Rendered like canaries — compact, dimmed, minimal visual weight

These carry no actionable information but confirm session activity. They contribute to ambient awareness (chip pulsing, geiger ticking) without cluttering the feed.

---

## Filter Bar

### Layout

Two horizontal zones stacked vertically below the header.

**Zone 1: Event Controls**

```
[all] [alert] [report] [exchange] [canary] [notify]  |  min: [normal ▾]  ···  [🔔 monitoring ▾]  [auto-scroll ✓]  [clear]
```

- **Type filter chips** — click to show only that type. "all" shows everything. Chips appear dynamically as event types are seen (same as current behavior, with "exchange" added).
- **Priority minimum** — dropdown or cycle control. Sets the minimum severity for events to appear in the feed. Default: "normal" (hides low-priority message-only exchanges and canaries without special logic).
- **Alert profile selector** — bell icon with current profile name. Click for popover with profile switching and customization (see NEURODIVERGENT_MODALITIES.md).
- **Auto-scroll** — checkbox. When on, feed scrolls to bottom on new events.
- **Clear** — clears the feed display (events remain in the log).

**Zone 2: Session Strip**

```
[● bragi] [● phoenix] [● yggdrasil a] [● yggdrasil b]  [reset]
```

- Appears once the first session-tagged datagram arrives.
- One chip per active session, labeled by workspace name.
- If multiple sessions share a workspace: "bragi a", "bragi b".
- Chips are color-coded by workspace color family (see below).
- Chips are living activity indicators (see "Workspace Chip States" below).

### How Filters Interact

The three filter dimensions AND together:

1. **Type** — which event categories to show
2. **Priority minimum** — what severity threshold to display
3. **Session visibility** — which workspace sessions to include

Example combinations:
- "Show me everything from bragi" → type=all, min=trace, only bragi active
- "Show me only system prompt changes across all sessions" → type=exchange, min=high, all sessions active
- "Show me alerts and reports, ignore noise" → alert+report types, min=normal, all sessions active
- "What's happening in phoenix right now" → only phoenix active, type=all, min=normal

---

## Session Management

### Workspace Naming

Sessions display by **workspace name** (bragi, phoenix, yggdrasil), not by session UUID. The workspace name is how the user thinks about their projects. UUIDs are internal plumbing.

Multiple sessions in the same workspace (rare but possible) get alphabetic suffixes: "bragi a", "bragi b". Suffixes appear only when disambiguation is needed — a solo session in a workspace is just "bragi".

### Color Families

Each workspace is assigned a color family. Sessions within the same workspace get shades from the same family — visually related but distinct.

| Family | Shades | Assignment |
|--------|--------|------------|
| Blue | blue, turquoise, cyan | First workspace seen |
| Warm | yellow, amber, orange | Second workspace seen |
| Red | coral, red, crimson | Third workspace seen |
| Purple | navy, violet, purple | Fourth workspace seen |

Assignment is stable within a Hlidskjalf session (first-seen order). The session color appears:
- As the chip background/border in the session strip
- As a left border or color accent on feed rows from that session
- As the LED dot color in the header

### Workspace Chip States

Chips are not static toggles. They are **living activity indicators**:

| State | Visual | Meaning |
|-------|--------|---------|
| **Busy** | Pulsing/breathing in session color, rate proportional to activity | Events flowing — tool uses, exchanges, reports |
| **Quiet** | Solid color, still | Session exists but nothing happening right now |
| **Critical** | Brief bright flash or glow | A critical event just fired from this session |
| **Inactive** | Dimmed | No activity for a sustained period |

The pulse rate corresponds to datagram arrival rate from that session. Every datagram type contributes: tool use events, exchange diffs, alerts, reports. Canaries confirm the session is alive but don't contribute to pulse rate.

### Chip Interaction

- **Click a chip** → toggle that session's visibility in the feed (on/off)
- **Active** = filled background in session color, white text
- **Inactive** = transparent background, outlined, dimmed text. Events from that session disappear from the feed but remain in the log.
- **Chips persist** when a session goes quiet. They dim but remain clickable — you might want to review a finished session's events.
- **Reset button** → turns all sessions ON. One click to restore full visibility after selectively toggling sessions off.

### Feed Row Coloring

Each event row in the feed carries a visual indicator of its session:
- A colored left border (2-3px) in the session's color
- Or the workspace label text colored in the session's color

This lets you visually scan the feed and follow one session's narrative by color without reading workspace names. With the color families, same-workspace sessions look related (both blue-ish) while different workspaces are visually distinct (blue vs warm vs red).

---

## New Datagram Type

The `type` enum in the datagram schema needs a new value: `"exchange"`.

Updated enum: `["alert", "report", "canary", "notify", "exchange"]`

This requires updating:
- `schemas/datagram.schema.json` — add "exchange" to the type enum
- nornir `socket_emit` — add `Exchange` variant to `DatagramKind` enum (already present)
- `HlidskjalfView.svelte` — add "exchange" to `typeIcon` mapping, add renderer for exchange_diff payloads
- New component: `ExchangeDiffReport.svelte` — renderer for exchange_diff payloads (parallel to GleipnirReport.svelte)

---

## Open Questions

- **Payload detail level**: How much of the system block text to include in the datagram vs. available on-demand from the JSONL files? Large system blocks could make datagrams heavy.
- **Compaction response display**: Should Hlidskjalf display the compaction summary that came back from the API (the response), not just the fact that compaction happened? This requires capturing the response in Bifrost and including it in the datagram or a follow-up datagram.
- **Exchange diff timing**: Does Bifrost emit the datagram immediately when it sees the second exchange of a pair, or does it batch/debounce?
- **Session end detection**: How does Hlidskjalf know a session has ended vs. gone quiet? Timeout-based dimming, or an explicit "session ended" datagram from Bifrost?
