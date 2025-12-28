<script setup lang="ts">
/**
 * CharacterGraph.vue
 *
 * Force-directed graph visualization of character interactions.
 * Styled to match BrutWrite's warm, editorial aesthetic.
 */

import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import * as d3 from 'd3';
import { useCharacterGraph } from '../../composables/domain/intelligence/useCharacterGraph';
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
}>();

// --- Composables ---

const { payload, isLoading, error, ghosts, alerts, analyze } = useCharacterGraph();

// --- Refs ---

const svgRef = ref<SVGSVGElement | null>(null);
const selectedNodeId = ref<string | null>(null);
const focusedNodeId = ref<string | null>(null);
const liveAnnouncement = ref('');

// --- Computed ---

const nodes = computed(() => payload.value?.nodes ?? []);
const edges = computed(() => payload.value?.edges ?? []);
const metrics = computed(() => payload.value?.metrics);

const mappedNodes = computed(() => nodes.value.filter((n) => n.isMapped));

// --- D3 Simulation ---

type D3Node = GraphNode & d3.SimulationNodeDatum;
type D3Link = { source: D3Node; target: D3Node; weight: number; interactionType: string };

let simulation: d3.Simulation<D3Node, D3Link> | null = null;

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

  // Create SVG groups
  const linksGroup = svg.append('g').attr('class', 'links-group');
  const nodesGroup = svg.append('g').attr('class', 'nodes-group');
  const labelsGroup = svg.append('g').attr('class', 'labels-group');

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

  // Draw nodes - warm ink color with paper fill
  const nodeElements = nodesGroup
    .selectAll('circle')
    .data(nodeData)
    .join('circle')
    .attr('class', 'graph-node')
    .attr('r', (d) => radiusScale(d.valence))
    .attr('fill', 'var(--ink)')
    .attr('stroke', 'var(--paper)')
    .attr('stroke-width', 2)
    .attr('cursor', 'pointer')
    .attr('tabindex', 0)
    .attr('role', 'button')
    .attr('aria-label', (d) => `${d.label}, ${d.mentionCount} mentions`)
    .on('click', (_event, d) => handleNodeClick(d))
    .on('focus', (_event, d) => handleNodeFocus(d))
    .on('blur', () => handleNodeBlur())
    .on('keydown', (event, d) => handleNodeKeydown(event, d, nodeData, linkData));

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
        .attr('fill', (d) => (d.id === newId ? 'var(--accent)' : 'var(--ink)'))
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

function getConnectionCount(nodeId: string): number {
  return edges.value.filter((e) => e.source === nodeId || e.target === nodeId).length;
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
