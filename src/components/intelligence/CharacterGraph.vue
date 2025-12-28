<script setup lang="ts">
/**
 * CharacterGraph.vue
 *
 * Force-directed graph visualization of character interactions.
 * Styled to match BrutWrite's warm, editorial aesthetic.
 *
 * Features:
 * - Role-based node coloring
 * - Hover tooltips
 * - Zoom & pan controls
 * - Keyboard navigation
 */

import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import * as d3 from 'd3';
import { useCharacterGraph } from '../../composables/domain/intelligence/useCharacterGraph';
import { useProjectStore } from '../../stores/project';
import type { GraphNode } from '../../types/intelligence';

// --- Props & Emits ---

interface Props {
  width?: number;
  height?: number;
}

const props = withDefaults(defineProps<Props>(), {
  width: 800,
  height: 600,
});

const emit = defineEmits<{
  (e: 'nodeSelect', node: GraphNode | null): void;
  (e: 'navigateToMention', chapterId: string): void;
}>();

// --- Composables ---

const { payload, isLoading, error, ghosts, alerts, analyze } = useCharacterGraph();
const projectStore = useProjectStore();

// --- Refs ---

const svgRef = ref<SVGSVGElement | null>(null);
const selectedNodeId = ref<string | null>(null);
const focusedNodeId = ref<string | null>(null);
const liveAnnouncement = ref('');
const tooltipData = ref<{ node: D3Node; x: number; y: number } | null>(null);
const currentZoom = ref(1);
const contextMenuData = ref<{ node: D3Node; x: number; y: number } | null>(null);

// --- Computed ---

const nodes = computed(() => payload.value?.nodes ?? []);
const edges = computed(() => payload.value?.edges ?? []);
const metrics = computed(() => payload.value?.metrics);

const mappedNodes = computed(() => nodes.value.filter((n) => n.isMapped));

// --- Role-based color mapping ---

function getNodeColor(nodeId: string): string {
  const char = projectStore.characterById(nodeId);
  if (!char) return 'var(--ink)';

  switch (char.role) {
    case 'protagonist':
      return 'var(--accent)'; // Orange
    case 'antagonist':
      return '#DC2626'; // Red
    case 'secondary':
      return 'var(--ink)'; // Default black
    case 'extra':
      return 'rgba(26, 26, 26, 0.5)'; // Muted
    default:
      return 'var(--ink)';
  }
}

function getRoleName(nodeId: string): string {
  const char = projectStore.characterById(nodeId);
  if (!char) return 'Unknown';
  return char.role.charAt(0).toUpperCase() + char.role.slice(1);
}

// --- D3 Simulation ---

type D3Node = GraphNode & d3.SimulationNodeDatum;
type D3Link = { source: D3Node; target: D3Node; weight: number; interactionType: string };

let simulation: d3.Simulation<D3Node, D3Link> | null = null;
let zoomBehavior: d3.ZoomBehavior<SVGSVGElement, unknown> | null = null;

function initGraph() {
  if (!svgRef.value || !payload.value) return;

  const svg = d3.select(svgRef.value);
  svg.selectAll('*').remove();

  const { width, height } = props;

  // Prepare node data with D3 simulation properties
  const nodeData: D3Node[] = mappedNodes.value.map((n) => ({
    ...n,
    x: width / 2 + (Math.random() - 0.5) * 100,
    y: height / 2 + (Math.random() - 0.5) * 100,
  }));

  const nodeById = new Map(nodeData.map((n) => [n.id, n]));

  // Prepare link data
  const linkData: D3Link[] = edges.value
    .filter((e) => nodeById.has(e.source) && nodeById.has(e.target))
    .map((e) => ({
      source: nodeById.get(e.source)!,
      target: nodeById.get(e.target)!,
      weight: e.weight,
      interactionType: e.interactionType,
    }));

  // Check for reduced motion preference
  const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  // Create simulation
  simulation = d3
    .forceSimulation<D3Node>(nodeData)
    .force('charge', d3.forceManyBody().strength(-400))
    .force(
      'link',
      d3
        .forceLink<D3Node, D3Link>(linkData)
        .id((d) => d.id)
        .distance((d) => 120 / (d.weight + 0.1))
    )
    .force('center', d3.forceCenter(width / 2, height / 2).strength(0.05))
    .velocityDecay(0.85);

  // Create main group for zoom/pan
  const mainGroup = svg.append('g').attr('class', 'main-group');

  // Setup zoom behavior
  zoomBehavior = d3
    .zoom<SVGSVGElement, unknown>()
    .scaleExtent([0.5, 3])
    .on('zoom', (event) => {
      mainGroup.attr('transform', event.transform);
      currentZoom.value = event.transform.k;
    });

  svg.call(zoomBehavior);

  // Create SVG groups inside main group
  const linksGroup = mainGroup.append('g').attr('class', 'links-group');
  const nodesGroup = mainGroup.append('g').attr('class', 'nodes-group');
  const labelsGroup = mainGroup.append('g').attr('class', 'labels-group');

  // Draw links - using warm ink color
  const links = linksGroup
    .selectAll('line')
    .data(linkData)
    .join('line')
    .attr('class', 'graph-link')
    .attr('stroke', 'rgba(26, 26, 26, 0.15)')
    .attr('stroke-width', (d) => Math.min(d.weight * 0.5 + 1, 4))
    .attr('stroke-dasharray', (d) => (d.interactionType === 'reference' ? '4,4' : 'none'));

  // Calculate node radius based on valence
  const maxValence = Math.max(...nodeData.map((n) => n.valence), 1);
  const radiusScale = d3.scaleLinear().domain([0, maxValence]).range([10, 28]);

  // Draw nodes with role-based coloring
  const nodeElements = nodesGroup
    .selectAll('circle')
    .data(nodeData)
    .join('circle')
    .attr('class', 'graph-node')
    .attr('r', (d) => radiusScale(d.valence))
    .attr('fill', (d) => getNodeColor(d.id))
    .attr('stroke', 'var(--paper)')
    .attr('stroke-width', 2)
    .attr('cursor', 'pointer')
    .attr('tabindex', 0)
    .attr('role', 'button')
    .attr('aria-label', (d) => `${d.label}, ${d.mentionCount} mentions`)
    .on('click', (_event, d) => handleNodeClick(d))
    .on('dblclick', (_event, d) => handleNodeDoubleClick(d))
    .on('contextmenu', (event, d) => handleNodeContextMenu(event, d))
    .on('focus', (_event, d) => handleNodeFocus(d))
    .on('blur', () => handleNodeBlur())
    .on('keydown', (event, d) => handleNodeKeydown(event, d, nodeData, linkData))
    .on('mouseenter', (event, d) => showTooltip(d, event))
    .on('mouseleave', () => hideTooltip());

  // Draw labels - serif font matching project style
  const labels = labelsGroup
    .selectAll('text')
    .data(nodeData)
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

  // Handle reduced motion
  if (prefersReducedMotion) {
    simulation.stop();
    for (let i = 0; i < 300; i++) simulation.tick();
    updatePositions();
  } else {
    simulation.on('tick', updatePositions);
  }

  function updatePositions() {
    nodeElements.attr('cx', (d) => d.x ?? 0).attr('cy', (d) => d.y ?? 0);

    labels.attr('x', (d) => d.x ?? 0).attr('y', (d) => (d.y ?? 0) + radiusScale(d.valence) + 16);

    links
      .attr('x1', (d) => d.source.x ?? 0)
      .attr('y1', (d) => d.source.y ?? 0)
      .attr('x2', (d) => d.target.x ?? 0)
      .attr('y2', (d) => d.target.y ?? 0);
  }

  // Selection highlighting
  watch(
    selectedNodeId,
    (newId) => {
      nodeElements
        .attr('fill', (d) => (d.id === newId ? 'var(--accent)' : getNodeColor(d.id)))
        .attr('stroke', (d) => (d.id === newId ? 'var(--paper)' : 'var(--paper)'))
        .attr('stroke-width', (d) => (d.id === newId ? 3 : 2));

      // Dim non-connected nodes when one is selected
      if (newId) {
        const connectedIds = new Set<string>([newId]);
        linkData.forEach((l) => {
          if (l.source.id === newId) connectedIds.add(l.target.id);
          if (l.target.id === newId) connectedIds.add(l.source.id);
        });

        nodeElements.attr('opacity', (d) => (connectedIds.has(d.id) ? 1 : 0.15));
        links.attr('opacity', (l) => (l.source.id === newId || l.target.id === newId ? 1 : 0.1));
        labels.attr('opacity', (d) => (connectedIds.has(d.id) ? 1 : 0.15));
      } else {
        nodeElements.attr('opacity', 1);
        links.attr('opacity', 1);
        labels.attr('opacity', 1);
      }
    },
    { immediate: true }
  );
}

// --- Tooltip ---

function showTooltip(node: D3Node, event: MouseEvent) {
  const svgRect = svgRef.value?.getBoundingClientRect();
  if (!svgRect) return;

  tooltipData.value = {
    node,
    x: event.clientX - svgRect.left,
    y: event.clientY - svgRect.top - 10,
  };
}

function hideTooltip() {
  tooltipData.value = null;
}

function getConnectionCount(nodeId: string): number {
  return edges.value.filter((e) => e.source === nodeId || e.target === nodeId).length;
}

// --- Context Menu ---

function handleNodeContextMenu(event: MouseEvent, node: D3Node) {
  event.preventDefault();
  contextMenuData.value = {
    node,
    x: event.clientX,
    y: event.clientY,
  };
}

function closeContextMenu() {
  contextMenuData.value = null;
}

async function copyCharacterTag() {
  if (!contextMenuData.value) return;
  const tag = `@${contextMenuData.value.node.label}`;
  try {
    await navigator.clipboard.writeText(tag);
    liveAnnouncement.value = `Copied ${tag} to clipboard`;
  } catch {
    liveAnnouncement.value = 'Failed to copy to clipboard';
  }
  closeContextMenu();
}

// --- Zoom Controls ---

function zoomIn() {
  if (!svgRef.value || !zoomBehavior) return;
  const svg = d3.select(svgRef.value);
  svg.transition().duration(300).call(zoomBehavior.scaleBy, 1.3);
}

function zoomOut() {
  if (!svgRef.value || !zoomBehavior) return;
  const svg = d3.select(svgRef.value);
  svg.transition().duration(300).call(zoomBehavior.scaleBy, 0.7);
}

function resetZoom() {
  if (!svgRef.value || !zoomBehavior) return;
  const svg = d3.select(svgRef.value);
  svg.transition().duration(300).call(zoomBehavior.transform, d3.zoomIdentity);
}

// --- Event Handlers ---

function handleNodeClick(node: D3Node) {
  if (selectedNodeId.value === node.id) {
    selectedNodeId.value = null;
    emit('nodeSelect', null);
  } else {
    selectedNodeId.value = node.id;
    emit('nodeSelect', node);
  }
}

function handleNodeDoubleClick(node: D3Node) {
  if (node.firstMention) {
    emit('navigateToMention', node.firstMention.chapterId);
  }
}

function handleNodeFocus(node: D3Node) {
  focusedNodeId.value = node.id;
  liveAnnouncement.value = `${node.label}. ${node.mentionCount} mentions. ${getConnectionCount(node.id)} connections.`;
}

function handleNodeBlur() {
  focusedNodeId.value = null;
}

function handleNodeKeydown(
  event: KeyboardEvent,
  node: D3Node,
  _nodeData: D3Node[],
  linkData: D3Link[]
) {
  if (event.key === 'Enter' || event.key === ' ') {
    event.preventDefault();
    handleNodeClick(node);
    return;
  }

  // Arrow key navigation to adjacent nodes
  if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(event.key)) {
    event.preventDefault();

    // Find connected nodes
    const connected = linkData
      .filter((l) => l.source.id === node.id || l.target.id === node.id)
      .map((l) => (l.source.id === node.id ? l.target : l.source));

    if (connected.length === 0) return;

    // Simple navigation: cycle through connected nodes
    const currentIdx = connected.findIndex((n) => n.id === focusedNodeId.value);
    let nextIdx = 0;
    if (event.key === 'ArrowRight' || event.key === 'ArrowDown') {
      nextIdx = (currentIdx + 1) % connected.length;
    } else {
      nextIdx = (currentIdx - 1 + connected.length) % connected.length;
    }

    // Focus the next node
    const nextNode = connected[nextIdx];
    const nodeElement = d3
      .select(svgRef.value)
      .selectAll<SVGCircleElement, D3Node>('.graph-node')
      .filter((d) => d.id === nextNode.id)
      .node();

    if (nodeElement) {
      nodeElement.focus();
    }
  }
}

// --- Lifecycle ---

onMounted(async () => {
  await analyze();
  await nextTick();
  initGraph();
});

onUnmounted(() => {
  if (simulation) {
    simulation.stop();
    simulation = null;
  }
});

watch(payload, () => {
  nextTick(() => initGraph());
});
</script>

<template>
  <div class="graph-container">
    <!-- Loading State -->
    <div v-if="isLoading" class="loading-overlay">
      <div class="flex flex-col items-center gap-3">
        <div class="w-8 h-8 border-2 border-ink/20 border-t-accent rounded-full animate-spin"></div>
        <span class="text-xs uppercase tracking-widest text-ink/40 font-bold">Analyzing...</span>
      </div>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="error-overlay">
      <div class="text-center">
        <div
          class="w-12 h-12 rounded-full bg-red-100 flex items-center justify-center mx-auto mb-3"
        >
          <svg class="w-6 h-6 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
            />
          </svg>
        </div>
        <span class="text-sm text-red-600 font-medium">{{ error }}</span>
      </div>
    </div>

    <!-- Graph SVG -->
    <svg
      v-else
      ref="svgRef"
      :viewBox="`0 0 ${props.width} ${props.height}`"
      class="graph-svg"
      role="img"
      aria-label="Character interaction graph"
    />

    <!-- Tooltip -->
    <div
      v-if="tooltipData"
      class="tooltip"
      :style="{ left: `${tooltipData.x}px`, top: `${tooltipData.y}px` }"
    >
      <div class="tooltip-name">{{ tooltipData.node.label }}</div>
      <div class="tooltip-role">{{ getRoleName(tooltipData.node.id) }}</div>
      <div class="tooltip-stats">
        <span>{{ tooltipData.node.mentionCount }} mentions</span>
        <span>•</span>
        <span>{{ getConnectionCount(tooltipData.node.id) }} connections</span>
      </div>
      <div class="tooltip-valence">Valence: {{ tooltipData.node.valence.toFixed(2) }}</div>
    </div>

    <!-- Context Menu -->
    <Teleport to="body">
      <div
        v-if="contextMenuData"
        class="fixed inset-0 z-90"
        @click="closeContextMenu"
        @contextmenu.prevent="closeContextMenu"
      ></div>
      <div
        v-if="contextMenuData"
        class="context-menu"
        :style="{ left: `${contextMenuData.x}px`, top: `${contextMenuData.y}px` }"
      >
        <button class="context-menu-item" @click="copyCharacterTag">
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002 2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2 2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3"
            />
          </svg>
          Copy @{{ contextMenuData.node.label }}
        </button>
      </div>
    </Teleport>

    <!-- Zoom Controls -->
    <div class="zoom-controls">
      <button class="zoom-btn" title="Zoom in" @click="zoomIn">
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 4v16m8-8H4"
          />
        </svg>
      </button>
      <span class="zoom-level">{{ Math.round(currentZoom * 100) }}%</span>
      <button class="zoom-btn" title="Zoom out" @click="zoomOut">
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
        </svg>
      </button>
      <button class="zoom-btn" title="Reset zoom" @click="resetZoom">
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
          />
        </svg>
      </button>
    </div>

    <!-- Ghost Panel - styled to match CharacterDetail cards -->
    <aside v-if="ghosts.length > 0" class="ghost-panel">
      <div class="flex items-center gap-2 mb-3">
        <h4 class="text-xs uppercase tracking-widest text-purple-700/60 font-bold">
          Unmapped Characters
        </h4>
        <div class="h-px flex-1 bg-purple-500/10"></div>
      </div>
      <ul class="space-y-1.5">
        <li
          v-for="ghost in ghosts"
          :key="ghost.id"
          class="text-sm text-ink/60 font-medium flex items-center gap-2"
        >
          <span class="w-1.5 h-1.5 rounded-full bg-purple-400/50"></span>
          {{ ghost.label }}
        </li>
      </ul>
    </aside>

    <!-- Role Legend -->
    <div class="role-legend">
      <div class="legend-item">
        <span class="legend-dot" style="background-color: var(--accent)"></span>
        <span>Protagonist</span>
      </div>
      <div class="legend-item">
        <span class="legend-dot" style="background-color: #dc2626"></span>
        <span>Antagonist</span>
      </div>
      <div class="legend-item">
        <span class="legend-dot" style="background-color: var(--ink)"></span>
        <span>Secondary</span>
      </div>
      <div class="legend-item">
        <span class="legend-dot" style="background-color: rgba(26, 26, 26, 0.5)"></span>
        <span>Extra</span>
      </div>
    </div>

    <!-- Metrics HUD - styled to match project aesthetic -->
    <footer v-if="metrics" class="metrics-hud">
      <div class="metrics-stats">
        <div class="metric-item">
          <span class="metric-label">Density</span>
          <span class="metric-value">{{ (metrics.networkDensity * 100).toFixed(0) }}%</span>
        </div>
        <div class="metric-divider"></div>
        <div class="metric-item">
          <span class="metric-label">Components</span>
          <span class="metric-value">{{ metrics.connectedComponents }}</span>
        </div>
        <div class="metric-divider"></div>
        <div class="metric-item">
          <span class="metric-label">Isolated</span>
          <span class="metric-value">{{ (metrics.isolationRatio * 100).toFixed(0) }}%</span>
        </div>
      </div>

      <!-- Alert Badges - styled like role buttons -->
      <div v-if="alerts.length > 0" class="metrics-alerts">
        <span v-for="alert in alerts" :key="alert.code" class="alert-badge" :title="alert.tooltip">
          {{ alert.primaryText }}
        </span>
      </div>
    </footer>

    <!-- Screen Reader Live Region -->
    <div aria-live="polite" class="sr-only">{{ liveAnnouncement }}</div>
  </div>
</template>

<style scoped>
.graph-container {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 400px;
  background-color: transparent;
  overflow: hidden;
}

.graph-svg {
  width: 100%;
  height: 100%;
  cursor: grab;
}

.graph-svg:active {
  cursor: grabbing;
}

.loading-overlay,
.error-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--paper);
}

/* Tooltip */
.tooltip {
  position: absolute;
  transform: translate(-50%, -100%);
  padding: 0.75rem 1rem;
  background: var(--paper);
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 0.75rem;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  pointer-events: none;
  z-index: 100;
  min-width: 150px;
}

.tooltip-name {
  font-family: 'Playfair Display', serif;
  font-size: 0.875rem;
  font-weight: 600;
  font-style: italic;
  color: var(--ink);
  margin-bottom: 2px;
}

.tooltip-role {
  font-size: 0.625rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--accent);
  margin-bottom: 8px;
}

.tooltip-stats {
  display: flex;
  gap: 0.5rem;
  font-size: 0.75rem;
  color: rgba(26, 26, 26, 0.6);
}

.tooltip-valence {
  font-size: 0.625rem;
  color: rgba(26, 26, 26, 0.4);
  margin-top: 6px;
}

/* Context Menu */
.context-menu {
  position: fixed;
  z-index: 9999;
  padding: 0.5rem;
  background: var(--paper);
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 0.75rem;
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.15);
  min-width: 180px;
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem 0.75rem;
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--ink);
  border-radius: 0.5rem;
  transition: all 0.15s ease;
  text-align: left;
}

.context-menu-item:hover {
  background-color: var(--accent);
  color: white;
}

/* Zoom Controls */
.zoom-controls {
  position: absolute;
  top: 1rem;
  left: 1rem;
  display: flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.25rem;
  background: var(--paper);
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 0.5rem;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.zoom-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 0.375rem;
  color: rgba(26, 26, 26, 0.6);
  transition: all 0.15s ease;
}

.zoom-btn:hover {
  background-color: rgba(0, 0, 0, 0.05);
  color: var(--ink);
}

.zoom-level {
  font-size: 0.625rem;
  font-weight: 700;
  color: rgba(26, 26, 26, 0.4);
  min-width: 36px;
  text-align: center;
}

/* Role Legend */
.role-legend {
  position: absolute;
  bottom: 3.5rem;
  left: 1rem;
  display: flex;
  gap: 1rem;
  padding: 0.5rem 0.75rem;
  background: rgba(255, 255, 255, 0.8);
  border: 1px solid rgba(0, 0, 0, 0.05);
  border-radius: 0.5rem;
  backdrop-filter: blur(4px);
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.625rem;
  font-weight: 600;
  color: rgba(26, 26, 26, 0.6);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.legend-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

/* Ghost Panel - matches CharacterDetail card styling */
.ghost-panel {
  position: absolute;
  top: 1rem;
  right: 1rem;
  max-width: 200px;
  padding: 1rem 1.25rem;
  background: linear-gradient(to bottom right, rgba(168, 85, 247, 0.05), transparent);
  border: 1px solid rgba(168, 85, 247, 0.1);
  border-radius: 1rem;
  backdrop-filter: blur(8px);
}

/* Metrics HUD - refined to match project style */
.metrics-hud {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 1.5rem;
  background: linear-gradient(to top, var(--paper), transparent);
  border-top: 1px solid rgba(0, 0, 0, 0.05);
}

.metrics-stats {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.metric-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.metric-label {
  font-size: 0.625rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: rgba(26, 26, 26, 0.4);
}

.metric-value {
  font-size: 0.875rem;
  font-weight: 600;
  font-family: 'Playfair Display', serif;
  color: var(--ink);
}

.metric-divider {
  width: 1px;
  height: 24px;
  background-color: rgba(0, 0, 0, 0.08);
}

.metrics-alerts {
  display: flex;
  gap: 0.5rem;
}

.alert-badge {
  padding: 0.375rem 0.75rem;
  font-size: 0.625rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--paper);
  background-color: var(--accent);
  border-radius: 0.5rem;
  cursor: help;
  transition: all 0.2s ease;
}

.alert-badge:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(255, 95, 31, 0.3);
}

/* Accessibility: Screen reader only */
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

/* D3 Graph Styling */
:deep(.graph-node) {
  transition:
    fill 0.2s ease,
    opacity 0.2s ease,
    stroke-width 0.2s ease;
}

:deep(.graph-node:focus) {
  outline: none;
  stroke: var(--accent);
  stroke-width: 3px;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%,
  100% {
    stroke-opacity: 1;
  }
  50% {
    stroke-opacity: 0.5;
  }
}

:deep(.graph-link) {
  transition: opacity 0.2s ease;
}

:deep(.graph-label) {
  pointer-events: none;
  transition: opacity 0.2s ease;
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {
  :deep(.graph-node:focus) {
    animation: none;
  }
}
</style>
