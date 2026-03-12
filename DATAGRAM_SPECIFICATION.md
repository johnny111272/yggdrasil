# Datagram Specification

Technical specification for the Hlidskjalf datagram format, JSON schema, and sender tool interfaces. This document is the build reference — see `DISPLAY_AND_FILTERING.md` for feed rendering and filter design.

## Transport

- **Primary:** UDP multicast on 239.0.0.1:9899 (loopback only, TTL=0) — live consumers (Hlidskjalf, etc.)
- **Secondary:** Unix stream socket at `/tmp/ai_logger.sock` — persistent archive via record_datagrams daemon
- **Framing:** Newline-delimited JSON (one complete JSON object per line, terminated by `\n`)
- **Encoding:** UTF-8
- **Connection model:** Fire-and-forget. Both channels send independently, either can fail silently.

## Datagram Schema

### Field Reference

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `timestamp` | number | yes | Unix epoch, float64, seconds | When the datagram was created |
| `source` | string | yes | 1–64 chars, `[a-z0-9_]` only | The binary that emitted this datagram via datagram |
| `kind` | string | yes | enum: see below | Content type — what this datagram IS |
| `priority` | string | yes | enum: see below | Severity/importance level |
| `workspace` | string | yes | 0–128 chars | Project/workspace name (empty string if not workspace-scoped) |
| `detail` | string | no | 0–512 chars | Human-readable one-line summary |
| `speech` | string | no | 0–256 chars | Voice alert text composed for spoken delivery |
| `payload` | object | no | — | Structured data whose shape is determined by `kind` |

### `kind` Enum

`kind` names what the datagram IS (content type), not who produced it. Drives rendering (which component draws it) and filtering (which chip toggles it).

| Value | Semantics |
|-------|-----------|
| `alert` | Something requires attention — security events, errors, denied actions |
| `quality` | Code quality report — issues grouped by (tool, code) with file-aggregated locations |
| `canary` | Proof-of-life pulse from an infrastructure component |
| `notify` | Informational status update — task progress, completion, state changes |
| `traffic` | API round-trip — semantic diff between consecutive Claude API exchanges |

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

Source names identify the emitting binary — the process that calls datagram. Use snake_case, no dots or slashes.

| Source | Component |
|--------|-----------|
| `syn` | Code quality checker (emits `kind: "quality"`) |
| `bifrost` | Traffic intercept pipeline (emits `kind: "traffic"`) |
| `send_alert` | Critical alert sender |
| `send_warning` | Warning sender |
| `send_notification` | Notification sender |
| `send_heartbeat` | Heartbeat/canary sender |
| `lockfile_monitor` | Hlidskjalf lockfile watcher |

New sources may be added freely — the field is not a closed enum.

## JSON Schema

File: `schemas/datagram.schema.json`

Payload schemas: `schemas/payloads/payload.quality.schema.json`, `schemas/payloads/payload.traffic.schema.json`

## Sender Binaries

All built as nornir crates. Fire-and-forget — senders never block or retry.

### Fixed-Severity Senders

These binaries have their `kind` and `priority` hardcoded. The binary name is the severity contract.

#### `send_heartbeat`

```
send_heartbeat <source>
```

| Field | Value |
|-------|-------|
| `kind` | `canary` |
| `priority` | `low` |
| `source` | from arg |
| `workspace` | auto-detected from `$CLAUDE_PROJECT_DIR` |

#### `send_notification`

```
send_notification <source> <message>
```

| Field | Value |
|-------|-------|
| `kind` | `notify` |
| `priority` | `normal` |
| `source` | from arg |
| `detail` | from `<message>` arg |

#### `send_warning`

```
send_warning <source> <message>
```

| Field | Value |
|-------|-------|
| `kind` | `alert` |
| `priority` | `high` |
| `source` | from arg |
| `detail` | from `<message>` arg |
| `speech` | auto-generated: `"Warning from {source}: {message}"` |

#### `send_alert`

```
send_alert <source> <message>
```

| Field | Value |
|-------|-------|
| `kind` | `alert` |
| `priority` | `critical` |
| `source` | from arg |
| `detail` | from `<message>` arg |
| `speech` | auto-generated: `"ALERT from {source}: {message}"` |

### Full-Protocol Sender

#### `send_datagram`

```
send_datagram --source <source> --type <kind> --priority <priority> [options]
```

| Flag | Required | Description |
|------|----------|-------------|
| `--source` | yes | Source identifier |
| `--type` | yes | Datagram kind (alert, quality, canary, notify, traffic) |
| `--priority` | yes | Priority level |
| `--workspace` | no | Override auto-detected workspace |
| `--detail` | no | Human-readable summary |
| `--speech` | no | Speech text |
| `--payload-file` | no | Path to JSON file for payload |
| `--payload` | no | Inline JSON string for payload |

Not deployed to LLM-accessible paths. For trusted tooling only.

## Payload Schemas

Payloads are kind-specific. Shape is determined entirely by `kind` — you never inspect the payload to identify the datagram.

### Quality Payload (`kind: "quality"`)

Schema: `schemas/payloads/payload.quality.schema.json`

Emitted by the syn binary. Groups issues by (tool, code) pair with file-aggregated locations.

```json
{
  "total": 17,
  "check_types": 5,
  "groups": [
    {
      "tool": "ruff",
      "code": "E501",
      "message": "Line too long",
      "severity": "warning",
      "count": 12,
      "file_count": 4,
      "signal": "",
      "direction": "",
      "canary": "",
      "locations": [
        { "file": "src/lib.rs", "lines": [42, 85] },
        { "file": "src/main.rs", "line": 200 }
      ]
    }
  ]
}
```

### Traffic Payload (`kind: "traffic"`)

Schema: `schemas/payloads/payload.traffic.schema.json`

Emitted by bifrost. See `DISPLAY_AND_FILTERING.md` for full field reference and size distributions.

## Receiver Behavior

### On datagram arrival (Hlidskjalf)

1. Receive on UDP multicast 239.0.0.1:9899
2. Parse JSON line
3. Emit as Tauri event `"datagram"` to frontend
4. Frontend applies display filters (kind, priority threshold, session visibility)
5. Frontend applies speech logic:
   - If `speech` field is present and priority >= speech threshold → speak it
   - If `speech` absent → silent

Note: Local event logging removed from Hlidskjalf. Persistence is handled by nornir's `record_datagrams` daemon via the Unix stream channel.

## Lockfile Paths

| File | Purpose |
|------|---------|
| `~/.ai/hlidskjalf/KEEP_ALIVE.lock` | Must exist for hooks to allow actions |
| `~/.ai/hlidskjalf/KILL.lock` | Must not exist — presence blocks everything |
| `{workspace}/.ai/SYN.lock` | Presence triggers full workspace syn scan |
