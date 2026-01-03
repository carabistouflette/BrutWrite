<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { useCharacterGraph } from '../../composables/domain/intelligence/useCharacterGraph';
import { useProjectStore } from '../../stores/project';
import {
  CharacterGraphEngine,
  type D3Node,
} from '../../composables/domain/intelligence/CharacterGraphEngine';
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
const liveAnnouncement = ref('');
const tooltipData = ref<{ node: D3Node; x: number; y: number } | null>(null);
const currentZoom = ref(1); // Not synced with engine yet, maybe remove or expose getter
const contextMenuData = ref<{ node: D3Node; x: number; y: number } | null>(null);

let engine: CharacterGraphEngine | null = null;

// --- Computed ---

const metrics = computed(() => payload.value?.metrics);

// --- Helpers ---

function getNodeColor(nodeId: string): string {
  const char = projectStore.characterById(nodeId);
  if (!char) return 'var(--ink)';

  switch (char.role) {
    case 'protagonist':
      return 'var(--accent)'; // Orange
    case 'antagonist':
      return 'var(--antagonist, #DC2626)'; // Red
    case 'secondary':
      return 'var(--ink)'; // Default black
    case 'extra':
      return 'var(--ink-muted, rgba(26, 26, 26, 0.5))'; // Muted
    default:
      return 'var(--ink)';
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
      onZoom: (k) => {
        currentZoom.value = k;
      },
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

// --- Lifecycle ---

onMounted(async () => {
  await analyze();
  await nextTick();
  initEngine();
});

onUnmounted(() => {
  engine?.dispose();
  engine = null;
});

watch(payload, () => {
  nextTick(() => initEngine());
});
</script>

<template>
  <div class="relative w-full h-full min-h-[400px] bg-transparent overflow-hidden">
    <!-- Loading State -->
    <div
      v-if="isLoading"
      class="absolute inset-0 flex flex-col items-center justify-center bg-(--paper)"
    >
      <div class="flex flex-col items-center gap-3">
        <div
          class="w-8 h-8 border-2 border-(--ink)/20 border-t-(--accent) rounded-full animate-spin"
        ></div>
        <span class="text-xs uppercase tracking-widest text-(--ink)/40 font-bold"
          >Analyzing...</span
        >
      </div>
    </div>

    <!-- Error State -->
    <div
      v-else-if="error"
      class="absolute inset-0 flex flex-col items-center justify-center bg-(--paper)"
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

    <!-- Graph SVG -->
    <svg
      v-else
      ref="svgRef"
      :viewBox="`0 0 ${props.width} ${props.height}`"
      class="w-full h-full cursor-grab active:cursor-grabbing"
      role="img"
      aria-label="Character interaction graph"
    />

    <!-- Tooltip -->
    <div
      v-if="tooltipData"
      class="absolute transform -translate-x-1/2 -translate-y-full p-3 px-4 bg-(--paper) border border-black/10 rounded-xl shadow-lg pointer-events-none z-tooltip min-w-[150px]"
      :style="{ left: `${tooltipData.x}px`, top: `${tooltipData.y}px` }"
    >
      <div class="font-serif text-sm font-semibold italic text-(--ink) mb-0.5">
        {{ tooltipData.node.label }}
      </div>
      <div class="text-[10px] font-bold uppercase tracking-widest text-(--accent) mb-2">
        {{ getRoleName(tooltipData.node.id) }}
      </div>
      <div class="flex gap-2 text-xs text-black/60">
        <span>{{ tooltipData.node.mentionCount }} mentions</span>
      </div>
      <div class="text-[10px] text-black/40 mt-1.5">
        Valence: {{ tooltipData.node.valence.toFixed(2) }}
      </div>
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
        class="fixed z-max p-2 bg-(--paper) border border-black/10 rounded-xl shadow-2xl min-w-[180px]"
        :style="{ left: `${contextMenuData.x}px`, top: `${contextMenuData.y}px` }"
      >
        <button
          class="flex items-center gap-2 w-full p-2 px-3 text-xs font-medium text-(--ink) rounded-lg transition-all text-left hover:bg-(--accent) hover:text-white"
          @click="copyCharacterTag"
        >
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
    <div
      class="absolute top-4 left-4 flex items-center gap-1 p-1 bg-(--paper) border border-black/10 rounded-lg shadow-sm"
    >
      <button
        class="flex items-center justify-center w-7 h-7 rounded text-black/60 transition-all hover:bg-black/5 hover:text-(--ink)"
        title="Zoom in"
        @click="zoomIn"
      >
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 4v16m8-8H4"
          />
        </svg>
      </button>
      <button
        class="flex items-center justify-center w-7 h-7 rounded text-black/60 transition-all hover:bg-black/5 hover:text-(--ink)"
        title="Zoom out"
        @click="zoomOut"
      >
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
        </svg>
      </button>
      <button
        class="flex items-center justify-center w-7 h-7 rounded text-black/60 transition-all hover:bg-black/5 hover:text-(--ink)"
        title="Reset zoom"
        @click="resetZoom"
      >
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

    <!-- Ghost Panel -->
    <aside
      v-if="ghosts.length > 0"
      class="absolute top-4 right-4 max-w-[200px] p-4 px-5 bg-linear-to-br from-purple-500/5 to-transparent border border-purple-500/10 rounded-2xl backdrop-blur-md"
    >
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
          class="text-sm text-(--ink)/60 font-medium flex items-center gap-2"
        >
          <span class="w-1.5 h-1.5 rounded-full bg-purple-400/50"></span>
          {{ ghost.label }}
        </li>
      </ul>
    </aside>

    <!-- Role Legend -->
    <div
      class="absolute bottom-14 left-4 flex gap-4 p-2 px-3 bg-white/80 border border-black/5 rounded-lg backdrop-blur-sm"
    >
      <div
        class="flex items-center gap-1.5 text-[10px] font-semibold text-black/60 uppercase tracking-wide"
      >
        <span class="w-2 h-2 rounded-full" style="background-color: var(--accent)"></span>
        <span>Protagonist</span>
      </div>
      <div
        class="flex items-center gap-1.5 text-[10px] font-semibold text-black/60 uppercase tracking-wide"
      >
        <span class="w-2 h-2 rounded-full" style="background-color: #dc2626"></span>
        <span>Antagonist</span>
      </div>
      <div
        class="flex items-center gap-1.5 text-[10px] font-semibold text-black/60 uppercase tracking-wide"
      >
        <span class="w-2 h-2 rounded-full" style="background-color: var(--ink)"></span>
        <span>Secondary</span>
      </div>
      <div
        class="flex items-center gap-1.5 text-[10px] font-semibold text-black/60 uppercase tracking-wide"
      >
        <span class="w-2 h-2 rounded-full" style="background-color: rgba(26, 26, 26, 0.5)"></span>
        <span>Extra</span>
      </div>
    </div>

    <!-- Metrics HUD -->
    <footer
      v-if="metrics"
      class="absolute bottom-0 left-0 right-0 flex justify-between items-center p-3 px-6 bg-linear-to-t from-(--paper) to-transparent border-t border-black/5"
    >
      <div class="flex items-center gap-4">
        <div class="flex flex-col gap-0.5">
          <span class="text-[10px] font-bold uppercase tracking-widest text-black/40">Density</span>
          <span class="text-sm font-semibold font-serif text-(--ink)"
            >{{ (metrics.networkDensity * 100).toFixed(0) }}%</span
          >
        </div>
        <div class="w-px h-6 bg-black/8"></div>
        <div class="flex flex-col gap-0.5">
          <span class="text-[10px] font-bold uppercase tracking-widest text-black/40"
            >Components</span
          >
          <span class="text-sm font-semibold font-serif text-(--ink)">{{
            metrics.connectedComponents
          }}</span>
        </div>
        <div class="w-px h-6 bg-black/8"></div>
        <div class="flex flex-col gap-0.5">
          <span class="text-[10px] font-bold uppercase tracking-widest text-black/40"
            >Isolated</span
          >
          <span class="text-sm font-semibold font-serif text-(--ink)"
            >{{ (metrics.isolationRatio * 100).toFixed(0) }}%</span
          >
        </div>
      </div>

      <!-- Alert Badges -->
      <div v-if="alerts.length > 0" class="flex gap-2">
        <span
          v-for="alert in alerts"
          :key="alert.code"
          class="p-1.5 px-3 text-[10px] font-bold uppercase tracking-wide text-(--paper) bg-(--accent) rounded-lg cursor-help transition-all hover:-translate-y-px hover:shadow-lg hover:shadow-orange-500/30"
          :title="alert.tooltip"
        >
          {{ alert.primaryText }}
        </span>
      </div>
    </footer>

    <!-- Screen Reader Live Region -->
    <div aria-live="polite" class="sr-only">{{ liveAnnouncement }}</div>
  </div>
</template>

<style scoped>
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
