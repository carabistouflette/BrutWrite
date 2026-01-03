import * as d3 from 'd3';
import type { GraphNode, GraphEdge } from '../../../types/intelligence';

// =============================================================================
// Graph Simulation Constants
// =============================================================================

/** Configuration for D3 force simulation parameters */
const GRAPH_CONFIG = {
  /** Repulsion force between nodes (negative = repel) */
  CHARGE_STRENGTH: -400,
  /** Base distance for links before weight adjustment */
  LINK_BASE_DISTANCE: 120,
  /** Offset to prevent division by zero in link distance */
  LINK_WEIGHT_OFFSET: 0.1,
  /** Gravity strength toward center */
  CENTER_STRENGTH: 0.05,
  /** Simulation damping factor (0-1, higher = slower) */
  VELOCITY_DECAY: 0.85,
  /** Ticks for static layout in reduced-motion mode */
  STATIC_TICKS: 300,
} as const;

// Extended types for D3
export type D3Node = GraphNode & d3.SimulationNodeDatum;
export type D3Link = d3.SimulationLinkDatum<D3Node> & {
  weight: number;
  interactionType: string;
};

export interface GraphEngineOptions {
  width: number;
  height: number;
  onNodeClick?: (node: D3Node) => void;
  onNodeDoubleClick?: (node: D3Node) => void;
  onNodeContextMenu?: (event: MouseEvent, node: D3Node) => void;
  onNodeHover?: (event: MouseEvent, node: D3Node) => void;
  onNodeBlur?: () => void;
  onNodeFocus?: (node: D3Node) => void;
  getNodeColor: (id: string) => string;
}

export class CharacterGraphEngine {
  private svg: d3.Selection<SVGSVGElement, unknown, null, undefined>;
  private simulation: d3.Simulation<D3Node, D3Link> | null = null;
  private zoomBehavior: d3.ZoomBehavior<SVGSVGElement, unknown> | null = null;

  private width: number;
  private height: number;
  private options: GraphEngineOptions;
  private radiusScale: d3.ScaleLinear<number, number> | null = null;

  // Elements
  private mainGroup: d3.Selection<SVGGElement, unknown, null, undefined> | null = null;
  private linkSelection: d3.Selection<SVGLineElement, D3Link, SVGGElement, unknown> | null = null;
  private nodeSelection: d3.Selection<SVGCircleElement, D3Node, SVGGElement, unknown> | null = null;
  private labelSelection: d3.Selection<SVGTextElement, D3Node, SVGGElement, unknown> | null = null;

  public onZoom?: (k: number) => void;

  constructor(
    svgElement: SVGSVGElement,
    options: GraphEngineOptions & { onZoom?: (k: number) => void }
  ) {
    this.svg = d3.select(svgElement);
    this.width = options.width;
    this.height = options.height;
    this.options = options;
    this.onZoom = options.onZoom;
  }

  /**
   * Initialize or update the graph with new data.
   * Uses D3's data-join to minimize DOM churn and preserve state.
   */
  public update(nodes: GraphNode[], edges: GraphEdge[]) {
    // 1. Prepare Data
    const existingNodes = this.nodeSelection
      ? new Map(this.nodeSelection.data().map((n) => [n.id, n]))
      : new Map<string, D3Node>();

    const d3Nodes: D3Node[] = nodes.map((n) => {
      const existing = existingNodes.get(n.id);
      if (existing) {
        // Carry over position and velocity
        return { ...existing, ...n };
      }

      // New node: Deterministic initial position based on ID if possible
      // This prevents the graph from "jumping" randomly on every load
      const hash = n.id.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0);
      return {
        ...n,
        x: this.width / 2 + ((hash % 100) - 50),
        y: this.height / 2 + ((hash % 70) - 35),
      };
    });

    const nodeById = new Map(d3Nodes.map((n) => [n.id, n]));

    const d3Links: D3Link[] = edges
      .filter((e) => nodeById.has(e.source) && nodeById.has(e.target))
      .map((e) => ({
        source: nodeById.get(e.source)!,
        target: nodeById.get(e.target)!,
        weight: e.weight,
        interactionType: e.interactionType,
      }));

    // 2. Update Simulation
    if (!this.simulation) {
      this.setupSimulation(d3Nodes, d3Links);
    } else {
      // Update nodes and links in existing simulation
      this.simulation.nodes(d3Nodes);

      const linkForce = this.simulation.force<d3.ForceLink<D3Node, D3Link>>('link');
      if (linkForce) {
        linkForce.links(d3Links);
      }

      // Re-heat simulation slightly to settle new structure
      this.simulation.alpha(0.3).restart();
    }

    // 3. Update Visuals
    this.renderGraph(d3Nodes, d3Links);

    // Reset highlights on data update to avoid stale visual states
    this.highlightNode(null);
  }

  private setupSimulation(nodes: D3Node[], links: D3Link[]) {
    this.simulation = d3
      .forceSimulation<D3Node>(nodes)
      .force('charge', d3.forceManyBody().strength(GRAPH_CONFIG.CHARGE_STRENGTH))
      .force(
        'link',
        d3
          .forceLink<D3Node, D3Link>(links)
          .id((d) => d.id)
          .distance(
            (d) => GRAPH_CONFIG.LINK_BASE_DISTANCE / (d.weight + GRAPH_CONFIG.LINK_WEIGHT_OFFSET)
          )
      )
      .force(
        'center',
        d3.forceCenter(this.width / 2, this.height / 2).strength(GRAPH_CONFIG.CENTER_STRENGTH)
      )
      .velocityDecay(GRAPH_CONFIG.VELOCITY_DECAY);

    // Reduced motion check
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
      this.simulation.stop();
      // Run simulation synchronously to static state
      this.simulation.tick(GRAPH_CONFIG.STATIC_TICKS);
      this.updatePositions();
    } else {
      this.simulation.on('tick', () => this.updatePositions());
    }
  }

  private renderGraph(nodes: D3Node[], links: D3Link[]) {
    if (!this.mainGroup) {
      this.mainGroup = this.svg.append('g').attr('class', 'main-group');
      this.mainGroup.append('g').attr('class', 'links-group');
      this.mainGroup.append('g').attr('class', 'nodes-group');
      this.mainGroup.append('g').attr('class', 'labels-group');

      this.zoomBehavior = d3
        .zoom<SVGSVGElement, unknown>()
        .scaleExtent([0.5, 3])
        .on('zoom', (event) => {
          this.mainGroup?.attr('transform', event.transform);
          this.onZoom?.(event.transform.k);
        });
      this.svg.call(this.zoomBehavior);
    }

    const linksGroup = this.mainGroup.select<SVGGElement>('.links-group');
    const nodesGroup = this.mainGroup.select<SVGGElement>('.nodes-group');
    const labelsGroup = this.mainGroup.select<SVGGElement>('.labels-group');

    // Links
    this.linkSelection = linksGroup
      .selectAll<SVGLineElement, D3Link>('line')
      .data(links, (d) => `${(d.source as D3Node).id}-${(d.target as D3Node).id}`)
      .join('line')
      .attr('class', 'graph-link')
      .attr('stroke', 'rgba(26, 26, 26, 0.15)')
      .attr('stroke-width', (d) => Math.min(d.weight * 0.5 + 1, 4))
      .attr('stroke-dasharray', (d) => (d.interactionType === 'reference' ? '4,4' : 'none'));

    // Node Radius
    const maxValence = Math.max(...nodes.map((n) => n.valence), 1);
    this.radiusScale = d3.scaleLinear().domain([0, maxValence]).range([10, 28]);

    // Nodes
    this.nodeSelection = nodesGroup
      .selectAll<SVGCircleElement, D3Node>('circle')
      .data(nodes, (d) => d.id)
      .join('circle')
      .attr('class', 'graph-node')
      .attr('r', (d) => this.radiusScale!(d.valence))
      .attr('fill', (d) => this.options.getNodeColor(d.id))
      .attr('stroke', 'var(--paper)')
      .attr('stroke-width', 2)
      .attr('cursor', 'pointer')
      .attr('tabindex', 0)
      .attr('role', 'button')
      .attr('aria-label', (d) => `${d.label}, ${d.mentionCount} mentions`)
      .on('click', (_e, d) => this.options.onNodeClick?.(d))
      .on('dblclick', (_e, d) => this.options.onNodeDoubleClick?.(d))
      .on('contextmenu', (e, d) => this.options.onNodeContextMenu?.(e, d))
      .on('focus', (_e, d) => this.options.onNodeFocus?.(d))
      .on('blur', () => this.options.onNodeBlur?.())
      .on('mouseenter', (e, d) => this.options.onNodeHover?.(e, d))
      .on('mouseleave', () => this.options.onNodeBlur?.())
      .on('keydown', (e, d) => this.handleNodeKeydown(e, d));

    // Labels
    this.labelSelection = labelsGroup
      .selectAll<SVGTextElement, D3Node>('text')
      .data(nodes, (d) => d.id)
      .join('text')
      .attr('class', 'graph-label')
      .attr('font-family', 'Playfair Display, Georgia, serif')
      .attr('font-size', '12px')
      .attr('font-weight', '500')
      .attr('font-style', 'italic')
      .attr('fill', 'var(--ink)')
      .attr('text-anchor', 'middle')
      .attr('dominant-baseline', 'middle')
      .text((d) => d.label);

    // Initial positioning
    this.updatePositions();
  }

  private handleNodeKeydown(event: KeyboardEvent, node: D3Node) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      this.options.onNodeClick?.(node);
      return;
    }

    // Arrow key navigation to adjacent nodes
    if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(event.key)) {
      event.preventDefault();

      if (!this.linkSelection || !this.nodeSelection) return;

      const links = this.linkSelection.data();
      // Find connected nodes
      const connected = links
        .filter((l) => (l.source as D3Node).id === node.id || (l.target as D3Node).id === node.id)
        .map((l) => ((l.source as D3Node).id === node.id ? l.target : l.source) as D3Node);

      if (connected.length === 0) return;

      // Deterministic sort for consistent cycling
      connected.sort((a, b) => a.id.localeCompare(b.id));

      // Just pick the first connected node to jump to
      const nextNode = connected[0];

      // Focus element
      const el = this.nodeSelection.filter((d) => d.id === nextNode.id).node();
      el?.focus();
    }
  }

  private updatePositions() {
    if (!this.nodeSelection || !this.linkSelection || !this.labelSelection || !this.radiusScale)
      return;

    this.nodeSelection.attr('cx', (d) => d.x ?? 0).attr('cy', (d) => d.y ?? 0);

    this.labelSelection
      .attr('x', (d) => d.x ?? 0)
      .attr('y', (d) => (d.y ?? 0) + this.radiusScale!(d.valence) + 16);

    this.linkSelection
      .attr('x1', (d) => (d.source as D3Node).x ?? 0)
      .attr('y1', (d) => (d.source as D3Node).y ?? 0)
      .attr('x2', (d) => (d.target as D3Node).x ?? 0)
      .attr('y2', (d) => (d.target as D3Node).y ?? 0);
  }

  public zoomIn() {
    if (!this.zoomBehavior) return;
    this.svg.transition().duration(300).call(this.zoomBehavior.scaleBy, 1.3);
  }

  public zoomOut() {
    if (!this.zoomBehavior) return;
    this.svg.transition().duration(300).call(this.zoomBehavior.scaleBy, 0.7);
  }

  public resetZoom() {
    if (!this.zoomBehavior) return;
    this.svg.transition().duration(300).call(this.zoomBehavior.transform, d3.zoomIdentity);
  }

  public highlightNode(nodeId: string | null) {
    if (!this.nodeSelection || !this.linkSelection || !this.labelSelection) return;

    if (nodeId) {
      // Recalculate basic styling
      this.nodeSelection
        .attr('fill', (d) => (d.id === nodeId ? 'var(--accent)' : this.options.getNodeColor(d.id)))
        .attr('stroke', (d) => (d.id === nodeId ? 'var(--paper)' : 'var(--paper)'))
        .attr('stroke-width', (d) => (d.id === nodeId ? 3 : 2));

      // Dimming
      const connectedIds = new Set<string>([nodeId]);
      const links = this.linkSelection.data();
      links.forEach((l) => {
        const s = l.source as D3Node;
        const t = l.target as D3Node;
        if (s.id === nodeId) connectedIds.add(t.id);
        if (t.id === nodeId) connectedIds.add(s.id);
      });

      this.nodeSelection.attr('opacity', (d) => (connectedIds.has(d.id) ? 1 : 0.15));

      this.linkSelection.attr('opacity', (l) => {
        const s = l.source as D3Node;
        const t = l.target as D3Node;
        return s.id === nodeId || t.id === nodeId ? 1 : 0.1;
      });

      this.labelSelection.attr('opacity', (d) => (connectedIds.has(d.id) ? 1 : 0.15));
    } else {
      // Reset
      this.nodeSelection
        .attr('fill', (d) => this.options.getNodeColor(d.id))
        .attr('stroke', 'var(--paper)')
        .attr('stroke-width', 2)
        .attr('opacity', 1);

      this.linkSelection.attr('opacity', 1);
      this.labelSelection.attr('opacity', 1);
    }
  }

  public dispose() {
    // Stop simulation first
    if (this.simulation) {
      this.simulation.stop();
      this.simulation = null;
    }

    // Remove zoom behavior and its event listeners
    if (this.zoomBehavior) {
      this.svg.on('.zoom', null);
      this.zoomBehavior = null;
    }

    // Clean up node event listeners
    if (this.nodeSelection) {
      this.nodeSelection
        .on('click', null)
        .on('dblclick', null)
        .on('contextmenu', null)
        .on('focus', null)
        .on('blur', null)
        .on('mouseenter', null)
        .on('mouseleave', null)
        .on('keydown', null);
      this.nodeSelection = null;
    }

    // Clean up link selection
    if (this.linkSelection) {
      this.linkSelection = null;
    }

    // Clean up label selection
    if (this.labelSelection) {
      this.labelSelection = null;
    }

    // Clear all SVG content
    this.svg.selectAll('*').remove();
    this.mainGroup = null;

    // Clear callback reference
    this.onZoom = undefined;
  }
}
