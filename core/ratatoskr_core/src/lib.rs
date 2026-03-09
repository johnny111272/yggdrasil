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

// ============================================================================
// JSON-LD Parsing
// ============================================================================

fn is_jsonld(value: &Value) -> bool {
    match value {
        Value::Object(obj) => {
            obj.contains_key("@context")
                || obj.contains_key("@graph")
                || obj.contains_key("@id")
        }
        Value::Array(arr) => arr.iter().any(|v| {
            v.as_object()
                .map(|o| o.contains_key("@id"))
                .unwrap_or(false)
        }),
        _ => false,
    }
}

fn extract_label(obj: &serde_json::Map<String, Value>, id: &str) -> String {
    for key in &["name", "rdfs:label", "label", "title", "schema:name"] {
        if let Some(Value::String(s)) = obj.get(*key) {
            return s.clone();
        }
        if let Some(Value::Object(inner)) = obj.get(*key) {
            if let Some(Value::String(s)) = inner.get("@value") {
                return s.clone();
            }
        }
    }
    id.rsplit(|c| c == '/' || c == '#')
        .next()
        .unwrap_or(id)
        .to_string()
}

fn extract_type(obj: &serde_json::Map<String, Value>) -> String {
    match obj.get("@type") {
        Some(Value::String(s)) => {
            s.rsplit(|c| c == '/' || c == '#')
                .next()
                .unwrap_or(s)
                .to_string()
        }
        Some(Value::Array(arr)) => {
            arr.first()
                .and_then(|v| v.as_str())
                .map(|s| {
                    s.rsplit(|c| c == '/' || c == '#')
                        .next()
                        .unwrap_or(s)
                        .to_string()
                })
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

fn parse_embedded_graph(value: &Value) -> Result<GraphData, String> {
    let obj = value
        .as_object()
        .ok_or("Expected object with nodes and edges")?;

    let mut nodes: Vec<GraphNode> = Vec::new();
    let mut edges: Vec<GraphEdge> = Vec::new();

    if let Some(Value::Array(node_arr)) = obj.get("nodes") {
        for node_val in node_arr {
            if let Some(node_obj) = node_val.as_object() {
                let id = node_obj
                    .get("@id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let label = node_obj
                    .get("label")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| extract_label(node_obj, &id));

                let node_type = extract_type(node_obj);

                let mut metadata: HashMap<String, Value> = HashMap::new();
                for (key, val) in node_obj.iter() {
                    if !key.starts_with('@') && key != "label" {
                        metadata.insert(key.clone(), val.clone());
                    }
                }

                if !id.is_empty() {
                    nodes.push(GraphNode {
                        id,
                        label,
                        node_type,
                        color: None,
                        metadata,
                    });
                }
            }
        }
    }

    if let Some(Value::Array(edge_arr)) = obj.get("edges") {
        for edge_val in edge_arr {
            if let Some(edge_obj) = edge_val.as_object() {
                let source = edge_obj
                    .get("from")
                    .or_else(|| edge_obj.get("source"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let target = edge_obj
                    .get("to")
                    .or_else(|| edge_obj.get("target"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let label = edge_obj
                    .get("data")
                    .or_else(|| edge_obj.get("label"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let edge_type = edge_obj
                    .get("direction")
                    .or_else(|| edge_obj.get("edge_type"))
                    .or_else(|| edge_obj.get("@type"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let weight = edge_obj
                    .get("weight")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0);

                if !source.is_empty() && !target.is_empty() {
                    edges.push(GraphEdge {
                        source,
                        target,
                        label,
                        edge_type,
                        weight,
                    });
                }
            }
        }
    }

    let mut metadata: HashMap<String, Value> = HashMap::new();
    if let Some(Value::String(name)) = obj.get("name") {
        metadata.insert("name".to_string(), Value::String(name.clone()));
    }
    if let Some(Value::String(desc)) = obj.get("description") {
        metadata.insert("description".to_string(), Value::String(desc.clone()));
    }
    if let Some(id) = obj.get("@id") {
        metadata.insert("@id".to_string(), id.clone());
    }
    if let Some(t) = obj.get("@type") {
        metadata.insert("@type".to_string(), t.clone());
    }

    Ok(GraphData {
        nodes,
        edges,
        metadata,
    })
}

fn jsonld_to_graph(value: Value) -> Result<GraphData, String> {
    if let Value::Object(ref obj) = value {
        if obj.contains_key("nodes") && obj.contains_key("edges") {
            return parse_embedded_graph(&value);
        }
    }

    let mut nodes: Vec<GraphNode> = Vec::new();
    let mut edges: Vec<GraphEdge> = Vec::new();
    let mut node_ids: HashSet<String> = HashSet::new();

    let objects: Vec<&serde_json::Map<String, Value>> = match &value {
        Value::Object(obj) => {
            if let Some(Value::Array(graph)) = obj.get("@graph") {
                graph
                    .iter()
                    .filter_map(|v| v.as_object())
                    .collect()
            } else if obj.contains_key("@id") {
                vec![obj]
            } else {
                obj.values()
                    .filter_map(|v| match v {
                        Value::Object(o) if o.contains_key("@id") => Some(o),
                        _ => None,
                    })
                    .collect()
            }
        }
        Value::Array(arr) => arr.iter().filter_map(|v| v.as_object()).collect(),
        _ => return Err("Invalid JSON-LD: expected object or array".to_string()),
    };

    for obj in &objects {
        if let Some(Value::String(id)) = obj.get("@id") {
            if !node_ids.contains(id) {
                node_ids.insert(id.clone());

                let label = extract_label(obj, id);
                let node_type = extract_type(obj);

                let mut metadata: HashMap<String, Value> = HashMap::new();
                for (key, val) in obj.iter() {
                    if !key.starts_with('@') && key != "name" && key != "label" && key != "title" {
                        match val {
                            Value::Object(o) if o.contains_key("@id") => {}
                            Value::Array(arr)
                                if arr.iter().any(|v| {
                                    v.as_object()
                                        .map(|o| o.contains_key("@id"))
                                        .unwrap_or(false)
                                }) => {}
                            _ => {
                                metadata.insert(key.clone(), val.clone());
                            }
                        }
                    }
                }

                nodes.push(GraphNode {
                    id: id.clone(),
                    label,
                    node_type,
                    color: None,
                    metadata,
                });
            }
        }
    }

    for obj in &objects {
        if let Some(Value::String(source_id)) = obj.get("@id") {
            for (key, val) in obj.iter() {
                if key.starts_with('@') {
                    continue;
                }

                let target_ids: Vec<String> = match val {
                    Value::Object(o) => {
                        if let Some(Value::String(target)) = o.get("@id") {
                            vec![target.clone()]
                        } else {
                            vec![]
                        }
                    }
                    Value::Array(arr) => arr
                        .iter()
                        .filter_map(|v| {
                            v.as_object()
                                .and_then(|o| o.get("@id"))
                                .and_then(|id| id.as_str())
                                .map(String::from)
                        })
                        .collect(),
                    _ => vec![],
                };

                for target_id in target_ids {
                    if node_ids.contains(&target_id) {
                        let label = key
                            .rsplit(|c| c == '/' || c == '#' || c == ':')
                            .next()
                            .unwrap_or(key)
                            .to_string();

                        edges.push(GraphEdge {
                            source: source_id.clone(),
                            target: target_id,
                            label: label.clone(),
                            edge_type: label,
                            weight: 1.0,
                        });
                    }
                }
            }
        }
    }

    Ok(GraphData {
        nodes,
        edges,
        metadata: HashMap::new(),
    })
}

// ============================================================================
// JSON-LD Reference Resolution
// ============================================================================

fn resolve_reference(id: &str, base_dir: &Path) -> Option<PathBuf> {
    if let Some(rest) = id.strip_prefix("kerak:patterns/") {
        let filename = format!("{}_pattern.jsonld", rest);
        let path = base_dir.join(&filename);
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
    let mut config = MergeConfig::default();

    if let Value::Object(obj) = value {
        if let Some(join) = obj.get("join_on") {
            match join {
                Value::String(s) => {
                    config.join_on.insert(s.clone());
                }
                Value::Array(arr) => {
                    for item in arr {
                        if let Value::String(s) = item {
                            config.join_on.insert(s.clone());
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(Value::String(p)) = obj.get("prefix") {
            config.prefix = Some(p.clone());
        }

        if let Some(Value::String(c)) = obj.get("color") {
            config.color = Some(c.clone());
        }

        if let Some(Value::Object(stylesheet)) = obj.get("stylesheet") {
            for (label, color) in stylesheet {
                if let Value::String(c) = color {
                    config.stylesheet.insert(label.clone(), c.clone());
                }
            }
        }
    }

    config
}

fn prefix_graph_ids(graph: &mut GraphData, prefix: &str, join_on: &HashSet<String>) {
    let mut id_map: HashMap<String, String> = HashMap::new();

    for node in &mut graph.nodes {
        let is_shared = join_on.contains(&node.label);

        if !is_shared && node.id.starts_with('#') {
            let new_id = format!("{}:{}", prefix, &node.id[1..]);
            id_map.insert(node.id.clone(), new_id.clone());
            node.id = new_id;
            node.label = format!("{}{}", prefix, node.label);
        }
    }

    for edge in &mut graph.edges {
        if let Some(new_source) = id_map.get(&edge.source) {
            edge.source = new_source.clone();
        }
        if let Some(new_target) = id_map.get(&edge.target) {
            edge.target = new_target.clone();
        }
    }
}

fn deduplicate_nodes(graph: &mut GraphData) {
    let mut seen: HashSet<String> = HashSet::new();
    graph.nodes.retain(|node| {
        if seen.contains(&node.id) {
            false
        } else {
            seen.insert(node.id.clone());
            true
        }
    });
}

fn apply_colors(graph: &mut GraphData, stylesheet: &HashMap<String, String>, default_color: Option<&str>) {
    for node in &mut graph.nodes {
        if node.color.is_some() {
            continue;
        }

        let unprefixed_label = node.label
            .split(": ")
            .last()
            .unwrap_or(&node.label);

        if let Some(color) = stylesheet.get(&node.label) {
            node.color = Some(color.clone());
        } else if let Some(color) = stylesheet.get(unprefixed_label) {
            node.color = Some(color.clone());
        } else if let Some(default) = default_color {
            node.color = Some(default.to_string());
        }
    }
}

fn extract_references(value: &Value) -> Vec<String> {
    let mut refs = Vec::new();

    if let Value::Object(obj) = value {
        for (key, val) in obj {
            if key == "@id" {
                continue;
            }

            match val {
                Value::Object(inner) => {
                    if let Some(Value::String(id)) = inner.get("@id") {
                        if !id.starts_with('#') {
                            refs.push(id.clone());
                        }
                    }
                }
                Value::Array(arr) => {
                    for item in arr {
                        if let Value::Object(inner) = item {
                            if let Some(Value::String(id)) = inner.get("@id") {
                                if !id.starts_with('#') {
                                    refs.push(id.clone());
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    refs
}

fn load_jsonld_with_refs(path: &Path, visited: &mut HashSet<PathBuf>) -> Result<GraphData, String> {
    load_jsonld_with_refs_inner(path, visited, &HashSet::new(), &HashMap::new())
}

fn load_jsonld_with_refs_inner(
    path: &Path,
    visited: &mut HashSet<PathBuf>,
    parent_join_on: &HashSet<String>,
    parent_stylesheet: &HashMap<String, String>,
) -> Result<GraphData, String> {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if visited.contains(&canonical) {
        return Ok(GraphData {
            nodes: vec![],
            edges: vec![],
            metadata: HashMap::new(),
        });
    }
    visited.insert(canonical.clone());

    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let value: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;

    let base_dir = path.parent().unwrap_or(Path::new("."));

    let config = extract_merge_config(&value);

    let mut combined_join_on = parent_join_on.clone();
    combined_join_on.extend(config.join_on.clone());

    let mut combined_stylesheet = parent_stylesheet.clone();
    combined_stylesheet.extend(config.stylesheet.clone());

    let refs = extract_references(&value);

    let mut graph = if let Value::Object(ref obj) = value {
        if obj.contains_key("nodes") && obj.contains_key("edges") {
            let mut g = parse_embedded_graph(&value)?;
            if let Some(ref prefix) = config.prefix {
                prefix_graph_ids(&mut g, prefix, &combined_join_on);
            }
            apply_colors(&mut g, &combined_stylesheet, config.color.as_deref());
            g
        } else {
            let id = value
                .get("@id")
                .and_then(|v| v.as_str())
                .unwrap_or("root")
                .to_string();

            let label = value
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(&id)
                .to_string();

            let node_type = extract_type(obj);

            let mut metadata: HashMap<String, Value> = HashMap::new();
            if let Some(desc) = obj.get("description") {
                metadata.insert("description".to_string(), desc.clone());
            }

            let color = combined_stylesheet.get(&label)
                .cloned()
                .or_else(|| config.color.clone());

            GraphData {
                nodes: vec![GraphNode {
                    id: id.clone(),
                    label,
                    node_type,
                    color,
                    metadata,
                }],
                edges: vec![],
                metadata: HashMap::new(),
            }
        }
    } else {
        let mut g = jsonld_to_graph(value)?;
        if let Some(ref prefix) = config.prefix {
            prefix_graph_ids(&mut g, prefix, &combined_join_on);
        }
        apply_colors(&mut g, &combined_stylesheet, config.color.as_deref());
        g
    };

    for ref_id in refs {
        if let Some(ref_path) = resolve_reference(&ref_id, base_dir) {
            match load_jsonld_with_refs_inner(&ref_path, visited, &combined_join_on, &combined_stylesheet) {
                Ok(ref_graph) => {
                    if let Some(parent_id) = graph.nodes.first().map(|n| n.id.clone()) {
                        if let Some(ref_root) = ref_graph.nodes.first() {
                            if !combined_join_on.contains(&ref_root.label) {
                                graph.edges.push(GraphEdge {
                                    source: parent_id,
                                    target: ref_root.id.clone(),
                                    label: "gates".to_string(),
                                    edge_type: "reference".to_string(),
                                    weight: 1.0,
                                });
                            }
                        }
                    }

                    graph.nodes.extend(ref_graph.nodes);
                    graph.edges.extend(ref_graph.edges);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load reference {}: {}", ref_id, e);
                }
            }
        }
    }

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
        let mut visited = HashSet::new();
        load_jsonld_with_refs(file_path, &mut visited)
    } else {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let value: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        if is_jsonld(&value) {
            let mut visited = HashSet::new();
            load_jsonld_with_refs(file_path, &mut visited)
        } else {
            serde_json::from_value(value)
                .map_err(|e| format!("Failed to parse graph JSON: {}", e))
        }
    }
}

pub fn save_graph(path: &str, graph: &GraphData) -> Result<(), String> {
    let content = serde_json::to_string_pretty(graph)
        .map_err(|e| format!("Failed to serialize graph: {}", e))?;

    std::fs::write(path, content)
        .map_err(|e| format!("Failed to write file: {}", e))?;

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
    let nodes = vec![
        GraphNode {
            id: "svalinn".to_string(),
            label: "Svalinn".to_string(),
            node_type: "app".to_string(),
            color: Some("#4361ee".to_string()),
            metadata: HashMap::new(),
        },
        GraphNode {
            id: "hlidskjalf".to_string(),
            label: "Hlidskjalf".to_string(),
            node_type: "app".to_string(),
            color: Some("#4361ee".to_string()),
            metadata: HashMap::new(),
        },
        GraphNode {
            id: "ratatoskr".to_string(),
            label: "Ratatoskr".to_string(),
            node_type: "app".to_string(),
            color: Some("#4361ee".to_string()),
            metadata: HashMap::new(),
        },
        GraphNode {
            id: "ui".to_string(),
            label: "@yggdrasil/ui".to_string(),
            node_type: "library".to_string(),
            color: Some("#6bcb77".to_string()),
            metadata: HashMap::new(),
        },
        GraphNode {
            id: "tauri".to_string(),
            label: "Tauri".to_string(),
            node_type: "framework".to_string(),
            color: Some("#ffd93d".to_string()),
            metadata: HashMap::new(),
        },
        GraphNode {
            id: "svelte".to_string(),
            label: "Svelte 5".to_string(),
            node_type: "framework".to_string(),
            color: Some("#ffd93d".to_string()),
            metadata: HashMap::new(),
        },
        GraphNode {
            id: "d3".to_string(),
            label: "D3.js".to_string(),
            node_type: "library".to_string(),
            color: Some("#6bcb77".to_string()),
            metadata: HashMap::new(),
        },
    ];

    let edges = vec![
        GraphEdge { source: "svalinn".into(), target: "ui".into(), label: "uses".into(), edge_type: "dependency".into(), weight: 1.0 },
        GraphEdge { source: "hlidskjalf".into(), target: "ui".into(), label: "uses".into(), edge_type: "dependency".into(), weight: 1.0 },
        GraphEdge { source: "ratatoskr".into(), target: "ui".into(), label: "uses".into(), edge_type: "dependency".into(), weight: 1.0 },
        GraphEdge { source: "svalinn".into(), target: "tauri".into(), label: "built with".into(), edge_type: "dependency".into(), weight: 1.0 },
        GraphEdge { source: "hlidskjalf".into(), target: "tauri".into(), label: "built with".into(), edge_type: "dependency".into(), weight: 1.0 },
        GraphEdge { source: "ratatoskr".into(), target: "tauri".into(), label: "built with".into(), edge_type: "dependency".into(), weight: 1.0 },
        GraphEdge { source: "svalinn".into(), target: "svelte".into(), label: "uses".into(), edge_type: "dependency".into(), weight: 1.0 },
        GraphEdge { source: "hlidskjalf".into(), target: "svelte".into(), label: "uses".into(), edge_type: "dependency".into(), weight: 1.0 },
        GraphEdge { source: "ratatoskr".into(), target: "svelte".into(), label: "uses".into(), edge_type: "dependency".into(), weight: 1.0 },
        GraphEdge { source: "ratatoskr".into(), target: "d3".into(), label: "uses".into(), edge_type: "dependency".into(), weight: 1.0 },
        GraphEdge { source: "ui".into(), target: "svelte".into(), label: "built with".into(), edge_type: "dependency".into(), weight: 1.0 },
    ];

    GraphData {
        nodes,
        edges,
        metadata: HashMap::new(),
    }
}
