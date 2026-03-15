<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { Button, AppHeader, ErrorBanner, EmptyState, SidebarLayout, TreeNode, Breadcrumbs } from "@yggdrasil/ui";
  import { onMount } from "svelte";
  import * as d3 from "d3";

  let {
    commands = {
      list_directory: "list_directory",
      load_graph: "load_graph",
      save_graph: "save_graph",
      get_graph_stats: "get_graph_stats",
      generate_sample_graph: "generate_sample_graph",
    },
  }: {
    commands?: {
      list_directory: string;
      load_graph: string;
      save_graph: string;
      get_graph_stats: string;
      generate_sample_graph: string;
    };
  } = $props();

  interface GraphNode {
    id: string;
    label: string;
    node_type: string;
    color?: string | null;
    metadata: Record<string, unknown>;
    // D3 simulation properties
    x?: number;
    y?: number;
    fx?: number | null;
    fy?: number | null;
  }

  interface GraphEdge {
    source: string | GraphNode;
    target: string | GraphNode;
    label: string;
    edge_type: string;
    weight: number;
  }

  interface GraphData {
    nodes: GraphNode[];
    edges: GraphEdge[];
    metadata: Record<string, unknown>;
  }

  interface GraphStats {
    node_count: number;
    edge_count: number;
    node_types: Record<string, number>;
    edge_types: Record<string, number>;
    density: number;
  }

  let graphData: GraphData | null = $state(null);
  let stats: GraphStats | null = $state(null);
  let selectedNode: GraphNode | null = $state(null);
  let error = $state("");
  let loading = $state(false);
  let showStats = $state(true);
  let showSidebar = $state(true);
  let svgElement: SVGSVGElement;
  let simulation: d3.Simulation<GraphNode, GraphEdge> | null = null;

  // ── File tree state ──────────────────────────────────────────────
  interface FileEntry {
    name: string;
    path: string;
    is_dir: boolean;
    extension: string | null;
    size_bytes: number;
  }

  interface TreeNodeData {
    name: string;
    path: string;
    is_dir: boolean;
    expanded?: boolean;
    loading?: boolean;
    children?: TreeNodeData[];
  }

  const HOME = "/Users/johnny";
  let directory = $state(`${HOME}/.ai`);
  let showHidden = $state(false);
  let treeNodes: TreeNodeData[] = $state([]);
  let selectedFile: string | null = $state(null);

  const GRAPH_EXTENSIONS = new Set(["json", "jsonld", "json-ld"]);

  function isGraphFile(entry: FileEntry): boolean {
    return !entry.is_dir && entry.extension != null && GRAPH_EXTENSIONS.has(entry.extension);
  }

  function toTreeNode(entry: FileEntry): TreeNodeData {
    return {
      name: entry.name,
      path: entry.path,
      is_dir: entry.is_dir,
      expanded: false,
      children: entry.is_dir ? [] : undefined,
    };
  }

  async function loadDirectory(path: string) {
    try {
      const entries = await invoke<FileEntry[]>(commands.list_directory, {
        directory: path,
        showHidden,
      });
      // Show directories + graph files only
      return entries
        .filter(entry => entry.is_dir || isGraphFile(entry))
        .map(toTreeNode);
    } catch (e) {
      error = String(e);
      return [];
    }
  }

  async function loadRootDirectory() {
    treeNodes = await loadDirectory(directory);
  }

  function navigateToDirectory(path: string) {
    directory = path;
    loadRootDirectory();
  }

  async function handleTreeToggle(path: string) {
    const toggle = async (nodes: TreeNodeData[]): Promise<TreeNodeData[]> => {
      return Promise.all(nodes.map(async (node) => {
        if (node.path === path && node.is_dir) {
          if (node.expanded) {
            return { ...node, expanded: false };
          }
          const children = await loadDirectory(path);
          return { ...node, expanded: true, children };
        }
        if (node.children) {
          return { ...node, children: await toggle(node.children) };
        }
        return node;
      }));
    };
    treeNodes = await toggle(treeNodes);
  }

  async function handleTreeSelect(path: string) {
    selectedFile = path;
    loading = true;
    error = "";
    try {
      graphData = await invoke<GraphData>(commands.load_graph, { path });
      stats = await invoke<GraphStats>(commands.get_graph_stats, { graph: graphData });
      renderGraph();
    } catch (e) {
      error = String(e);
    }
    loading = false;
  }

  function handleTreeDblClickDir(path: string) {
    navigateToDirectory(path);
  }

  function getFileIcon(path: string, isDir: boolean): string {
    if (isDir) return "";
    if (path.endsWith(".jsonld") || path.endsWith(".json-ld")) return "\uD83D\uDD17";
    return "\uD83D\uDCC8";
  }

  function cssVar(name: string): string {
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  }

  function nodeColorMap(): Record<string, string> {
    return {
      app: cssVar("--graph-node-app"),
      library: cssVar("--graph-node-library"),
      framework: cssVar("--graph-node-framework"),
      default: cssVar("--graph-node-default"),
    };
  }

  function getNodeColor(node: GraphNode): string {
    if (node.color) return node.color;
    const colors = nodeColorMap();
    return colors[node.node_type] || colors.default;
  }

  function getTypeColor(nodeType: string): string {
    const colors = nodeColorMap();
    return colors[nodeType] || colors.default;
  }

  async function loadSampleGraph() {
    loading = true;
    error = "";
    try {
      graphData = await invoke<GraphData>(commands.generate_sample_graph);
      stats = await invoke<GraphStats>(commands.get_graph_stats, { graph: graphData });
      renderGraph();
    } catch (e) {
      error = String(e);
    }
    loading = false;
  }

  async function loadGraphFromFile() {
    const selected = await open({
      filters: [{ name: "Graph Files", extensions: ["json", "jsonld", "json-ld"] }],
    });
    if (selected && typeof selected === "string") {
      loading = true;
      error = "";
      try {
        graphData = await invoke<GraphData>(commands.load_graph, { path: selected });
        stats = await invoke<GraphStats>(commands.get_graph_stats, { graph: graphData });
        renderGraph();
      } catch (e) {
        error = String(e);
      }
      loading = false;
    }
  }

  async function saveGraph() {
    if (!graphData) return;
    const path = await save({
      filters: [{ name: "JSON", extensions: ["json"] }],
      defaultPath: "graph.json",
    });
    if (path) {
      try {
        await invoke(commands.save_graph, { path, graph: graphData });
      } catch (e) {
        error = String(e);
      }
    }
  }

  function setupSvg(rootSelection: d3.Selection<SVGSVGElement, unknown, null, undefined>) {
    const graphLayer = rootSelection.append("g");

    rootSelection.call(d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on("zoom", (event) => {
        graphLayer.attr("transform", event.transform);
      }));

    rootSelection.append("defs").append("marker")
      .attr("id", "arrowhead")
      .attr("viewBox", "-0 -5 10 10")
      .attr("refX", 20)
      .attr("refY", 0)
      .attr("orient", "auto")
      .attr("markerWidth", 8)
      .attr("markerHeight", 8)
      .append("path")
      .attr("d", "M 0,-5 L 10,0 L 0,5")
      .attr("fill", cssVar("--graph-edge"));

    rootSelection.on("click", () => { selectedNode = null; });

    return graphLayer;
  }

  function createEdges(graphLayer: d3.Selection<SVGGElement, unknown, null, undefined>, edges: GraphEdge[]) {
    const links = graphLayer.append("g")
      .attr("class", "links")
      .selectAll("line")
      .data(edges)
      .enter()
      .append("line")
      .attr("stroke", cssVar("--graph-edge"))
      .attr("stroke-opacity", 0.6)
      .attr("stroke-width", edge => Math.sqrt(edge.weight || 1))
      .attr("marker-end", "url(#arrowhead)");

    const labels = graphLayer.append("g")
      .attr("class", "link-labels")
      .selectAll("text")
      .data(edges)
      .enter()
      .append("text")
      .attr("font-size", "10px")
      .attr("fill", cssVar("--graph-edge-label"))
      .attr("text-anchor", "middle")
      .text(edge => edge.label);

    return { links, labels };
  }

  function createNodes(graphLayer: d3.Selection<SVGGElement, unknown, null, undefined>, nodes: GraphNode[]) {
    const groups = graphLayer.append("g")
      .attr("class", "nodes")
      .selectAll("g")
      .data(nodes)
      .enter()
      .append("g")
      .call(d3.drag<SVGGElement, GraphNode>()
        .on("start", dragstarted)
        .on("drag", dragged)
        .on("end", dragended));

    groups.append("circle")
      .attr("r", 16)
      .attr("fill", graphNode => getNodeColor(graphNode))
      .attr("stroke", cssVar("--graph-node-stroke"))
      .attr("stroke-width", 2)
      .style("cursor", "pointer")
      .on("click", (event, graphNode) => {
        event.stopPropagation();
        selectedNode = graphNode;
      });

    groups.append("text")
      .attr("dy", 30)
      .attr("text-anchor", "middle")
      .attr("fill", cssVar("--graph-node-label"))
      .attr("font-size", "12px")
      .text(graphNode => graphNode.label);

    return groups;
  }

  function dragstarted(event: d3.D3DragEvent<SVGGElement, GraphNode, GraphNode>) {
    if (!event.active) simulation!.alphaTarget(0.3).restart();
    event.subject.fx = event.subject.x;
    event.subject.fy = event.subject.y;
  }

  function dragged(event: d3.D3DragEvent<SVGGElement, GraphNode, GraphNode>) {
    event.subject.fx = event.x;
    event.subject.fy = event.y;
  }

  function dragended(event: d3.D3DragEvent<SVGGElement, GraphNode, GraphNode>) {
    if (!event.active) simulation!.alphaTarget(0);
    event.subject.fx = null;
    event.subject.fy = null;
  }

  function renderGraph() {
    if (!graphData || !svgElement) return;

    d3.select(svgElement).selectAll("*").remove();

    const width = svgElement.clientWidth || 1200;
    const height = svgElement.clientHeight || 700;
    const nodes: GraphNode[] = graphData.nodes.map(n => ({ ...n }));
    const edges: GraphEdge[] = graphData.edges.map(e => ({ ...e }));

    const svg = d3.select(svgElement);
    const graphLayer = setupSvg(svg);
    const { links, labels: linkLabels } = createEdges(graphLayer, edges);
    const nodeGroups = createNodes(graphLayer, nodes);

    simulation = d3.forceSimulation<GraphNode>(nodes)
      .force("link", d3.forceLink<GraphNode, GraphEdge>(edges)
        .id(entry => entry.id)
        .distance(120))
      .force("charge", d3.forceManyBody().strength(-400))
      .force("center", d3.forceCenter(width / 2, height / 2))
      .force("collision", d3.forceCollide().radius(40));

    simulation.on("tick", () => {
      links
        .attr("x1", edge => (edge.source as GraphNode).x!)
        .attr("y1", edge => (edge.source as GraphNode).y!)
        .attr("x2", edge => (edge.target as GraphNode).x!)
        .attr("y2", edge => (edge.target as GraphNode).y!);

      linkLabels
        .attr("x", edge => ((edge.source as GraphNode).x! + (edge.target as GraphNode).x!) / 2)
        .attr("y", edge => ((edge.source as GraphNode).y! + (edge.target as GraphNode).y!) / 2);

      nodeGroups.attr("transform", graphNode => `translate(${graphNode.x},${graphNode.y})`);
    });
  }

  function resetZoom() {
    if (!svgElement) return;
    const svg = d3.select(svgElement);
    svg.transition().duration(300).call(
      d3.zoom<SVGSVGElement, unknown>().transform,
      d3.zoomIdentity
    );
  }

  onMount(() => {
    loadSampleGraph();
    loadRootDirectory();

    const handleResize = () => {
      if (graphData) renderGraph();
    };
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  });
</script>

<SidebarLayout
  {showSidebar}
  sidebarTitle="Graph Files"
  onCloseSidebar={() => showSidebar = false}
  fullWidth
>
  {#snippet sidebar()}
    <Breadcrumbs path={directory} onNavigate={navigateToDirectory} />
    {#if treeNodes.length > 0}
      {#each treeNodes as node}
        <TreeNode
          {node}
          selected={selectedFile}
          onToggle={handleTreeToggle}
          onSelect={handleTreeSelect}
          onDblClickDir={handleTreeDblClickDir}
          getIcon={getFileIcon}
        />
      {/each}
    {:else}
      <div class="sidebar-empty">No graph files found</div>
    {/if}
  {/snippet}

  <main>
    <AppHeader appName="RATATOSKR" subtitle="Graph Viewer">
      {#snippet right()}
        <div class="button-row">
          {#if !showSidebar}
            <Button variant="ghost" size="sm" onclick={() => showSidebar = true}>Files</Button>
          {/if}
          <Button size="sm" onclick={loadSampleGraph} disabled={loading}>
            {loading ? "Loading..." : "Sample"}
          </Button>
          <Button size="sm" onclick={loadGraphFromFile} disabled={loading}>
            Open...
          </Button>
          <Button size="sm" onclick={saveGraph} disabled={!graphData || loading}>
            Save
          </Button>
          <Button variant="ghost" size="sm" onclick={resetZoom} disabled={!graphData}>
            Reset Zoom
          </Button>
          <Button variant="ghost" size="sm" onclick={() => showStats = !showStats}>
            {showStats ? "Hide Stats" : "Show Stats"}
          </Button>
        </div>
      {/snippet}
    </AppHeader>

    {#if error}
      <ErrorBanner onDismiss={() => error = ""}>{error}</ErrorBanner>
    {/if}

    <div class="graph-container">
      <svg bind:this={svgElement} class="graph-svg"></svg>

      {#if showStats && stats}
        <aside class="stats-panel">
          <h3>Graph Statistics</h3>
          <dl>
            <dt>Nodes</dt>
            <dd>{stats.node_count}</dd>
            <dt>Edges</dt>
            <dd>{stats.edge_count}</dd>
            <dt>Density</dt>
            <dd>{(stats.density * 100).toFixed(1)}%</dd>
          </dl>

          <h4>Node Types</h4>
          <ul class="type-list">
            {#each Object.entries(stats.node_types) as [type, count]}
              <li>
                <span class="type-dot" style="background: {getTypeColor(type)}"></span>
                {type}: {count}
              </li>
            {/each}
          </ul>

          {#if Object.keys(stats.edge_types).length > 0}
            <h4>Edge Types</h4>
            <ul class="type-list">
              {#each Object.entries(stats.edge_types) as [type, count]}
                <li>{type}: {count}</li>
              {/each}
            </ul>
          {/if}
        </aside>
      {/if}

      {#if selectedNode}
        <aside class="node-panel">
          <h3>Selected Node</h3>
          <dl>
            <dt>ID</dt>
            <dd>{selectedNode.id}</dd>
            <dt>Label</dt>
            <dd>{selectedNode.label}</dd>
            <dt>Type</dt>
            <dd>
              <span class="type-dot" style="background: {getNodeColor(selectedNode)}"></span>
              {selectedNode.node_type || "default"}
            </dd>
          </dl>
          {#if Object.keys(selectedNode.metadata).length > 0}
            <h4>Metadata</h4>
            <pre class="metadata">{JSON.stringify(selectedNode.metadata, null, 2)}</pre>
          {/if}
          <Button variant="ghost" onclick={() => selectedNode = null}>Close</Button>
        </aside>
      {/if}
    </div>

    {#if !graphData && !loading}
      <EmptyState message="Load a graph JSON file or generate a sample graph" />
    {/if}
  </main>
</SidebarLayout>

<style>
  main {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: var(--space-lg);
    overflow: auto;
  }

  .button-row {
    display: flex;
    gap: var(--space-sm);
    flex-wrap: wrap;
    align-items: center;
  }

  .sidebar-empty {
    padding: var(--space-lg);
    color: var(--text-muted);
    font-size: var(--text-sm);
    text-align: center;
  }

  .graph-container {
    flex: 1;
    position: relative;
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    overflow: hidden;
    min-height: 400px;
  }

  .graph-svg {
    width: 100%;
    height: 100%;
    display: block;
  }

  .stats-panel,
  .node-panel {
    position: absolute;
    top: var(--space-lg);
    background: var(--bg-primary);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    padding: var(--space-lg);
    max-width: 250px;
    font-size: var(--text-sm);
  }

  .stats-panel {
    right: var(--space-lg);
  }

  .node-panel {
    left: var(--space-lg);
  }

  .stats-panel h3,
  .node-panel h3 {
    margin: 0 0 var(--space-md);
    font-size: var(--text-base);
    color: var(--text-primary);
  }

  .stats-panel h4,
  .node-panel h4 {
    margin: var(--space-lg) 0 var(--space-sm);
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--space-sm) var(--space-lg);
    margin: 0;
  }

  dt {
    color: var(--text-secondary);
  }

  dd {
    margin: 0;
    color: var(--text-primary);
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .type-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .type-list li {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xs) 0;
    color: var(--text-muted);
  }

  .type-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    display: inline-block;
  }

  .metadata {
    background: var(--bg-secondary);
    padding: var(--space-sm);
    border-radius: var(--radius-sm);
    overflow-x: auto;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    margin: var(--space-sm) 0;
  }

</style>
