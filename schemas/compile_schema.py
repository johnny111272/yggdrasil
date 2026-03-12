#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.13"
# ///
"""Compile datagram schema with payload subschemas inlined.

Reads the envelope schema and all payload schemas, produces a single
validate.datagram.schema.json with conditional payload enforcement.

Nested $defs inside payload schemas are hoisted to the root $defs with
namespaced keys (e.g. quality_check_group) and all internal $ref paths
are rewritten to match.

Source schemas (human-readable, standalone):
  datagram.schema.json
  payloads/payload.quality.schema.json
  payloads/payload.traffic.schema.json

Output (machine-enforceable, embedded in nornir binaries):
  validate.datagram.schema.json
"""

import json
import sys
from pathlib import Path

SCHEMAS_DIR = Path(__file__).parent
PAYLOADS_DIR = SCHEMAS_DIR / "payloads"

ENVELOPE = SCHEMAS_DIR / "datagram.schema.json"
OUTPUT = SCHEMAS_DIR / "validate.datagram.schema.json"

STANDALONE_FIELDS = ("$schema", "$id", "title", "description")

PAYLOAD_SCHEMAS = {
    "quality": PAYLOADS_DIR / "payload.quality.schema.json",
    "traffic": PAYLOADS_DIR / "payload.traffic.schema.json",
}


def hoist_nested_defs(schema, prefix, hoisted):
    """Extract nested $defs to root level with prefixed names, rewrite $refs."""
    if not isinstance(schema, dict):
        return 0

    count = 0
    nested_defs = schema.pop("$defs", None)
    if isinstance(nested_defs, dict):
        for name, definition in nested_defs.items():
            hoisted_name = f"{prefix}_{name}"
            hoisted[hoisted_name] = definition
            count += 1
            count += hoist_nested_defs(definition, prefix, hoisted)

    for key, value in schema.items():
        if key == "$ref" and isinstance(value, str) and value.startswith("#/$defs/"):
            old_name = value.removeprefix("#/$defs/")
            schema[key] = f"#/$defs/{prefix}_{old_name}"
        elif isinstance(value, dict):
            count += hoist_nested_defs(value, prefix, hoisted)
        elif isinstance(value, list):
            for item in value:
                count += hoist_nested_defs(item, prefix, hoisted)

    return count


def conditional_kind(conditional):
    """Extract the kind value from an if/then conditional, or None."""
    return (
        conditional
        .get("if", {})
        .get("properties", {})
        .get("kind", {})
        .get("const")
    )


def merge_conditionals(envelope_conditionals, payload_conditionals, payload_kinds):
    """Merge envelope conditionals (classifier enforcement) with payload conditionals.

    For kinds that appear in both, merge their then clauses (union required, merge properties).
    Envelope-only conditionals pass through unchanged.
    Payload-only conditionals are appended.
    """
    merged = []
    matched_kinds = set()

    for existing in envelope_conditionals:
        kind_value = conditional_kind(existing)
        if kind_value and kind_value in payload_kinds:
            for generated in payload_conditionals:
                if conditional_kind(generated) == kind_value:
                    existing_then = existing.get("then", {})
                    generated_then = generated["then"]
                    merged.append({
                        "if": existing["if"],
                        "then": {
                            "required": sorted(
                                set(existing_then.get("required", []))
                                | set(generated_then.get("required", []))
                            ),
                            "properties": {
                                **existing_then.get("properties", {}),
                                **generated_then.get("properties", {}),
                            },
                        },
                    })
                    matched_kinds.add(kind_value)
                    break
        else:
            merged.append(existing)

    for generated in payload_conditionals:
        if conditional_kind(generated) not in matched_kinds:
            merged.append(generated)

    return merged


def main() -> int:
    for path in [ENVELOPE, *PAYLOAD_SCHEMAS.values()]:
        if not path.exists():
            sys.stderr.write(f"MISSING: {path}\n")
            return 1

    envelope = json.loads(ENVELOPE.read_text())

    root_defs = {}
    payload_conditionals = []

    for kind, path in sorted(PAYLOAD_SCHEMAS.items()):
        payload_schema = json.loads(path.read_text())
        for key in STANDALONE_FIELDS:
            payload_schema.pop(key, None)

        hoist_nested_defs(payload_schema, kind, root_defs)

        def_name = f"{kind}_payload"
        root_defs[def_name] = payload_schema
        payload_conditionals.append({
            "if": {"properties": {"kind": {"const": kind}}},
            "then": {
                "required": ["payload"],
                "properties": {"payload": {"$ref": f"#/$defs/{def_name}"}},
            },
        })

    envelope["$id"] = "validate-datagram-schema-v1"
    envelope["$defs"] = root_defs
    envelope["allOf"] = merge_conditionals(
        envelope.get("allOf", []),
        payload_conditionals,
        set(PAYLOAD_SCHEMAS.keys()),
    )

    OUTPUT.write_text(json.dumps(envelope, indent=2) + "\n")

    hoisted_count = len(root_defs) - len(PAYLOAD_SCHEMAS)
    sys.stdout.write(
        f"OK: {OUTPUT.name} "
        f"({len(PAYLOAD_SCHEMAS)} payloads, {hoisted_count} hoisted defs)\n"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
