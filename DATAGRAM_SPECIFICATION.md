# Datagram Specification

Technical specification for the Hlidskjalf datagram format, JSON schema, and sender tool interfaces. This document is the build reference — see `HLIDSKJALF_DATAGRAM.md` for design rationale and architecture.

## Transport

- **Socket:** Unix stream socket at `/tmp/hlidskjalf.sock`
- **Framing:** Newline-delimited JSON (one complete JSON object per line, terminated by `\n`)
- **Encoding:** UTF-8
- **Connection model:** Connect, write, close. No persistent connections. Fire-and-forget — sender does not wait for acknowledgement.
- **Failure mode:** If the socket is unavailable, the send silently fails. Senders never block or retry.

## Datagram Schema

### Field Reference

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `timestamp` | number | yes | Unix epoch, float64, seconds | When the event occurred |
| `source` | string | yes | 1–64 chars, `[a-z0-9_]` only | Identifies the sending component |
| `type` | string | yes | enum: see below | Event class |
| `priority` | string | yes | enum: see below | Severity/importance level |
| `workspace` | string | yes | 0–128 chars | Project context (empty string if not workspace-scoped) |
| `detail` | string | no | 0–512 chars | Human-readable summary for the event feed |
| `speech` | string | no | 0–256 chars | Text for macOS `say`. If absent, receiver may generate from type handler |
| `payload` | object | no | Max 64KB serialized | Type-specific structured data |

### `type` Enum

| Value | Semantics |
|-------|-----------|
| `alert` | Something requires attention — security events, errors, denied actions |
| `report` | Structured results from a scan or process — syn reports, saga outcomes |
| `canary` | Proof-of-life pulse from an infrastructure component |
| `notify` | Informational status update — task progress, completion, state changes |

### `priority` Enum

Ordered from highest to lowest. Speech and display thresholds compare against this ordering.

| Value | Numeric | Semantics |
|-------|---------|-----------|
| `critical` | 4 | System integrity threat. Kill switch territory. |
| `high` | 3 | Immediate attention. Denied actions, security violations. |
| `normal` | 2 | Standard events. Warnings, scan results, notifications. |
| `low` | 1 | Background. Canary pulses, routine progress. |
| `trace` | 0 | Debug-level. Individual file scans, verbose state. |

### `source` Conventions

Source names identify the component, not the instance. Use snake_case, no dots or slashes.

| Source | Component |
|--------|-----------|
| `hook_intercept_llm_tool` | PreToolUse hook for LLM file access |
| `hook_intercept_llm_bash` | PreToolUse hook for LLM shell commands |
| `hook_intercept_subagent_tool` | PreToolUse hook for subagent file access |
| `hook_intercept_subagent_bash` | PreToolUse hook for subagent shell commands |
| `syn` | Code quality scanner |
| `saga` | Build/check orchestrator |
| `canary_prompt` | System prompt section heartbeat |
| `canary_hook` | Hook infrastructure heartbeat |
| `subagent` | Subagent status updates |
| `user` | Manual sends from the terminal |

New sources may be added freely — the field is not a closed enum. The conventions above are for consistency.

## JSON Schema

File: `schemas/datagram.schema.json`

Embedded at compile time in all sender binaries via `include_str!()`. Validated before socket write.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "datagram-schema-v1",
  "title": "Hlidskjalf Datagram",
  "description": "Event datagram for the Hlidskjalf messaging protocol.",
  "type": "object",
  "required": ["timestamp", "source", "type", "priority", "workspace"],
  "additionalProperties": false,
  "properties": {
    "timestamp": {
      "type": "number",
      "description": "Unix epoch timestamp in seconds (float64)."
    },
    "source": {
      "type": "string",
      "pattern": "^[a-z0-9_]{1,64}$",
      "description": "Sending component identifier."
    },
    "type": {
      "type": "string",
      "enum": ["alert", "report", "canary", "notify"],
      "description": "Event class."
    },
    "priority": {
      "type": "string",
      "enum": ["critical", "high", "normal", "low", "trace"],
      "description": "Severity level for threshold filtering."
    },
    "workspace": {
      "type": "string",
      "maxLength": 128,
      "description": "Project context. Empty string if not workspace-scoped."
    },
    "detail": {
      "type": "string",
      "maxLength": 512,
      "description": "Human-readable event summary."
    },
    "speech": {
      "type": "string",
      "maxLength": 256,
      "description": "Text for spoken voice alert. Omit to let receiver generate or skip."
    },
    "payload": {
      "type": "object",
      "description": "Type-specific structured data. Schema varies by type+source."
    }
  }
}
```

## Sender Binaries

All built as nornir crates. Schema embedded at compile time. All senders validate the datagram against the schema before sending. Invalid datagrams are rejected at the sender — they never reach the socket.

### Fixed-Severity Senders

These binaries have their `type` and `priority` hardcoded. No flags to override. The binary name is the severity contract.

#### `send_heartbeat`

```
send_heartbeat <source>
```

| Field | Value |
|-------|-------|
| `type` | `canary` |
| `priority` | `low` |
| `source` | from arg |
| `workspace` | auto-detected from `$CLAUDE_PROJECT_DIR` |
| `detail` | none |
| `speech` | none |
| `payload` | none |

Exit: always 0. Fire-and-forget.

#### `send_notification`

```
send_notification <source> <message>
```

| Field | Value |
|-------|-------|
| `type` | `notify` |
| `priority` | `normal` |
| `source` | from arg |
| `workspace` | auto-detected |
| `detail` | from `<message>` arg |
| `speech` | none |
| `payload` | none |

#### `send_warning`

```
send_warning <source> <message>
```

| Field | Value |
|-------|-------|
| `type` | `alert` |
| `priority` | `high` |
| `source` | from arg |
| `workspace` | auto-detected |
| `detail` | from `<message>` arg |
| `speech` | auto-generated: `"Warning from {source}: {message}"` |
| `payload` | none |

#### `send_alert`

```
send_alert <source> <message>
```

| Field | Value |
|-------|-------|
| `type` | `alert` |
| `priority` | `critical` |
| `source` | from arg |
| `workspace` | auto-detected |
| `detail` | from `<message>` arg |
| `speech` | auto-generated: `"ALERT from {source}: {message}"` |
| `payload` | none |

### Full-Protocol Sender

#### `send_datagram`

```
send_datagram --source <source> --type <type> --priority <priority> [options]
```

| Flag | Required | Description |
|------|----------|-------------|
| `--source` | yes | Source identifier |
| `--type` | yes | Event type (alert, report, canary, notify) |
| `--priority` | yes | Priority level |
| `--workspace` | no | Override auto-detected workspace |
| `--detail` | no | Human-readable summary |
| `--speech` | no | Speech text |
| `--payload-file` | no | Path to JSON file for payload |
| `--payload` | no | Inline JSON string for payload |

Not deployed to LLM-accessible paths. For trusted tooling only.

## Payload Schemas

Payloads are type-specific. Each known source+type combination may define its own payload schema. Unknown payloads are passed through without validation — only the envelope is validated.

### Hook Intercept Payload

Source: `hook_intercept_*`, type: `alert`

```json
{
  "category": "probing",
  "decision": "warn",
  "tool": "Read",
  "target": "/path/to/file",
  "context_injected": "Additional context sent to the LLM..."
}
```

### Syn Report Payload

Source: `syn`, type: `report`

Identical to syn's `--output json` format:

```json
{
  "type": "syn_report",
  "total": 17,
  "check_types": 5,
  "groups": [ ... ]
}
```

Hlidskjalf has a dedicated renderer for this payload type (the existing `GleipnirReport.svelte` component).

## Receiver Behavior

### On datagram arrival

1. Parse JSON line
2. Validate envelope against `datagram.schema.json` — drop malformed datagrams (log parse error)
3. Append raw JSON line to rolling log (`~/.ai/hlidskjalf/events.jsonl`)
4. Emit as Tauri event to frontend
5. Frontend applies display filters (source, type, priority threshold)
6. Frontend applies speech logic:
   - If `speech` field is present and priority >= speech threshold → speak it
   - If `speech` absent and a type handler exists → handler generates speech from payload
   - If `speech` absent and no handler → generic template or silent

### Rolling Log

- Location: `~/.ai/hlidskjalf/events.jsonl`
- Rotation: On startup, or when file exceeds size limit, truncate to retain last 24 hours
- Format: Identical to socket format — one JSON object per line
- Purpose: Pattern analysis, replay, trigger counting

## Lockfile Paths

| File | Purpose |
|------|---------|
| `~/.ai/hlidskjalf/KEEP_ALIVE.lock` | Must exist for hooks to allow actions |
| `~/.ai/hlidskjalf/KILL.lock` | Must not exist — presence blocks everything |
| `{workspace}/.ai/SYN.lock` | Presence triggers full workspace syn scan |

## Implementation Status

### Completed

- `Datagram` struct with typed `DatagramKind` and `Priority` enums lives in nornir's `socket_emit` crate
- `hlidskjalf_core` re-exports `Datagram`, `DatagramKind`, `Priority` from `socket_emit` (cross-repo path dep)
- Legacy `HookEvent` format accepted via `From<HookEvent> for Datagram` conversion
- All nornir sender binaries (`send_alert`, `send_warning`, `send_notification`, `send_heartbeat`, `send_datagram`) updated to use typed enums
- Rolling `.jsonl` log in hlidskjalf_core
- Lockfile monitoring in hlidskjalf_core
- Display filtering in HlidskjalfView (type + priority threshold)
- Speech threshold UI in HlidskjalfView

### Remaining

- Add `"exchange"` to the `type` enum in `datagram.schema.json` and `DatagramKind` in socket_emit
- Migrate `hook_io` and `syn` to emit new datagram format natively (currently still emit HookEvent, converted on receive)
- Remove old `WatchtowerEvent` / `HookEvent` once all emitters are migrated
