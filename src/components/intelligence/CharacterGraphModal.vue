<script setup lang="ts">
/**
 * CharacterGraphModal.vue
 *
 * Modal wrapper for the Character Graph visualization.
 * Features:
 * - Chapter filtering
 * - Export to PNG
 * - Styled to match CharacterSheet's warm, editorial aesthetic.
 */

import { ref, defineAsyncComponent, computed } from 'vue';
import { storeToRefs } from 'pinia';
import { save } from '@tauri-apps/plugin-dialog';
import { writeFile } from '@tauri-apps/plugin-fs';
import { useProjectStore } from '../../stores/project';
import { useCharacterGraph } from '../../composables/domain/intelligence/useCharacterGraph';
import { useAppStatus } from '../../composables/ui/useAppStatus';

const CharacterGraph = defineAsyncComponent(() => import('./CharacterGraph.vue'));

defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const projectStore = useProjectStore();
const { flatNodes } = storeToRefs(projectStore);
const graphStore = useCharacterGraph();
const { analyze } = graphStore;
const { isLoading } = storeToRefs(graphStore);

const handleRefresh = async () => {
  await analyze(selectedChapterIds.value.length > 0 ? selectedChapterIds.value : undefined);
};

// Chapter filter state
const selectedChapterIds = ref<string[]>([]);
const showChapterFilter = ref(false);

// Get all chapters (leaf nodes only)
const chapters = computed(() =>
  flatNodes.value.filter((n) => !n.children || n.children.length === 0)
);

const close = () => {
  emit('close');
};

/**
 * Navigate to a specific chapter when double-clicking a character node.
 */
const handleNavigateToMention = (chapterId: string) => {
  projectStore.setActiveId(chapterId);
  close();
};

/**
 * Toggle chapter selection for filtering
 */
const toggleChapter = (chapterId: string) => {
  const index = selectedChapterIds.value.indexOf(chapterId);
  if (index === -1) {
    selectedChapterIds.value.push(chapterId);
  } else {
    selectedChapterIds.value.splice(index, 1);
  }
};

/**
 * Apply chapter filter and re-analyze
 */
const applyFilter = async () => {
  showChapterFilter.value = false;
  await analyze(selectedChapterIds.value.length > 0 ? selectedChapterIds.value : undefined);
};

/**
 * Clear filter and show all chapters
 */
const clearFilter = async () => {
  selectedChapterIds.value = [];
  showChapterFilter.value = false;
  await analyze();
};

const { notify, notifyError } = useAppStatus();

/**
 * Export the graph as PNG
 */
const exportPng = async () => {
  const svg = document.querySelector('.graph-svg') as SVGSVGElement | null;
  if (!svg) return;

  // Clone SVG and prepare for export
  const clone = svg.cloneNode(true) as SVGSVGElement;
  clone.setAttribute('xmlns', 'http://www.w3.org/2000/svg');

  // Inline styles for export
  const styleElement = document.createElementNS('http://www.w3.org/2000/svg', 'style');
  styleElement.textContent = `
    .graph-node { fill: #1a1a1a; stroke: #f4f4f0; stroke-width: 2; }
    .graph-link { stroke: rgba(26, 26, 26, 0.15); }
    .graph-label { font-family: Georgia, serif; font-size: 12px; fill: #1a1a1a; }
  `;
  clone.insertBefore(styleElement, clone.firstChild);

  // Add white background
  const bg = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
  bg.setAttribute('width', '100%');
  bg.setAttribute('height', '100%');
  bg.setAttribute('fill', '#f4f4f0');
  clone.insertBefore(bg, clone.firstChild);

  // Convert to data URL
  const svgData = new XMLSerializer().serializeToString(clone);
  const svgBlob = new Blob([svgData], { type: 'image/svg+xml;charset=utf-8' });
  const url = URL.createObjectURL(svgBlob);

  // Create canvas and draw
  const img = new Image();
  img.onload = async () => {
    const canvas = document.createElement('canvas');
    canvas.width = 1000;
    canvas.height = 550;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    ctx.fillStyle = '#f4f4f0';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.drawImage(img, 0, 0);

    URL.revokeObjectURL(url);

    try {
      // Get data URL and convert to Uint8Array
      const dataUrl = canvas.toDataURL('image/png');
      const base64 = dataUrl.replace(/^data:image\/\w+;base64,/, '');
      const binaryString = atob(base64);
      const bytes = new Uint8Array(binaryString.length);
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }

      // Open save dialog
      const path = await save({
        defaultPath: `character-graph-${new Date().toISOString().split('T')[0]}.png`,
        filters: [{ name: 'Image', extensions: ['png'] }],
      });

      // Write to file if path selected
      if (path) {
        await writeFile(path, bytes);
        notify('Graph exported successfully');
      }
    } catch (e) {
      notifyError('Failed to export graph', e);
    }
  };
  img.src = url;
};
</script>

<template>
  <Teleport to="#app-scale-root">
    <Transition
      enter-active-class="transition duration-300 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition duration-200 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4 lg:p-8">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" @click="close"></div>

        <!-- Modal Container -->
        <div
          class="relative w-full max-w-6xl h-[85%] flex flex-col bg-(--paper)/95 backdrop-blur-2xl border border-white/20 shadow-2xl rounded-2xl overflow-hidden text-(--ink) modal-container"
          style="box-shadow: 0 24px 60px -12px rgba(0, 0, 0, 0.4)"
        >
          <!-- Header -->
          <header
            class="px-8 py-4 border-b border-(--ink)/5 flex justify-between items-center bg-(--paper)/50"
          >
            <div>
              <h2 class="text-2xl font-serif font-bold italic text-(--ink)">Character Dynamics</h2>
              <p class="text-xs uppercase tracking-widest text-(--ink)/40 font-bold mt-0.5">
                Narrative Gravity Visualization
                <span v-if="selectedChapterIds.length > 0" class="text-(--accent)">
                  ({{ selectedChapterIds.length }} chapters filtered)
                </span>
              </p>
            </div>
            <div class="flex items-center gap-2">
              <!-- Refresh Button -->
              <button
                class="flex items-center justify-center w-8 h-8 rounded-lg text-black/40 transition-all hover:bg-black/5 hover:text-(--ink)"
                :class="{ 'animate-spin': isLoading }"
                title="Refresh Graph"
                @click="handleRefresh"
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

              <!-- Filter Button -->
              <button
                class="flex items-center justify-center w-8 h-8 rounded-lg text-black/40 transition-all hover:bg-black/5 hover:text-(--ink)"
                :class="{
                  'bg-(--accent)/10 text-(--accent)': selectedChapterIds.length > 0,
                }"
                title="Filter by chapters"
                @click="showChapterFilter = !showChapterFilter"
              >
                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z"
                  />
                </svg>
              </button>

              <!-- Export Button -->
              <button
                class="flex items-center justify-center w-8 h-8 rounded-lg text-black/40 transition-all hover:bg-black/5 hover:text-(--ink)"
                title="Export as PNG"
                @click="exportPng"
              >
                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
                  />
                </svg>
              </button>

              <!-- Close Button -->
              <button
                class="w-8 h-8 flex items-center justify-center rounded-full hover:bg-black/5 text-(--ink)/40 hover:text-(--ink) transition-colors"
                @click="close"
              >
                <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            </div>
          </header>

          <!-- Chapter Filter Panel -->
          <Transition
            enter-active-class="transition-all duration-200"
            enter-from-class="-translate-y-2 opacity-0"
            leave-active-class="transition-all duration-150"
            leave-to-class="-translate-y-2 opacity-0"
          >
            <div v-if="showChapterFilter" class="px-8 py-4 border-b border-(--ink)/5 bg-(--ink)/5">
              <div class="flex items-center justify-between mb-3">
                <h3 class="text-xs uppercase tracking-widest text-(--ink)/60 font-bold">
                  Filter by Chapters
                </h3>
                <div class="flex gap-2">
                  <button
                    class="text-xs text-(--ink)/50 hover:text-(--ink) transition-colors"
                    @click="clearFilter"
                  >
                    Clear All
                  </button>
                  <button
                    class="text-xs text-(--accent) font-bold hover:text-(--accent)/80 transition-colors"
                    @click="applyFilter"
                  >
                    Apply Filter
                  </button>
                </div>
              </div>
              <div class="flex flex-wrap gap-2 max-h-32 overflow-y-auto">
                <button
                  v-for="chapter in chapters"
                  :key="chapter.id"
                  class="px-3 py-1.5 text-xs font-medium text-black/60 bg-black/5 border border-black/10 rounded-full transition-all hover:bg-black/10 hover:text-(--ink)"
                  :class="{
                    'bg-(--accent) border-(--accent) text-white hover:bg-(--accent) hover:text-white':
                      selectedChapterIds.includes(chapter.id),
                  }"
                  @click="toggleChapter(chapter.id)"
                >
                  {{ chapter.name }}
                </button>
              </div>
            </div>
          </Transition>

          <!-- Graph Container -->
          <div class="flex-1 overflow-hidden bg-transparent">
            <CharacterGraph
              :width="1000"
              :height="showChapterFilter ? 480 : 550"
              @navigate-to-mention="handleNavigateToMention"
            />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-container {
  animation: modal-pop 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes modal-pop {
  0% {
    transform: scale(0.95);
    opacity: 0;
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}
</style>
