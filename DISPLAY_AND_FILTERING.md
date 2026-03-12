# Hlidskjalf Display and Filtering Design

## Purpose

Design specification for how Hlidskjalf renders its datagram feed — the scrolling stream of events from all active sessions. Covers the normative datagram contract, the rendering system, specialized renderers for complex payloads, the filter bar, and session management.

## Context

Hlidskjalf receives datagrams via UDP multicast on 239.0.0.1:9899. Sources include the syn quality checker, the bifrost traffic intercept pipeline, lockfile monitors, and general alerts/notifications. The feed is always visible while working — a scrolling stream read by pattern, not by individual entry. The rendering system must show signal without intervention: steady rhythm = work happening, visual break = attention needed.

---

## Normative Datagram Contract

### Design Principle: Pattern B

The envelope is self-describing. `kind` tells you exactly what you're holding — you never read the payload to determine the datagram's identity. `kind` drives both rendering (which component draws it) and filtering (which chips toggle it).

### Envelope Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `timestamp` | f64 | yes | Unix timestamp (displayed as local HH:MM:SS) |
| `kind` | string enum | yes | Content type — what this datagram IS |
| `classifier` | string enum | conditional | Sub-classification within the kind. Required for traffic and quality. Closed enum per kind. |
| `source` | string | yes | Who sent it — the binary with the datagram call (e.g. "syn", "bifrost", "lockfile_monitor") |
| `priority` | string enum | yes | Severity: `"trace"`, `"low"`, `"normal"`, `"high"`, `"critical"` |
| `workspace` | string | yes | Project/workspace name |
| `detail` | string | no | Human-readable one-line summary |
| `speech` | string | no | Voice alert text (triggers TTS when priority exceeds speech threshold) |
| `payload` | object | no | Structured data — shape determined by `kind` |

### Classifier Enums

| Kind | Classifier values | Description |
|------|-------------------|-------------|
| `traffic` | `startup`, `conversation`, `tool`, `subagent`, `planning` | API round-trip sub-type |
| `quality` | `directory`, `single_file` | Multi-file scan vs single-file check |
| `alert` | — | No classifier (for now) |
| `canary` | — | No classifier |
| `notify` | — | No classifier |

### Kind Enum

Each kind is a specific, self-describing identity. No categories — every kind gets its own filter chip.

| Kind | Source | Payload? | Payload Schema | Description |
|------|--------|----------|----------------|-------------|
| `alert` | `send_alert`, `lockfile_monitor` | no | — | Operational alerts, kill switch, keep-alive failures |
| `canary` | heartbeat senders | no | — | Presence signal — session is alive |
| `notify` | general notifications | no | — | Informational messages |
| `quality` | `syn` | yes | `payload.quality.schema.json` | Code quality report — single-file or multi-file. Emitted by the syn binary (the process with the datagram call). Gleipnir is a detection library inside syn's pipeline — it never emits datagrams. |
| `traffic` | `bifrost` | yes | `payload.traffic.schema.json` | API round-trip: user/assistant/tools/system |

`kind` names what the datagram IS (content type), not who produced it. `source` names who SENT it (the binary). The enum is extensible — new producers add new kinds, each gets its own rendering and filtering without modifying existing kinds.

---

## Traffic Datagrams

**Schema:** `schemas/payloads/payload.traffic.schema.json` (symlink → nornir)

### Envelope

| Field | Value |
|-------|-------|
| `kind` | `"traffic"` |
| `source` | `"bifrost"` |
| `priority` | `"low"` for most exchanges, `"normal"` when system injections detected |
| `workspace` | Workspace name from session resolution (e.g. "odinn", "yggdrasil") |
| `payload` | Flat object with content fields (see below) |

Note: `detail` and `speech` are not currently set on exchange datagrams — the payload carries all the information.

### Payload Structure

The payload is **flat** — no nested `dimensions` object. Each field is present only when it has content (sparse). The schema is `traffic.schema.json` with `additionalProperties: false`.

**Always present:**

| Field | Type | Description |
|-------|------|-------------|
| `traffic_kind` | enum | `"startup"`, `"conversation"`, or `"tool"` (also `"subagent"`, `"planning"` per schema) |
| `source` | string | Reference to raw exchange: `"mainexch_{uuid}.jsonl:{line}"` |
| `system_injection` | boolean | True when platform-injected system-reminders are present |

**Content fields (present when non-empty):**

| Field | Type | Frequency | Description |
|-------|------|-----------|-------------|
| `user` | string | 34% | User-authored text content |
| `assistant` | string | 62% | Assistant response text |
| `thinking` | string | 25% | Chain-of-thought reasoning |
| `system` | string | 7% | System-reminder blocks (can be very large: up to 60KB) |
| `instructions` | string | <1% | New system prompt blocks (CLAUDE.md additions) |
| `tool_return` | string | 79% | Tool result content (can be very large: up to 127KB) |

**Tool invocation fields (present when tools used, single object or array):**

| Field | Type | Frequency | Object shape |
|-------|------|-----------|--------------|
| `shell` | tool_input[] | 48% | `{ command, description }` |
| `file_edit` | tool_input[] | 24% | `{ file_path, old_string, new_string, replace_all }` |
| `file_read` | tool_input[] | 6% | `{ file_path }` |
| `file_write` | tool_input[] | 1% | `{ file_path, content }` |
| `file_search` | tool_input[] | <1% | `{ pattern, path? }` |
| `text_search` | tool_input[] | 2% | `{ pattern, path?, output_mode? }` |
| `web_search` | tool_input[] | 0% | `{ query }` |
| `web_fetch` | tool_input[] | 0% | `{ url, prompt }` |
| `agent_dispatch` | tool_input[] | 0% | Task tool inputs |
| `ask_question` | tool_input[] | 0% | AskUserQuestion inputs |
| `start_plan` | tool_input[] | 0% | EnterPlanMode inputs |
| `finish_plan` | tool_input[] | 0% | ExitPlanMode inputs |
| `tool_use` | tool_input[] | 1% | Generic/unmapped tools (e.g. `{ taskId, status }`) |
| `tools_added` | string[] | 0% | Names of newly available tool definitions |

### Traffic Sub-Kind Distribution (real data, 234 datagrams from one session)

| Kind | Count | % | Typical content |
|------|-------|---|-----------------|
| `tool` | 153 | 65% | Tool invocations + results, usually no user text |
| `conversation` | 80 | 34% | User message + assistant response |
| `startup` | 1 | <1% | Session start — large system prompt, many tools |

### Field Size Reality

The payload can be heavy. Key fields by size:

| Field | Median | Max | Notes |
|-------|--------|-----|-------|
| `tool_return` | 190 chars | 127KB | Dominates payload size |
| `system` | 1.6KB | 61KB | System-reminders from platform injections |
| `file_edit` | 1.8KB | 86KB | Full old_string/new_string diffs |
| `user` | 113 chars | 22KB | Usually short, large when pasting context |
| `assistant` | 137 chars | 18KB | Varies widely |
| `thinking` | 704 chars | 29KB | When present, substantial |

### Priority Determination (current)

| Condition | Priority | Rationale |
|-----------|----------|-----------|
| `system_injection: true` | `normal` | Platform injected system-reminders — instructions changed |
| Everything else | `low` | Standard exchange activity |

**Future priority escalation** (not yet implemented in bifrost):

| Condition | Priority | Rationale |
|-----------|----------|-----------|
| `instructions` field present | `high` | New CLAUDE.md or workspace instructions loaded |
| `tools_added` field present | `normal` | Tool definitions changed |
| `traffic_kind: "startup"` | `normal` | Session start/resume/post-compaction |
| Compaction detected | `critical` | Context compressed — all prior context replaced |

---

## Feed Rendering

### Layered Disclosure

The rendering hierarchy surfaces the most important signal first. Everything else is available on demand.

**System changes — visible immediately.** New system block text is displayed directly in the feed row. If the text exceeds a threshold (e.g. 3 lines or 200 characters), it truncates with a visual indicator. Click the text to toggle between truncated and full view.

**Tool changes — summary line.** A compact summary: "tools updated — 1 added, 1 modified." Click to expand and see which tools, with full description diffs available on further expansion.

**Messages — collapsed row.** A "messages" row shows turn count but content is hidden. Click to expand and read the interchange. This serves as the context anchor when multiple sessions are running — you re-read the interchange to understand what was happening when a system prompt changed.

**Compaction — critical alert rendering.** Distinct from regular exchange diffs. Full-width banner treatment (similar to QualityReport). Shows that context was compressed, the workspace affected, and the timestamp. The compaction summary content (if captured in the response) would be the primary display.

### Message-Only Traffic

Traffic datagrams where only messages changed (no system or tool deltas) arrive as `priority: "low"`. They are:

- Logged to the rolling event log with everything else
- Hidden in the feed by default (the priority minimum filter excludes them)
- Visible when the user drops the priority threshold to "low" or "trace"
- Rendered like canaries — compact, dimmed, minimal visual weight

These carry no actionable information but confirm session activity. They contribute to ambient awareness (chip pulsing, geiger ticking) without cluttering the feed.

Note: `traffic_kind` in the payload is the same sub-classification as `classifier` in the envelope. The envelope `classifier` drives rendering; the payload `traffic_kind` remains for structured access within the payload.

---

## Filter Bar

### Layout

Two horizontal zones stacked vertically below the header.

**Zone 1: Event Controls**

```
[all] [alert] [quality] [traffic] [canary] [notify]  |  min: [normal ▾]  ···  [🔔 monitoring ▾]  [auto-scroll ✓]  [clear]
```

- **Kind filter chips** — click to show only that kind. "all" shows everything. Chips appear dynamically as kinds are seen.
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

1. **Kind** — which datagram kinds to show (per-kind granularity)
2. **Priority minimum** — what severity threshold to display
3. **Session visibility** — which workspace sessions to include

Example combinations:
- "Show me everything from bragi" → kind=all, min=trace, only bragi active
- "Show me only traffic across all sessions" → kind=traffic, min=low, all sessions active
- "Show me quality reports and alerts, ignore noise" → alert+quality kinds, min=normal, all sessions active
- "What's happening in phoenix right now" → only phoenix active, kind=all, min=normal

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

The pulse rate corresponds to datagram arrival rate from that session. Every datagram kind contributes: tool use events, exchange diffs, alerts, reports. Canaries confirm the session is alive but don't contribute to pulse rate.

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

---

## Open Questions

- **Payload detail level**: How much of the system block text to include in the datagram vs. available on-demand from the JSONL files? Large system blocks could make datagrams heavy.
- **Compaction response display**: Should Hlidskjalf display the compaction summary that came back from the API (the response), not just the fact that compaction happened? This requires capturing the response in Bifrost and including it in the datagram or a follow-up datagram.
- **Exchange diff timing**: Does Bifrost emit the datagram immediately when it sees the second exchange of a pair, or does it batch/debounce?
- **Session end detection**: How does Hlidskjalf know a session has ended vs. gone quiet? Timeout-based dimming, or an explicit "session ended" datagram from Bifrost?
