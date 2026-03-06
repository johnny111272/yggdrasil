# Hlidskjalf Datagram Protocol

## Purpose

Standardize the messaging format for all events flowing through Hlidskjalf's Unix socket (`/tmp/hlidskjalf.sock`). Every component in the system — hooks, syn, subagents, canaries, manual triggers — speaks the same protocol. Hlidskjalf receives everything, stores everything, and controls what gets displayed and spoken.

## Vision: General AI Work State Machine

Hlidskjalf is not a monitor for one Claude session. It is the central nervous system across all concurrent AI work — multiple agents, multiple subagents, multiple workspaces, all reporting into one place. It doesn't matter where the agents are working or how many there are. Hlidskjalf acts as a dashboard and control panel on the aggregate.

**Fully decoupled architecture.** Hlidskjalf is a radio receiver for LLM messages. It listens on a Unix socket, displays what arrives, and logs everything. Its control mechanisms — lockfiles on disk — are not Hlidskjalf integrations. They are plain Unix files that anyone can read, write, or remove:

- Hlidskjalf can touch `SYN.lock` to trigger a full scan
- You can touch it from the terminal to do the same thing
- A cron job can touch it on a schedule
- A monitor agent can touch it based on its own logic
- A script can remove `KEEP_ALIVE.lock` to trip the kill switch

The control plane belongs to nobody. Hlidskjalf is the most convenient dashboard for seeing what's happening and flipping the switches, but it is never the only way. Every control action reduces to creating or removing a file on disk — the simplest, most inspectable, most language-agnostic mechanism possible.

## Core Principles

1. **Never filter input, only filter display.** Every datagram arrives and is stored. Visibility and speech are controlled at the receiver, never at the sender.
2. **Persistence.** Rolling `.jsonl` log retains at least 24 hours of events for pattern analysis and replay.
3. **Sender knows its message.** Events with known formats include their own speech text and display data. Hlidskjalf doesn't need to understand every event type — it just needs to classify and route.
4. **Known types get handlers.** Syn reports, hook intercepts, and other well-defined formats get dedicated rendering in the UI. Unknown types get generic display based on classification fields.
5. **Lockfile triggers.** System-level behaviors (kill switch, forced scans) are controlled through lockfiles on disk — simple, inspectable, language-agnostic.

## Event Classes

| Class | Purpose | Examples |
|-------|---------|----------|
| **alert** | Something happened that needs attention | Hook intercepts, security violations, errors |
| **report** | Structured results from a scan or process | Syn reports, saga outcomes |
| **canary** | Proof-of-life from infrastructure components | System prompt section active, hook alive, settings intact |
| **notify** | Status updates from automated processes | Subagent finished, build complete, task progress |

## Classification Dimensions

| Field | Purpose |
|-------|---------|
| **source** | Which component sent it (hook, syn, subagent, canary, saga, manual) |
| **type** | Event class (alert, report, canary, notify) |
| **priority** | Threshold level for display and speech gating |
| **workspace** | Which project context produced the event |

## Priority Levels

_To be determined through pattern analysis of the rolling log. Initial candidates:_

| Level | Semantics |
|-------|-----------|
| **critical** | System integrity threat — kill switch territory |
| **high** | Immediate attention — deny events, blocked actions |
| **normal** | Standard notifications — warns, scan results |
| **low** | Background information — canary pulses, progress ticks |
| **trace** | Debug-level — individual file scans, routine events |

Speech threshold and display filters operate on these levels. Setting speech to "high" means only critical and high events get spoken.

## Datagram Format

All datagrams are JSON objects, one per line (newline-delimited), sent over the Unix stream socket.

### Required Fields

```json
{
  "timestamp": 1709712000.0,
  "source": "hook_intercept_llm_tool",
  "type": "alert",
  "priority": "high",
  "workspace": "bragi"
}
```

### Optional Fields

```json
{
  "detail": "Human-readable summary of what happened",
  "speech": "Text to speak via macOS say (overrides generated speech)",
  "payload": { ... }
}
```

- **detail** — Short human-readable description for the event feed
- **speech** — If set, Hlidskjalf speaks this text directly (gated by priority threshold). If absent, Hlidskjalf generates speech from the type-specific handler or a generic template
- **payload** — Type-specific structured data. Syn reports carry their full groups here. Hook intercepts carry file paths, categories, decisions

### Migration from Current Format

Current `WatchtowerEvent` fields map to the new format:

| Current | New |
|---------|-----|
| `category` | Moves into `payload` or `detail` depending on context |
| `decision` | Moves into `payload` (hook-specific) |
| `event_name` | Replaced by `source` + type-specific payload |
| `context_injected` | Moves into `payload` (hook-specific) |
| `speech` | Stays as `speech` |

## Persistence — Rolling Log

All datagrams are appended to a `.jsonl` file:

```
~/.ai/hlidskjalf/events.jsonl
```

- One JSON object per line (same as socket format)
- Rotated or truncated to retain ~24 hours of events
- Source data for pattern analysis, replay, and automated triggers

## Lockfile System

### Kill Switch

Two lockfiles, both conditions must be satisfied:

| File | Required State | Meaning |
|------|---------------|---------|
| `~/.ai/hlidskjalf/KEEP_ALIVE.lock` | Must exist | System is running normally |
| `~/.ai/hlidskjalf/KILL.lock` | Must not exist | No emergency shutdown active |

**Trigger conditions:**
- Manual: You remove `KEEP_ALIVE.lock` or create `KILL.lock`
- Automatic: Hlidskjalf detects alarming patterns in the log and creates `KILL.lock`

**Effect:** All hooks switch to BLOCK EVERYTHING mode. Nothing gets through until the lockfiles are restored to normal state.

### Forced Scan Trigger

```
{workspace}/.ai/SYN.lock
```

- Hlidskjalf monitors the log for scan events and triggers a full scan when either condition is met:
  - **Count-based:** N single-file scans since last full scan
  - **Time-based:** M minutes elapsed since last full scan
- Whichever threshold hits first, Hlidskjalf touches `SYN.lock` in the workspace
- The save hook sees `SYN.lock`, triggers a full workspace scan instead of single-file, then removes the lockfile
- You can also create `SYN.lock` manually to force an immediate full scan

## Display and Speech

### Display Filtering

Hlidskjalf UI provides toggles for:
- Source (show/hide per component)
- Type (show/hide per event class)
- Priority threshold (show events at or above selected level)
- Workspace (filter by project)

Filters affect visibility only — all events remain in the log.

### Speech Gating

Speech threshold is a priority level. Events at or above the threshold with either:
- An explicit `speech` field — spoken directly
- A known type with a format handler — speech generated from payload
- An unknown type above threshold — generic template: "{source} {type} in {workspace}"

### Canary Display

Canaries are not shown in the main event feed by default. They appear as:
- A row of indicator dots/icons in a status bar
- Each canary source gets a dot
- Active = pulsing/lit, stale = dimmed, dead = highlighted in red
- Staleness threshold configurable per canary source

## Context Injection

Separate from Hlidskjalf display. When a full syn scan completes:
- Results display in Hlidskjalf as a report
- Results are also injected into the LLM context (via hook mechanism)
- Injection cadence controlled by the `SYN.lock` system, not by Claude

This ensures Claude receives workspace-wide quality data at a controlled cadence rather than either drowning in per-file data or never seeing the full picture.

## Sender Binaries

Separate compiled binaries per severity level, deployed to `~/.ai/tools/bin/`. No flags for priority — the binary name IS the severity. This prevents LLMs and subagents from "helpfully" escalating their own messages.

### Fixed-severity senders

| Binary | Type | Priority | Args |
|--------|------|----------|------|
| **`send_heartbeat`** | canary | low | `<source>` |
| **`send_notification`** | notify | normal | `<source> <message>` |
| **`send_warning`** | alert | high | `<source> <message>` |
| **`send_alert`** | alert | critical | `<source> <message>` |

Each binary builds a proper datagram internally — timestamp, workspace detection, JSON serialization, socket send. The caller provides only source and message.

### Full-protocol sender

| Binary | Purpose |
|--------|---------|
| **`send_datagram`** | All fields exposed as flags. For trusted internal tooling only — not given to LLMs or subagents. |

### Hook permissions

Fixed-severity senders can be whitelisted in Claude Code hook config as always-allowed. No permission prompts, no friction. The binary name determines the ceiling — an LLM given `send_notification` cannot escalate to `send_alert` because it doesn't have that binary.

```json
{
  "hooks": {
    "PreToolUse:Bash": [{
      "allow": ["send_heartbeat", "send_notification"]
    }]
  }
}
```

Subagents get a restricted set. Main LLM sessions may get more. You control the envelope per context.

## Implementation Order

_Rough sequencing, not commitments:_

1. Standardize datagram format — update `socket_emit::WatchtowerEvent` and `hlidskjalf_core::HookEvent`
2. Rolling `.jsonl` log — write on receive in hlidskjalf_core
3. Sender binaries — `send_heartbeat`, `send_notification`, `send_warning`, `send_alert`, `send_datagram`
4. Kill switch — lockfile checks in hook_io
5. Syn escalation — count in log, touch `SYN.lock`
6. Canary infrastructure — emit from system prompt sections, display in status bar
7. Display filtering UI — source/type/priority toggles
8. Priority level refinement — analyze log patterns, adjust levels
