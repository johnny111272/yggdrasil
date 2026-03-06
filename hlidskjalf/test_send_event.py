#!/usr/bin/env python3
"""Send test events to Hlidskjalf via Unix stream socket.

Launch hlidskjalf.app first, then run this script.

Protocol: 4-byte big-endian length prefix + JSON bytes.
"""

import json
import socket
import struct
import time

SOCKET = "/tmp/hlidskjalf.sock"


def send(msg: dict) -> None:
    data = json.dumps(msg).encode()
    sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    try:
        sock.connect(SOCKET)
        sock.sendall(struct.pack(">I", len(data)))
        sock.sendall(data)
    finally:
        sock.close()


# 1. Simple warn event
send({
    "timestamp": time.time(),
    "category": "probing",
    "decision": "warn",
    "event_name": "read_rules_toml",
    "workspace": "galdr",
    "detail": "LLM attempted to read .gleipnir/rules.toml",
    "context_injected": "SIGNAL: Reading guardrail config suggests pattern-matching.",
})

time.sleep(0.3)

# 2. Gleipnir report payload
send({
    "timestamp": time.time(),
    "category": "suppression",
    "decision": "warn",
    "event_name": "gleipnir_summary",
    "workspace": "galdr",
    "detail": "17 violations across 5 check types",
    "context_injected": "GLEIPNIR: 17 violations across 5 check types\n\nno_print (error)...",
    "payload": {
        "type": "gleipnir_report",
        "total": 17,
        "check_types": 5,
        "groups": [
            {
                "code": "no_print",
                "severity": "error",
                "count": 13,
                "file_count": 1,
                "signal": "Application code returns typed data — it doesn't write to stdout. print() is an escape from the architecture.",
                "direction": "Diagnostic output → loguru.logger. Results → return typed data.",
                "canary": "print() in non-debug code usually means quick-and-dirty thinking has taken over.",
                "locations": [
                    {"file": "galdr/generate_structures.py", "line": 37},
                    {"file": "galdr/generate_structures.py", "line": 55},
                    {"file": "galdr/generate_structures.py", "line": 56},
                    {"file": "galdr/generate_structures.py", "line": 60},
                    {"file": "galdr/generate_structures.py", "line": 116},
                    {"file": "galdr/generate_structures.py", "line": 117},
                    {"file": "galdr/generate_structures.py", "line": 118},
                    {"file": "galdr/generate_structures.py", "line": 126},
                    {"file": "galdr/generate_structures.py", "line": 134},
                    {"file": "galdr/generate_structures.py", "line": 135},
                    {"file": "galdr/generate_structures.py", "line": 137},
                    {"file": "galdr/generate_structures.py", "line": 139},
                    {"file": "galdr/generate_structures.py", "line": 141},
                ],
            },
            {
                "code": "hardcoded_config",
                "severity": "error",
                "count": 1,
                "file_count": 1,
                "signal": "Config changes without code changes. Hard-coded values become invisible assumptions that break when the environment changes.",
                "direction": "Move to .toml config file or define as a Pydantic model in structures/.",
                "canary": "",
                "locations": [
                    {"file": "galdr/generate_structures.py", "line": 26},
                ],
            },
            {
                "code": "no_future_annotations",
                "severity": "error",
                "count": 1,
                "file_count": 1,
                "signal": "from __future__ import annotations was deprecated in Python 3.9. We are on 3.13. It breaks Pydantic validation by making all annotations strings at runtime.",
                "direction": "Remove the import. Fix forward references with explicit string annotations. Fix circular imports by restructuring dependencies.",
                "canary": "",
                "locations": [
                    {"file": "galdr/generate_structures.py", "line": 107},
                ],
            },
            {
                "code": "max_functions_outside_zones",
                "severity": "warning",
                "count": 1,
                "file_count": 1,
                "signal": "What is the ONE thing this module does? Files outside functions/ and structures/ should be thin wiring — not function collections.",
                "direction": "Move functions to functions/pure/ or functions/impure/.",
                "canary": "",
                "locations": [
                    {"file": "galdr/generate_structures.py", "line": 1},
                ],
            },
            {
                "code": "no_single_letter_names",
                "severity": "warning",
                "count": 1,
                "file_count": 1,
                "signal": "Names are instructions to every future session. A single letter forces every reader to trace back to the definition.",
                "direction": "What IS this value? Name it that.",
                "canary": "",
                "locations": [
                    {"file": "galdr/generate_structures.py", "line": 87},
                ],
            },
        ],
    },
})

time.sleep(0.3)

# 3. Deny event
send({
    "timestamp": time.time(),
    "category": "subversion",
    "decision": "deny",
    "event_name": "chflags_noschg",
    "workspace": "tyr",
    "detail": "LLM attempted to remove immutable flag from gleipnir source",
    "context_injected": "",
})

print("Sent 3 test events (probing warn, gleipnir report, subversion deny)")
