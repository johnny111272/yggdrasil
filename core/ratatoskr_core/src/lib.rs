use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

// ============================================================================
// Graph Data Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub node_type: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub edge_type: String,
    #[serde(default = "default_weight")]
    pub weight: f64,
}

fn default_weight() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub node_types: HashMap<String, usize>,
    pub edge_types: HashMap<String, usize>,
    pub density: f64,
}

fn empty_graph() -> GraphData {
    GraphData {
        nodes: vec![],
        edges: vec![],
        metadata: HashMap::new(),
    }
}

// ============================================================================
// Shared Extraction Primitives
// ============================================================================

fn local_name(resource_path: &str) -> &str {
    resource_path.rsplit(|c| c == '/' || c == '#')
        .next()
        .unwrap_or(resource_path)
}

fn str_field<'a>(fields: &'a serde_json::Map<String, Value>, field_name: &str) -> Option<&'a str> {
    fields.get(field_name).and_then(|value| value.as_str())
}

fn str_or_default(fields: &serde_json::Map<String, Value>, field_name: &str, fallback: &str) -> String {
    str_field(fields, field_name).unwrap_or(fallback).to_string()
}

fn is_jsonld(value: &Value) -> bool {
    match value {
        Value::Object(fields) => {
            fields.contains_key("@context")
                || fields.contains_key("@graph")
                || fields.contains_key("@id")
        }
        Value::Array(items) => items.iter().any(|item| {
            item.as_object()
                .is_some_and(|fields| fields.contains_key("@id"))
        }),
        _ => false,
    }
}

fn extract_label(fields: &serde_json::Map<String, Value>, id: &str) -> String {
    for key in &["name", "rdfs:label", "label", "title", "schema:name"] {
        if let Some(text) = str_field(fields, key) {
            return text.to_string();
        }
        if let Some(Value::Object(inner)) = fields.get(*key) {
            if let Some(text) = str_field(inner, "@value") {
                return text.to_string();
            }
        }
    }
    local_name(id).to_string()
}

fn extract_type(fields: &serde_json::Map<String, Value>) -> String {
    match fields.get("@type") {
        Some(Value::String(iri)) => local_name(iri).to_string(),
        Some(Value::Array(arr)) => {
            arr.first()
                .and_then(|v| v.as_str())
                .map(|iri| local_name(iri).to_string())
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

fn into_object(value: Value) -> Option<serde_json::Map<String, Value>> {
    match value {
        Value::Object(map) => Some(map),
        _ => None,
    }
}

fn is_reference_value(json_value: &Value) -> bool {
    match json_value {
        Value::Object(fields) => fields.contains_key("@id"),
        Value::Array(items) => items.iter().any(|item| {
            item.as_object()
                .is_some_and(|fields| fields.contains_key("@id"))
        }),
        _ => false,
    }
}

fn collect_non_ld_metadata(fields: serde_json::Map<String, Value>, skip_keys: &[&str]) -> HashMap<String, Value> {
    fields.into_iter()
        .filter(|(key, _)| !key.starts_with('@') && !skip_keys.contains(&key.as_str()))
        .collect()
}

// ============================================================================
// Embedded Graph Parsing (nodes/edges format)
// ============================================================================

fn parse_embedded_node(node_obj: serde_json::Map<String, Value>) -> Option<GraphNode> {
    let id = str_field(&node_obj, "@id").unwrap_or("").to_string();
    if id.is_empty() {
        return None;
    }

    let label = str_field(&node_obj, "label")
        .map(String::from)
        .unwrap_or_else(|| extract_label(&node_obj, &id));
    let node_type = extract_type(&node_obj);

    Some(GraphNode {
        id,
        label,
        node_type,
        color: None,
        metadata: collect_non_ld_metadata(node_obj, &["label"]),
    })
}

fn parse_embedded_edge(edge_obj: &serde_json::Map<String, Value>) -> Option<GraphEdge> {
    let source = edge_obj.get("from")
        .or_else(|| edge_obj.get("source"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let target = edge_obj.get("to")
        .or_else(|| edge_obj.get("target"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if source.is_empty() || target.is_empty() {
        return None;
    }

    Some(GraphEdge {
        source: source.to_string(),
        target: target.to_string(),
        label: str_or_default(edge_obj, "data",
            str_field(edge_obj, "label").unwrap_or("")),
        edge_type: edge_obj.get("direction")
            .or_else(|| edge_obj.get("edge_type"))
            .or_else(|| edge_obj.get("@type"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        weight: edge_obj.get("weight").and_then(|v| v.as_f64()).unwrap_or(1.0),
    })
}

fn parse_embedded_graph(value: Value) -> Result<GraphData, String> {
    let mut fields = match value {
        Value::Object(map) => map,
        _ => return Err("Expected object with nodes and edges".into()),
    };

    let nodes: Vec<GraphNode> = fields.remove("nodes")
        .and_then(|v| match v { Value::Array(arr) => Some(arr), _ => None })
        .map(|arr| arr.into_iter()
            .filter_map(|v| match v { Value::Object(map) => Some(map), _ => None })
            .filter_map(parse_embedded_node)
            .collect())
        .unwrap_or_default();

    let edges: Vec<GraphEdge> = fields.remove("edges")
        .and_then(|v| match v { Value::Array(arr) => Some(arr), _ => None })
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_object().and_then(parse_embedded_edge))
            .collect())
        .unwrap_or_default();

    let meta_keys: &[&str] = &["name", "description", "@id", "@type"];
    let metadata: HashMap<String, Value> = meta_keys.iter()
        .filter_map(|&key| fields.remove(key).map(|val| (key.into(), val)))
        .collect();

    Ok(GraphData { nodes, edges, metadata })
}

// ============================================================================
// JSON-LD Graph Parsing (@id/@graph format)
// ============================================================================

fn collect_ld_objects(value: Value) -> Result<Vec<serde_json::Map<String, Value>>, String> {
    match value {
        Value::Object(mut fields) => {
            if let Some(Value::Array(graph)) = fields.remove("@graph") {
                Ok(graph.into_iter().filter_map(into_object).collect())
            } else if fields.contains_key("@id") {
                Ok(vec![fields])
            } else {
                Ok(fields.into_iter()
                    .filter_map(|(_, val)| into_object(val))
                    .filter(|map| map.contains_key("@id"))
                    .collect())
            }
        }
        Value::Array(items) => Ok(items.into_iter().filter_map(into_object).collect()),
        _ => Err("Invalid JSON-LD: expected object or array".into()),
    }
}

fn extract_target_ids(json_value: &Value) -> Vec<&str> {
    match json_value {
        Value::Object(fields) => {
            str_field(fields, "@id").into_iter().collect()
        }
        Value::Array(items) => {
            items.iter()
                .filter_map(|v| v.as_object())
                .filter_map(|fields| str_field(fields, "@id"))
                .collect()
        }
        _ => vec![],
    }
}

fn jsonld_to_graph(value: Value) -> Result<GraphData, String> {
    if let Value::Object(ref fields) = value {
        if fields.contains_key("nodes") && fields.contains_key("edges") {
            return parse_embedded_graph(value);
        }
    }

    let objects = collect_ld_objects(value)?;

    let mut nodes: Vec<GraphNode> = Vec::new();
    let mut edges: Vec<GraphEdge> = Vec::new();
    let mut node_ids: HashSet<String> = HashSet::new();

    for fields in objects {
        let Some(id_ref) = str_field(&fields, "@id") else { continue };
        let id = String::from(id_ref);
        if !node_ids.insert(id.clone()) { continue; }

        // Scan reference fields for edges (borrows fields)
        for (key, val) in fields.iter() {
            if key.starts_with('@') { continue; }
            for target_id in extract_target_ids(val) {
                let short_key = local_name(key);
                edges.push(GraphEdge {
                    source: id.clone(),
                    target: target_id.to_string(),
                    label: short_key.to_string(),
                    edge_type: short_key.to_string(),
                    weight: 1.0,
                });
            }
        }

        // Extract label/type by reference, then consume fields into metadata
        let label = extract_label(&fields, &id);
        let node_type = extract_type(&fields);
        let metadata: HashMap<String, Value> = fields.into_iter()
            .filter(|(key, val)| {
                !key.starts_with('@')
                    && !["name", "label", "title"].contains(&key.as_str())
                    && !is_reference_value(val)
            })
            .collect();

        nodes.push(GraphNode { id, label, node_type, color: None, metadata });
    }

    // Filter edges to only reference known nodes
    edges.retain(|edge| node_ids.contains(&edge.target));

    Ok(GraphData { nodes, edges, metadata: HashMap::new() })
}

// ============================================================================
// JSON-LD Reference Resolution
// ============================================================================

fn resolve_reference(id: &str, base_dir: &Path) -> Option<PathBuf> {
    if let Some(rest) = id.strip_prefix("kerak:patterns/") {
        let path = base_dir.join(format!("{}_pattern.jsonld", rest));
        if path.exists() {
            return Some(path);
        }
    }

    if id.ends_with(".jsonld") || id.ends_with(".json") {
        let path = base_dir.join(id);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

#[derive(Debug, Clone, Default)]
struct MergeConfig {
    join_on: HashSet<String>,
    prefix: Option<String>,
    color: Option<String>,
    stylesheet: HashMap<String, String>,
}

fn extract_merge_config(value: &Value) -> MergeConfig {
    let Some(fields) = value.as_object() else { return MergeConfig::default() };
    let mut config = MergeConfig::default();

    match fields.get("join_on") {
        Some(Value::String(single)) => { config.join_on.insert(single.to_string()); }
        Some(Value::Array(items)) => {
            for item in items.iter().filter_map(|v| v.as_str()) {
                config.join_on.insert(item.to_string());
            }
        }
        _ => {}
    }

    config.prefix = str_field(fields, "prefix").map(String::from);
    config.color = str_field(fields, "color").map(String::from);

    if let Some(Value::Object(stylesheet)) = fields.get("stylesheet") {
        for (label, val) in stylesheet {
            if let Some(color) = val.as_str() {
                config.stylesheet.insert(label.to_string(), color.to_string());
            }
        }
    }

    config
}

fn prefix_graph_ids(graph: &mut GraphData, prefix: &str, join_on: &HashSet<String>) {
    let mut id_map: HashMap<String, String> = HashMap::new();

    for node in &mut graph.nodes {
        if join_on.contains(&node.label) || !node.id.starts_with('#') {
            continue;
        }
        let new_id = format!("{}:{}", prefix, &node.id[1..]);
        id_map.insert(std::mem::take(&mut node.id), new_id.clone());
        node.id = new_id;
        node.label = format!("{}{}", prefix, node.label);
    }

    for edge in &mut graph.edges {
        if let Some(new_source) = id_map.get(&edge.source) {
            edge.source.clone_from(new_source);
        }
        if let Some(new_target) = id_map.get(&edge.target) {
            edge.target.clone_from(new_target);
        }
    }
}

fn deduplicate_nodes(graph: &mut GraphData) {
    let mut seen: HashSet<String> = HashSet::new();
    graph.nodes.retain(|node| seen.insert(node.id.clone()));
}

fn apply_colors(graph: &mut GraphData, stylesheet: &HashMap<String, String>, default_color: Option<&str>) {
    for node in &mut graph.nodes {
        if node.color.is_some() {
            continue;
        }

        let unprefixed_label = node.label.split(": ").last().unwrap_or(&node.label);

        if let Some(color) = stylesheet.get(&node.label).or_else(|| stylesheet.get(unprefixed_label)) {
            node.color = Some(color.to_string());
        } else if let Some(default) = default_color {
            node.color = Some(default.to_string());
        }
    }
}

fn extract_references(value: &Value) -> Vec<String> {
    let Some(fields) = value.as_object() else { return vec![] };

    fields.iter()
        .filter(|(key, _)| key.as_str() != "@id")
        .flat_map(|(_, val)| match val {
            Value::Object(inner) => {
                str_field(inner, "@id")
                    .filter(|id| !id.starts_with('#'))
                    .map(String::from)
                    .into_iter()
                    .collect::<Vec<_>>()
            }
            Value::Array(items) => {
                items.iter()
                    .filter_map(|item| item.as_object())
                    .filter_map(|inner| str_field(inner, "@id"))
                    .filter(|id| !id.starts_with('#'))
                    .map(String::from)
                    .collect()
            }
            _ => vec![],
        })
        .collect()
}

struct RefTraversal {
    visited: HashSet<PathBuf>,
    join_on: HashSet<String>,
    stylesheet: HashMap<String, String>,
}

fn load_jsonld_with_refs(path: &Path) -> Result<GraphData, String> {
    let mut traversal = RefTraversal {
        visited: HashSet::new(),
        join_on: HashSet::new(),
        stylesheet: HashMap::new(),
    };
    load_jsonld_with_refs_inner(path, &mut traversal)
}

fn build_graph_from_value(
    value: Value,
    prefix: Option<&str>,
    default_color: Option<&str>,
    combined_join_on: &HashSet<String>,
    combined_stylesheet: &HashMap<String, String>,
) -> Result<GraphData, String> {
    let has_embedded = matches!(&value, Value::Object(fields) if fields.contains_key("nodes") && fields.contains_key("edges"));
    let is_object = matches!(&value, Value::Object(_));

    if has_embedded {
        let mut graph = parse_embedded_graph(value)?;
        if let Some(pfx) = prefix {
            prefix_graph_ids(&mut graph, pfx, combined_join_on);
        }
        apply_colors(&mut graph, combined_stylesheet, default_color);
        return Ok(graph);
    }

    if is_object {
        let Value::Object(mut fields) = value else { unreachable!() };
        let id = str_field(&fields, "@id").unwrap_or("root").to_string();
        let label = str_field(&fields, "name").unwrap_or(&id).to_string();
        let node_type = extract_type(&fields);

        let mut metadata: HashMap<String, Value> = HashMap::new();
        if let Some(desc) = fields.remove("description") {
            metadata.insert("description".into(), desc);
        }

        let color = combined_stylesheet.get(label.as_str())
            .map(|c| c.to_string())
            .or_else(|| default_color.map(String::from));

        return Ok(GraphData {
            nodes: vec![GraphNode { id, label, node_type, color, metadata }],
            edges: vec![],
            metadata: HashMap::new(),
        });
    }

    let mut graph = jsonld_to_graph(value)?;
    if let Some(pfx) = prefix {
        prefix_graph_ids(&mut graph, pfx, combined_join_on);
    }
    apply_colors(&mut graph, combined_stylesheet, default_color);
    Ok(graph)
}

fn merge_referenced_graphs(
    graph: &mut GraphData,
    refs: Vec<String>,
    base_dir: &Path,
    traversal: &mut RefTraversal,
) {
    for ref_id in refs {
        let Some(ref_path) = resolve_reference(&ref_id, base_dir) else { continue };
        let Ok(ref_graph) = load_jsonld_with_refs_inner(&ref_path, traversal) else { continue };

        if let (Some(parent), Some(ref_root)) = (graph.nodes.first(), ref_graph.nodes.first()) {
            if !traversal.join_on.contains(&ref_root.label) {
                graph.edges.push(GraphEdge {
                    source: parent.id.clone(),
                    target: ref_root.id.clone(),
                    label: "gates".into(),
                    edge_type: "reference".into(),
                    weight: 1.0,
                });
            }
        }

        graph.nodes.extend(ref_graph.nodes);
        graph.edges.extend(ref_graph.edges);
    }
}

fn load_jsonld_with_refs_inner(
    path: &Path,
    traversal: &mut RefTraversal,
) -> Result<GraphData, String> {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !traversal.visited.insert(canonical) {
        return Ok(empty_graph());
    }

    let content = std::fs::read_to_string(path)
        .map_err(|err| format!("Failed to read {}: {}", path.display(), err))?;

    let value: Value = serde_json::from_str(&content)
        .map_err(|err| format!("Failed to parse {}: {}", path.display(), err))?;

    let base_dir = path.parent().unwrap_or(Path::new("."));
    let config = extract_merge_config(&value);
    let prefix = config.prefix;
    let default_color = config.color;

    traversal.join_on.extend(config.join_on);
    for (label, color) in config.stylesheet {
        traversal.stylesheet.entry(label).or_insert(color);
    }

    let refs = extract_references(&value);
    let mut graph = build_graph_from_value(
        value,
        prefix.as_deref(),
        default_color.as_deref(),
        &traversal.join_on,
        &traversal.stylesheet,
    )?;

    merge_referenced_graphs(&mut graph, refs, base_dir, traversal);
    deduplicate_nodes(&mut graph);

    Ok(graph)
}

// ============================================================================
// Public Functions
// ============================================================================

pub fn load_graph(path: &str) -> Result<GraphData, String> {
    let file_path = Path::new(path);

    if !file_path.is_file() {
        return Err(format!("Not a file: {}", path));
    }

    let is_ld = path.ends_with(".jsonld") || path.ends_with(".json-ld");

    if is_ld {
        load_jsonld_with_refs(file_path)
    } else {
        let content = std::fs::read_to_string(file_path)
            .map_err(|err| format!("Failed to read file: {}", err))?;

        let value: Value = serde_json::from_str(&content)
            .map_err(|err| format!("Failed to parse JSON: {}", err))?;

        if is_jsonld(&value) {
            load_jsonld_with_refs(file_path)
        } else {
            serde_json::from_value(value)
                .map_err(|err| format!("Failed to parse graph JSON: {}", err))
        }
    }
}

pub fn save_graph(path: &str, graph: &GraphData) -> Result<(), String> {
    let content = serde_json::to_string_pretty(graph)
        .map_err(|err| format!("Failed to serialize graph: {}", err))?;

    std::fs::write(path, content)
        .map_err(|err| format!("Failed to write file: {}", err))?;

    Ok(())
}

pub fn get_graph_stats(graph: &GraphData) -> GraphStats {
    let node_count = graph.nodes.len();
    let edge_count = graph.edges.len();

    let mut node_types: HashMap<String, usize> = HashMap::new();
    for node in &graph.nodes {
        let type_name = if node.node_type.is_empty() { "default" } else { &node.node_type };
        *node_types.entry(type_name.to_string()).or_insert(0) += 1;
    }

    let mut edge_types: HashMap<String, usize> = HashMap::new();
    for edge in &graph.edges {
        let type_name = if edge.edge_type.is_empty() { "default" } else { &edge.edge_type };
        *edge_types.entry(type_name.to_string()).or_insert(0) += 1;
    }

    let density = if node_count > 1 {
        edge_count as f64 / (node_count as f64 * (node_count as f64 - 1.0))
    } else {
        0.0
    };

    GraphStats {
        node_count,
        edge_count,
        node_types,
        edge_types,
        density,
    }
}

pub fn generate_sample_graph() -> GraphData {
    let make_node = |id: &str, label: &str, node_type: &str, color: &str| -> GraphNode {
        GraphNode {
            id: id.to_string(),
            label: label.to_string(),
            node_type: node_type.to_string(),
            color: Some(color.to_string()),
            metadata: HashMap::new(),
        }
    };

    let make_edge = |source: &str, target: &str, label: &str| -> GraphEdge {
        GraphEdge {
            source: source.to_string(),
            target: target.to_string(),
            label: label.to_string(),
            edge_type: "dependency".into(),
            weight: 1.0,
        }
    };

    let app_color = "#4361ee";
    let lib_color = "#6bcb77";
    let fw_color = "#ffd93d";

    GraphData {
        nodes: vec![
            make_node("svalinn", "Svalinn", "app", app_color),
            make_node("hlidskjalf", "Hlidskjalf", "app", app_color),
            make_node("ratatoskr", "Ratatoskr", "app", app_color),
            make_node("ui", "@yggdrasil/ui", "library", lib_color),
            make_node("tauri", "Tauri", "framework", fw_color),
            make_node("svelte", "Svelte 5", "framework", fw_color),
            make_node("d3", "D3.js", "library", lib_color),
        ],
        edges: vec![
            make_edge("svalinn", "ui", "uses"),
            make_edge("hlidskjalf", "ui", "uses"),
            make_edge("ratatoskr", "ui", "uses"),
            make_edge("svalinn", "tauri", "built with"),
            make_edge("hlidskjalf", "tauri", "built with"),
            make_edge("ratatoskr", "tauri", "built with"),
            make_edge("svalinn", "svelte", "uses"),
            make_edge("hlidskjalf", "svelte", "uses"),
            make_edge("ratatoskr", "svelte", "uses"),
            make_edge("ratatoskr", "d3", "uses"),
            make_edge("ui", "svelte", "built with"),
        ],
        metadata: HashMap::new(),
    }
}
