<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { storeToRefs } from 'pinia';
import { useCharacterGraph } from '../../composables/domain/intelligence/useCharacterGraph';
import { useProjectStore } from '../../stores/project';
import { CharacterGraphEngine, type D3Node } from '../../utils/graphs/CharacterGraphEngine';
import type { GraphNode } from '../../types/intelligence';

// Sub-components
import GraphTooltip from './graph/GraphTooltip.vue';
import GraphContextMenu from './graph/GraphContextMenu.vue';
import GraphControls from './graph/GraphControls.vue';
import GraphMetrics from './graph/GraphMetrics.vue';
import GraphLegend from './graph/GraphLegend.vue';
import GraphGhosts from './graph/GraphGhosts.vue';

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

const graphStore = useCharacterGraph();
const { state, ghosts, alerts } = storeToRefs(graphStore);
const { analyze } = graphStore;

const payload = computed(() => state.value.payload);
const isLoading = computed(() => state.value.isLoading);
const error = computed(() => state.value.error);
const projectStore = useProjectStore();

// --- Refs ---

const svgRef = ref<SVGSVGElement | null>(null);
const selectedNodeId = ref<string | null>(null);
const liveAnnouncement = ref('');
const tooltipData = ref<{ node: D3Node; x: number; y: number } | null>(null);
const contextMenuData = ref<{ node: D3Node; x: number; y: number } | null>(null);

let engine: CharacterGraphEngine | null = null;

// --- Computed ---

const metrics = computed(() => payload.value?.metrics);

// --- Helpers ---

import { GRAPH_CONSTANTS } from '../../config/uiConstants';

// ... (keep existing imports)

// --- Helpers ---

function getNodeColor(nodeId: string): string {
  const char = projectStore.characterById(nodeId);
  if (!char) return GRAPH_CONSTANTS.THEME.ROLE_COLORS.DEFAULT;

  switch (char.role) {
    case 'protagonist':
      return GRAPH_CONSTANTS.THEME.ROLE_COLORS.PROTAGONIST;
    case 'antagonist':
      return GRAPH_CONSTANTS.THEME.ROLE_COLORS.ANTAGONIST;
    case 'secondary':
      return GRAPH_CONSTANTS.THEME.ROLE_COLORS.SECONDARY;
    case 'extra':
      return GRAPH_CONSTANTS.THEME.ROLE_COLORS.EXTRA;
    default:
      return GRAPH_CONSTANTS.THEME.ROLE_COLORS.DEFAULT;
  }
}

function getRoleName(nodeId: string): string {
  const char = projectStore.characterById(nodeId);
  if (!char) return 'Unknown';
  return char.role.charAt(0).toUpperCase() + char.role.slice(1);
}

// --- Engine Interaction ---

function initEngine() {
  if (!svgRef.value || !payload.value) return;

  // Initialize engine if it doesn't exist
  if (!engine) {
    engine = new CharacterGraphEngine(svgRef.value, {
      width: props.width,
      height: props.height,
      getNodeColor,
      onNodeClick: (node) => {
        selectedNodeId.value = node.id === selectedNodeId.value ? null : node.id;
        emit('nodeSelect', selectedNodeId.value ? node : null);
        if (engine) engine.highlightNode(selectedNodeId.value);
      },
      onNodeDoubleClick: (node) => {
        if (node.firstMention) {
          emit('navigateToMention', node.firstMention.chapterId);
        }
      },
      onNodeContextMenu: (event, node) => {
        event.preventDefault();
        const svgRect = svgRef.value?.getBoundingClientRect();
        if (svgRect) {
          contextMenuData.value = {
            node,
            x: event.clientX,
            y: event.clientY,
          };
        }
      },
      onNodeHover: (event, node) => {
        const svgRect = svgRef.value?.getBoundingClientRect();
        if (svgRect) {
          tooltipData.value = {
            node,
            x: event.clientX - svgRect.left,
            y: event.clientY - svgRect.top - 10,
          };
        }
      },
      onNodeBlur: () => {
        tooltipData.value = null;
      },
      onNodeFocus: (node) => {
        liveAnnouncement.value = `${node.label}. ${node.mentionCount} mentions.`;
      },
      // Zoom is handled by engine internally for transform,
      // but if we need external sync, we can use onZoom callback.
    });
  }

  // Update data without destroying simulation
  engine.update(payload.value.nodes, payload.value.edges);

  // Restore selection if ID persists
  if (selectedNodeId.value) {
    engine.highlightNode(selectedNodeId.value);
  }
}

// --- Controls ---

function zoomIn() {
  engine?.zoomIn();
}

function zoomOut() {
  engine?.zoomOut();
}

function resetZoom() {
  engine?.resetZoom();
}

function closeContextMenu() {
  contextMenuData.value = null;
}

async function copyCharacterTag(node: D3Node) {
  const tag = `@${node.label}`;
  try {
    await navigator.clipboard.writeText(tag);
    liveAnnouncement.value = `Copied ${tag} to clipboard`;
  } catch {
    liveAnnouncement.value = 'Failed to copy to clipboard';
  }
  closeContextMenu();
}

// --- Lifecycle ---

onMounted(async () => {
  await analyze();
  // Ensure DOM is ready for D3
  await nextTick();
  initEngine();
});

onUnmounted(() => {
  engine?.dispose();
  engine = null;
});

// efficiently watch for new graph data
watch(
  () => payload.value,
  (newData) => {
    if (newData && engine) {
      engine.update(newData.nodes, newData.edges);
    } else if (newData && !engine) {
      // Lazy init if not ready yet
      nextTick(() => initEngine());
    }
  },
  { deep: false } // Disable deep watch to improve performance
);
</script>

<template>
  <div class="relative w-full h-full min-h-[400px] bg-transparent overflow-hidden">
    <!-- Loading State (Overlay) -->
    <div
      v-if="isLoading"
      class="absolute inset-0 z-10 flex flex-col items-center justify-center bg-white/50 backdrop-blur-sm"
    >
      <div class="flex flex-col items-center gap-3">
        <div
          class="w-8 h-8 border-2 border-gray-900/20 border-t-orange-500 rounded-full animate-spin"
        ></div>
        <span class="text-xs uppercase tracking-widest text-gray-900/40 font-bold"
          >Analyzing...</span
        >
      </div>
    </div>

    <!-- Error State -->
    <div
      v-if="error"
      class="absolute inset-0 z-10 flex flex-col items-center justify-center bg-white"
    >
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

    <!-- Graph SVG (Always rendered if no error, or even with error but hidden behind) -->
    <svg
      ref="svgRef"
      :viewBox="`0 0 ${props.width} ${props.height}`"
      class="w-full h-full cursor-grab active:cursor-grabbing"
      role="img"
      aria-label="Character interaction graph"
    />

    <!-- Overlays -->
    <GraphTooltip
      v-if="tooltipData"
      :node="tooltipData.node"
      :x="tooltipData.x"
      :y="tooltipData.y"
      :role-name="getRoleName(tooltipData.node.id)"
    />

    <GraphContextMenu
      v-if="contextMenuData"
      :node="contextMenuData.node"
      :x="contextMenuData.x"
      :y="contextMenuData.y"
      @close="closeContextMenu"
      @copy="copyCharacterTag"
    />

    <GraphControls @zoom-in="zoomIn" @zoom-out="zoomOut" @reset="resetZoom" />

    <GraphGhosts v-if="ghosts.length > 0" :ghosts="ghosts" />

    <GraphLegend />

    <GraphMetrics v-if="metrics" :metrics="metrics" :alerts="alerts" />

    <!-- Screen Reader Live Region -->
    <div aria-live="polite" class="sr-only">{{ liveAnnouncement }}</div>
  </div>
</template>

<style scoped>
/* D3 Graph Styling - kept here as it's specific to the global SVG rendered by D3 */
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
