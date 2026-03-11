/**
 * JSON Schema Inspector — pure analysis functions.
 *
 * Port of inspect_schema.py analysis logic to TypeScript.
 * All functions are pure: JSON in → structured data out, no IO.
 */

// ============================================================================
// Types
// ============================================================================

export interface InspectedSchema {
  title: string;
  description: string;
  version: string;
  sections: InspectedSection[];
  crossGroupConds: Record<string, string[]>;
  stats: { fields: number; sections: number; semantic: [number, number] };
}

export interface InspectedSection {
  name: string;
  nullable: boolean;
  requirement: { label: string; cssClass: string };
  fields: InspectedField[];
}

export interface InspectedField {
  name: string;
  type: string;
  requirement: { label: string; cssClass: string };
  description?: string;
  defaultValue?: unknown;
  extensions: FieldExtensions;
  children?: InspectedField[];
  alternatives?: InspectedAlt[];
  isArrayItem?: boolean;
}

export interface InspectedAlt {
  label: string;
  type: string;
  fields?: InspectedField[];
}

export interface FieldExtensions {
  semantic?: SemanticExt;
  constraint?: ConstraintExt;
  notBlock?: string[];
  format?: string;
}

export interface SemanticExt {
  severity: string;
  intent?: string;
  checks?: string[];
  antiPatterns?: string[];
}

export interface ConstraintExt {
  constraints: Array<{
    rule: string;
    field?: string;
    severity?: string;
    reason?: string;
  }>;
}

type SchemaNode = Record<string, unknown>;

// ============================================================================
// $ref Resolution
// ============================================================================

export function resolveAllRefs(schema: SchemaNode): SchemaNode {
  const defs = (schema["$defs"] as Record<string, SchemaNode>) ?? {};
  if (Object.keys(defs).length === 0) return schema;
  return deepResolve(schema, defs) as SchemaNode;
}

function deepResolve(
  node: unknown,
  defs: Record<string, SchemaNode>,
): unknown {
  if (Array.isArray(node)) {
    return node.map((item) => deepResolve(item, defs));
  }
  if (node !== null && typeof node === "object") {
    const obj = node as Record<string, unknown>;
    if ("$ref" in obj) {
      const refPath = obj["$ref"];
      if (typeof refPath === "string" && refPath.startsWith("#/$defs/")) {
        const defName = refPath.slice("#/$defs/".length);
        const resolved = defs[defName];
        if (resolved !== undefined) {
          const base = deepResolve(resolved, defs) as Record<string, unknown>;
          const siblings: Record<string, unknown> = {};
          for (const [k, v] of Object.entries(obj)) {
            if (k !== "$ref") siblings[k] = deepResolve(v, defs);
          }
          if (Object.keys(siblings).length > 0) {
            return { ...base, ...siblings };
          }
          return base;
        }
      }
    }
    const result: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(obj)) {
      result[k] = deepResolve(v, defs);
    }
    return result;
  }
  return node;
}

// ============================================================================
// Nullable Unwrap
// ============================================================================

export function unwrapNullable(
  schema: SchemaNode,
): [SchemaNode, boolean] {
  const oneOf = schema["oneOf"];
  if (!Array.isArray(oneOf)) return [schema, false];
  let objVariant: SchemaNode | null = null;
  let hasNull = false;
  for (const variant of oneOf) {
    if (variant && typeof variant === "object") {
      if ((variant as SchemaNode)["type"] === "null") hasNull = true;
      else if ((variant as SchemaNode)["type"] === "object")
        objVariant = variant as SchemaNode;
    }
  }
  if (objVariant && hasNull) return [objVariant, true];
  return [schema, false];
}

// ============================================================================
// Type Summary
// ============================================================================

export function typeSummary(prop: SchemaNode): string {
  let t = prop["type"] ?? "?";

  // Nullable shorthand: {"type": ["string", "null"]}
  if (Array.isArray(t)) {
    const nonNull = t.filter((x) => x !== "null");
    const isNullable = nonNull.length < t.length;
    if (nonNull.length === 1) {
      t = nonNull[0];
      if (isNullable) return `${t}?`;
    } else {
      return nonNull.join(" | ");
    }
  }

  if ("const" in prop) return `const "${prop["const"]}"`;

  if ("enum" in prop) {
    const vals = (prop["enum"] as unknown[]).map(String).join(", ");
    return `enum [${vals}]`;
  }

  // Field-level oneOf (union types)
  if ("oneOf" in prop && t === "?") {
    const variants = prop["oneOf"] as SchemaNode[];
    const summaries = variants.map((v) => typeSummary(v));
    const unique = [...new Map(summaries.map((s) => [s, s])).values()];
    return unique.join(" | ");
  }

  if (t === "array") {
    // Tuple pattern: prefixItems at prop level
    if ("prefixItems" in prop) {
      const inner = (prop["prefixItems"] as SchemaNode[])
        .map((pi) => typeSummary(pi))
        .join(", ");
      return `tuple [${inner}]`;
    }
    const items = prop["items"];
    if (!items || typeof items !== "object" || Array.isArray(items))
      return "array [?]";
    const itemsObj = items as SchemaNode;
    if ("oneOf" in itemsObj) return "array [ALTERNATIVES]";
    if ("prefixItems" in itemsObj) {
      const inner = (itemsObj["prefixItems"] as SchemaNode[])
        .map((pi) => typeSummary(pi))
        .join(", ");
      return `tuple [${inner}]`;
    }
    if ("enum" in itemsObj) {
      const vals = (itemsObj["enum"] as unknown[]).map(String).join(", ");
      return `array [enum: ${vals}]`;
    }
    return `array [${typeSummary(itemsObj)}]`;
  }

  if (t === "integer") {
    const constraints: string[] = [];
    if ("minimum" in prop) constraints.push(`>=${prop["minimum"]}`);
    if (constraints.length) return `integer ${constraints.join(" ")}`;
    return "integer";
  }

  if (t === "string") {
    const fmt = prop["format"] as string | undefined;
    const pat = prop["pattern"] as string | undefined;
    if (fmt && pat) return `string /${pat}/ [${fmt}]`;
    if (fmt) return `string [${fmt}]`;
    if (pat) return `string /${pat}/`;
    return "string";
  }

  if (t === "boolean") return "boolean";

  return String(t);
}

// ============================================================================
// Conditional Extraction
// ============================================================================

function describeIfClause(ifClause: SchemaNode): string {
  const ifProps = ifClause["properties"] as Record<string, SchemaNode> | undefined;
  if (ifProps) {
    for (const [condField, condSchema] of Object.entries(ifProps)) {
      if (condSchema && typeof condSchema === "object") {
        if ("const" in condSchema) return `${condField}=${condSchema["const"]}`;
        if ("enum" in condSchema) {
          const vals = (condSchema["enum"] as unknown[]).map(String).join("|");
          return `${condField} in [${vals}]`;
        }
      }
      return `${condField}=?`;
    }
  }
  const ifRequired = ifClause["required"] as string[] | undefined;
  if (ifRequired?.length) return `${ifRequired[0]} present`;
  return "?";
}

export function extractConditionals(
  schemaObj: SchemaNode,
): Record<string, string[]> {
  const conditionals: Record<string, string[]> = {};
  const allOf = schemaObj["allOf"] as SchemaNode[] | undefined;
  if (!allOf) return conditionals;

  for (const item of allOf) {
    if (!("if" in item) || !("then" in item)) continue;

    const conditionDesc = describeIfClause(item["if"] as SchemaNode);
    const hasElse = "else" in item;
    const thenObj = item["then"] as SchemaNode;

    // ONLY WHEN: required iff condition (forbidden otherwise)
    // ALWAYS WHEN: required when condition (optional otherwise)
    const thenRequired = (thenObj["required"] as string[]) ?? [];
    for (const reqField of thenRequired) {
      const prefix = hasElse ? "ONLY WHEN" : "ALWAYS WHEN";
      (conditionals[reqField] ??= []).push(`${prefix} ${conditionDesc}`);
    }

    // NEVER WHEN: forbidden when condition (optional otherwise)
    const thenProps = (thenObj["properties"] as Record<string, unknown>) ?? {};
    for (const [fieldName, fieldSchema] of Object.entries(thenProps)) {
      if (fieldSchema === false) {
        (conditionals[fieldName] ??= []).push(`NEVER WHEN ${conditionDesc}`);
      }
    }
  }
  return conditionals;
}

// ============================================================================
// Cross-Group Conditionals
// ============================================================================

function unwrapNestedPath(
  obj: SchemaNode,
): [string | null, SchemaNode | false | null] {
  const parts: string[] = [];
  let current: SchemaNode = obj;

  while (current && typeof current === "object" && "properties" in current) {
    const props = current["properties"] as Record<string, unknown>;
    const keys = Object.keys(props);
    if (keys.length !== 1) break;
    const fieldName = keys[0];
    const inner = props[fieldName];
    parts.push(fieldName);

    if (inner && typeof inner === "object" && !Array.isArray(inner)) {
      const innerObj = inner as SchemaNode;
      if ("properties" in innerObj) {
        current = innerObj;
      } else if (
        "required" in innerObj &&
        !("properties" in innerObj)
      ) {
        const reqFields = innerObj["required"] as string[];
        if (reqFields?.length) {
          parts.push(reqFields[0]);
          return [parts.join("."), null];
        }
        return [parts.join("."), innerObj];
      } else {
        return [parts.join("."), innerObj];
      }
    } else if (inner === false) {
      return [parts.join("."), false];
    } else {
      return [parts.join("."), inner as SchemaNode | null];
    }
  }

  // Required at top level
  if (
    current &&
    typeof current === "object" &&
    "required" in current &&
    !("properties" in current)
  ) {
    const reqFields = current["required"] as string[];
    if (reqFields?.length) {
      parts.push(reqFields[0]);
      return [parts.join("."), null];
    }
  }
  return [null, null];
}

export function extractCrossGroupConditionals(
  schema: SchemaNode,
): Record<string, string[]> {
  const result: Record<string, string[]> = {};
  const allOf = schema["allOf"] as SchemaNode[] | undefined;
  if (!allOf) return result;

  for (const item of allOf) {
    if (!("if" in item) || !("then" in item)) continue;

    const [ifPath, ifSchema] = unwrapNestedPath(item["if"] as SchemaNode);
    if (ifPath === null) continue;

    let condition: string;
    if (ifSchema && typeof ifSchema === "object") {
      if ("const" in ifSchema) condition = `${ifPath}=${ifSchema["const"]}`;
      else if ("enum" in ifSchema) {
        const vals = (ifSchema["enum"] as unknown[]).map(String).join("|");
        condition = `${ifPath} in [${vals}]`;
      } else {
        condition = `${ifPath}=?`;
      }
    } else {
      condition = `${ifPath} present`;
    }

    const hasElse = "else" in item;
    const [thenPath, thenSchema] = unwrapNestedPath(
      item["then"] as SchemaNode,
    );
    if (thenPath === null) continue;

    if (thenSchema === false) {
      (result[thenPath] ??= []).push(`PROHIBITED WHEN ${condition}`);
    } else if (hasElse) {
      (result[thenPath] ??= []).push(`ONLY WHEN ${condition}`);
    } else {
      (result[thenPath] ??= []).push(`WHEN ${condition}`);
    }
  }
  return result;
}

// ============================================================================
// Field Requirement
// ============================================================================

export function fieldRequirement(
  fieldName: string,
  requiredSet: Set<string>,
  conditionals: Record<string, string[]>,
  parentMinProps: number | null,
): { label: string; cssClass: string } {
  const parts: string[] = [];

  if (requiredSet.has(fieldName)) parts.push("REQUIRED");

  if (conditionals[fieldName]) {
    parts.push(...conditionals[fieldName]);
  }

  if (parts.length === 0) {
    if (parentMinProps && parentMinProps >= 1)
      return { label: "ONE-OF (>=1 in group)", cssClass: "req-oneof" };
    return { label: "OPTIONAL", cssClass: "req-optional" };
  }

  const label = parts.join(" + ");
  if (parts.some((p) => p.includes("WHEN")))
    return { label, cssClass: "req-conditional" };
  return { label, cssClass: "req-required" };
}

// ============================================================================
// oneOf Label
// ============================================================================

function oneOfLabel(alt: SchemaNode, idx: number): string {
  if (alt["type"] === "object") {
    const props = Object.keys(
      (alt["properties"] as Record<string, unknown>) ?? {},
    );
    const req = (alt["required"] as string[]) ?? [];
    if (props.length === 1 && props[0] === "include") return "INCLUDE";
    if (req.length) return `ALT${idx + 1} (${req.join(", ")})`;
    if (props.length) return `ALT${idx + 1} (${props.slice(0, 3).join(", ")})`;
  }
  return `ALT${idx + 1}`;
}

// ============================================================================
// Semantic Counting
// ============================================================================

function countSemantic(schemaObj: SchemaNode): [number, number] {
  let semanticCount = 0;
  let totalCount = 0;
  const properties = (schemaObj["properties"] as Record<string, SchemaNode>) ?? {};

  for (const [, propSchema] of Object.entries(properties)) {
    const [unwrapped] = unwrapNullable(propSchema);
    if (unwrapped["type"] === "object" && unwrapped["properties"]) {
      const [s, t] = countSemantic(unwrapped);
      semanticCount += s;
      totalCount += t;
    } else if (unwrapped["type"] === "array") {
      const items = unwrapped["items"];
      if (!items || typeof items !== "object" || Array.isArray(items)) {
        totalCount += 1;
        continue;
      }
      const itemsObj = items as SchemaNode;
      if (itemsObj["type"] === "object" && itemsObj["properties"]) {
        const [s, t] = countSemantic(itemsObj);
        semanticCount += s;
        totalCount += t;
      } else if ("oneOf" in itemsObj) {
        for (const alt of itemsObj["oneOf"] as SchemaNode[]) {
          if (alt["type"] === "object" && alt["properties"]) {
            const [s, t] = countSemantic(alt);
            semanticCount += s;
            totalCount += t;
          }
        }
      } else {
        totalCount += 1;
        if (unwrapped["x-semantic"]) semanticCount += 1;
      }
    } else {
      totalCount += 1;
      if (unwrapped["x-semantic"]) semanticCount += 1;
    }
  }
  return [semanticCount, totalCount];
}

// ============================================================================
// Extension Collection
// ============================================================================

export function collectExtensions(fieldSchema: SchemaNode): FieldExtensions {
  const result: FieldExtensions = {};

  // Field-level extensions — raw JSON uses snake_case keys
  const semantic = fieldSchema["x-semantic"] as Record<string, unknown> | undefined;
  if (semantic) {
    result.semantic = {
      severity: (semantic["severity"] as string) ?? "?",
      intent: semantic["intent"] as string | undefined,
      checks: semantic["checks"] as string[] | undefined,
      antiPatterns: (semantic["anti_patterns"] ?? semantic["antiPatterns"]) as string[] | undefined,
    };
  }

  const constraint = fieldSchema["x-constraint"] as ConstraintExt | undefined;
  if (constraint) result.constraint = constraint;

  // Format
  let fmt = fieldSchema["format"] as string | undefined;
  if (!fmt && fieldSchema["type"] === "array") {
    const items = fieldSchema["items"] as SchemaNode | undefined;
    if (items && typeof items === "object") fmt = items["format"] as string;
  }
  if (fmt) result.format = fmt;

  // Not-block
  let notBlock = fieldSchema["not"] as SchemaNode | undefined;
  if (!notBlock && fieldSchema["type"] === "array") {
    const items = fieldSchema["items"] as SchemaNode | undefined;
    if (items && typeof items === "object")
      notBlock = items["not"] as SchemaNode | undefined;
  }
  if (notBlock) result.notBlock = notBlockPatterns(notBlock);

  // Bubble array items extensions
  if (fieldSchema["type"] === "array") {
    const items = fieldSchema["items"] as SchemaNode | undefined;
    if (items && typeof items === "object") {
      if (!result.semantic) {
        const itemSem = items["x-semantic"] as Record<string, unknown> | undefined;
        if (itemSem) {
          result.semantic = {
            severity: (itemSem["severity"] as string) ?? "?",
            intent: itemSem["intent"] as string | undefined,
            checks: itemSem["checks"] as string[] | undefined,
            antiPatterns: (itemSem["anti_patterns"] ?? itemSem["antiPatterns"]) as string[] | undefined,
          };
        }
      }
      if (!result.constraint) {
        const itemCon = items["x-constraint"] as ConstraintExt | undefined;
        if (itemCon) result.constraint = itemCon;
      }
    }
  }

  return result;
}

function notBlockPatterns(notBlock: SchemaNode): string[] {
  const anyOf = notBlock["anyOf"] as SchemaNode[] | undefined;
  if (anyOf) {
    return anyOf
      .filter((entry) => typeof entry === "object")
      .map((entry) => (entry["pattern"] as string) ?? "?");
  }
  const pat = notBlock["pattern"] as string | undefined;
  if (pat) return [pat];
  return [];
}

// ============================================================================
// First Sentence (description truncation)
// ============================================================================

function firstSentence(text: string, maxLen = 120): string {
  let line = text.trim().split("\n")[0];
  for (const sep of [". ", ".\n"]) {
    const idx = line.indexOf(sep);
    if (idx !== -1) {
      line = line.slice(0, idx + 1);
      break;
    }
  }
  if (line.length > maxLen) return line.slice(0, maxLen - 3) + "...";
  return line;
}

// ============================================================================
// Object Analysis (recursive tree walk — returns data, not HTML)
// ============================================================================

function analyzeObject(
  schemaObj: SchemaNode,
  crossGroupConds: Record<string, string[]>,
  pathPrefix: string,
): InspectedField[] {
  const fields: InspectedField[] = [];
  const properties =
    (schemaObj["properties"] as Record<string, SchemaNode>) ?? {};
  const requiredSet = new Set<string>(
    (schemaObj["required"] as string[]) ?? [],
  );
  const conditionals = extractConditionals(schemaObj);
  const minProps = (schemaObj["minProperties"] as number) ?? null;

  // Merge cross-group conditionals for fields at this path
  for (const fieldName of Object.keys(properties)) {
    const fieldPath = pathPrefix ? `${pathPrefix}.${fieldName}` : fieldName;
    if (crossGroupConds[fieldPath]) {
      (conditionals[fieldName] ??= []).push(
        ...crossGroupConds[fieldPath],
      );
    }
  }

  for (const [fieldName, fieldSchema] of Object.entries(properties)) {
    const fieldType = (fieldSchema["type"] as string) ?? "?";
    const req = fieldRequirement(fieldName, requiredSet, conditionals, minProps);
    const childPath = pathPrefix
      ? `${pathPrefix}.${fieldName}`
      : fieldName;

    // Get description
    let desc = fieldSchema["description"] as string | undefined;
    if (!desc && fieldType === "array") {
      const items = fieldSchema["items"] as SchemaNode | undefined;
      if (items && typeof items === "object")
        desc = items["description"] as string | undefined;
    }
    const shortDesc = desc ? firstSentence(desc) : undefined;
    const defaultVal = "default" in fieldSchema ? fieldSchema["default"] : undefined;

    if (fieldType === "object") {
      const subProps = fieldSchema["properties"] as Record<string, unknown> | undefined;
      if (subProps && Object.keys(subProps).length > 0) {
        const objMinProps = (fieldSchema["minProperties"] as number) ?? null;
        fields.push({
          name: fieldName,
          type: objMinProps && objMinProps >= 1 ? "object [>=1 field required]" : "object",
          requirement: req,
          description: shortDesc,
          defaultValue: defaultVal,
          extensions: collectExtensions(fieldSchema),
          children: analyzeObject(fieldSchema, crossGroupConds, childPath),
        });
      } else {
        fields.push({
          name: fieldName,
          type: "object",
          requirement: req,
          description: shortDesc,
          defaultValue: defaultVal,
          extensions: collectExtensions(fieldSchema),
        });
      }
    } else if (fieldType === "array") {
      // Tuple pattern
      if ("prefixItems" in fieldSchema) {
        fields.push({
          name: fieldName,
          type: typeSummary(fieldSchema),
          requirement: req,
          description: shortDesc,
          defaultValue: defaultVal,
          extensions: collectExtensions(fieldSchema),
        });
        continue;
      }

      const items = fieldSchema["items"];
      if (!items || typeof items !== "object" || Array.isArray(items)) {
        fields.push({
          name: fieldName,
          type: "array [?]",
          requirement: req,
          defaultValue: defaultVal,
          extensions: {},
        });
        continue;
      }

      const itemsObj = items as SchemaNode;
      const exts = collectExtensions(fieldSchema);

      if ("oneOf" in itemsObj) {
        // Array with alternatives
        const alts: InspectedAlt[] = [];
        for (let altIdx = 0; altIdx < (itemsObj["oneOf"] as SchemaNode[]).length; altIdx++) {
          const alt = (itemsObj["oneOf"] as SchemaNode[])[altIdx];
          const altLab = oneOfLabel(alt, altIdx);
          if (alt["type"] === "object" && alt["properties"]) {
            const altFields = analyzeObject(alt, {}, "");
            alts.push({ label: altLab, type: "object", fields: altFields });
          } else {
            alts.push({ label: altLab, type: typeSummary(alt) });
          }
        }
        fields.push({
          name: fieldName,
          type: "array [ALTERNATIVES]",
          requirement: req,
          description: shortDesc,
          defaultValue: defaultVal,
          extensions: exts,
          alternatives: alts,
        });
      } else if (itemsObj["type"] === "object" && itemsObj["properties"]) {
        // Array of objects — expand item fields
        const itemFields = analyzeObject(itemsObj, {}, "");
        fields.push({
          name: fieldName,
          type: "array [object]",
          requirement: req,
          description: shortDesc,
          defaultValue: defaultVal,
          extensions: exts,
          children: itemFields.map((f) => ({ ...f, isArrayItem: true })),
        });
      } else {
        fields.push({
          name: fieldName,
          type: typeSummary(fieldSchema),
          requirement: req,
          description: shortDesc,
          defaultValue: defaultVal,
          extensions: exts,
        });
      }
    } else {
      fields.push({
        name: fieldName,
        type: typeSummary(fieldSchema),
        requirement: req,
        description: shortDesc,
        defaultValue: defaultVal,
        extensions: collectExtensions(fieldSchema),
      });
    }
  }

  return fields;
}

// ============================================================================
// Top-Level Schema Analysis
// ============================================================================

export function analyzeSchema(rawJson: string): InspectedSchema {
  const raw = JSON.parse(rawJson) as SchemaNode;
  const schema = resolveAllRefs(raw);

  const title = (schema["title"] as string) ?? "Untitled Schema";
  const description = (schema["description"] as string) ?? "";
  const version = (schema["$schema"] as string) ?? "";

  const crossGroupConds = extractCrossGroupConditionals(schema);
  const properties =
    (schema["properties"] as Record<string, SchemaNode>) ?? {};
  const requiredSet = new Set<string>(
    (schema["required"] as string[]) ?? [],
  );

  const sections: InspectedSection[] = [];
  let totalFields = 0;
  let totalSemantic = 0;
  let totalLeaf = 0;

  for (const [sectionName, sectionSchema] of Object.entries(properties)) {
    const [unwrapped, nullable] = unwrapNullable(sectionSchema);

    // Section-level requirement
    const isRequired = requiredSet.has(sectionName);
    const sectionReq = isRequired
      ? { label: "REQUIRED", cssClass: "req-required" }
      : nullable
        ? { label: "OPTIONAL (nullable)", cssClass: "req-optional" }
        : { label: "OPTIONAL", cssClass: "req-optional" };

    if (unwrapped["type"] === "object" && unwrapped["properties"]) {
      const sectionFields = analyzeObject(
        unwrapped,
        crossGroupConds,
        sectionName,
      );
      const [sem, leaf] = countSemantic(unwrapped);
      totalSemantic += sem;
      totalLeaf += leaf;
      totalFields += countFields(sectionFields);
      sections.push({
        name: sectionName,
        nullable,
        requirement: sectionReq,
        fields: sectionFields,
      });
    } else {
      // Non-object top-level property (rare but possible)
      totalFields += 1;
      totalLeaf += 1;
      sections.push({
        name: sectionName,
        nullable,
        requirement: sectionReq,
        fields: [
          {
            name: sectionName,
            type: typeSummary(unwrapped),
            requirement: sectionReq,
            extensions: collectExtensions(unwrapped),
          },
        ],
      });
    }
  }

  return {
    title,
    description,
    version,
    sections,
    crossGroupConds,
    stats: {
      fields: totalFields,
      sections: sections.length,
      semantic: [totalSemantic, totalLeaf],
    },
  };
}

function countFields(fields: InspectedField[]): number {
  let count = 0;
  for (const f of fields) {
    count += 1;
    if (f.children) count += countFields(f.children);
    if (f.alternatives) {
      for (const alt of f.alternatives) {
        if (alt.fields) count += countFields(alt.fields);
      }
    }
  }
  return count;
}
