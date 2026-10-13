<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, shallowRef, ref, watch } from 'vue';
import * as pdfjsLib from 'pdfjs-dist';
import pdfWorkerUrl from 'pdfjs-dist/build/pdf.worker.min.mjs?url';
import { readFile } from '@tauri-apps/plugin-fs';
import BaseIcon from '../base/BaseIcon.vue';

// Bundle all PDF.js assets so previews work offline under the native CSP.
pdfjsLib.GlobalWorkerOptions.workerSrc = pdfWorkerUrl;
const pdfAssetRoot = `${import.meta.env.BASE_URL}pdfjs/`;

const props = defineProps<{
  path: string;
}>();

const containerRef = ref<HTMLElement | null>(null);
const canvasRef = ref<HTMLCanvasElement | null>(null);
const error = ref<string | null>(null);
const isLoading = ref(false);

const pageNumber = ref(1);
const scale = ref(1.0);
const pdfDoc = shallowRef<pdfjsLib.PDFDocumentProxy | null>(null);
const pageCount = computed(() => pdfDoc.value?.numPages ?? 0);
let renderGeneration = 0;
let activeRenderTask: { cancel: () => void } | null = null;

// Panning State
const isDragging = ref(false);
const startX = ref(0);
const startY = ref(0);
const scrollLeft = ref(0);
const scrollTop = ref(0);

const renderPDF = async () => {
  if (!containerRef.value || !canvasRef.value) return;

  const request = ++renderGeneration;
  activeRenderTask?.cancel();
  activeRenderTask = null;
  const currentScale = scale.value;
  const currentPage = pageNumber.value;
  isLoading.value = true;
  error.value = null;

  try {
    let pdf = pdfDoc.value;
    if (!pdf) {
      const data = await readFile(props.path);
      if (request !== renderGeneration) return;
      const loadingTask = pdfjsLib.getDocument({
        data,
        cMapUrl: `${pdfAssetRoot}cmaps/`,
        cMapPacked: true,
        standardFontDataUrl: `${pdfAssetRoot}standard_fonts/`,
        disableFontFace: true,
      });
      pdf = await loadingTask.promise;
      if (request !== renderGeneration) {
        void pdf.loadingTask.destroy();
        return;
      }
      pdfDoc.value = pdf;
    }

    const page = await pdf.getPage(currentPage);
    if (request !== renderGeneration) return;
    const viewport = page.getViewport({ scale: currentScale });
    const canvas = canvasRef.value;
    if (!canvas) return;
    const context = canvas.getContext('2d');

    if (context) {
      canvas.height = viewport.height;
      canvas.width = viewport.width;

      const renderContext = {
        canvasContext: context,
        viewport,
        canvas,
      };

      const renderTask = page.render(renderContext);
      activeRenderTask = renderTask;
      await renderTask.promise;
    }
  } catch (e: unknown) {
    if (request !== renderGeneration) return;
    error.value = 'Failed to load PDF: ' + (e instanceof Error ? e.message : String(e));
  } finally {
    if (request === renderGeneration) {
      isLoading.value = false;
      activeRenderTask = null;
    }
  }
};

const handleZoomIn = () => {
  scale.value = Math.min(scale.value + 0.25, 3.0);
  renderPDF();
};

const handleZoomOut = () => {
  scale.value = Math.max(scale.value - 0.25, 0.5);
  renderPDF();
};

const handleFitWidth = async () => {
  const pdf = pdfDoc.value;
  if (!pdf || !containerRef.value) return;
  const page = await pdf.getPage(pageNumber.value);
  if (pdf !== pdfDoc.value) return;
  const viewportUnscaled = page.getViewport({ scale: 1.0 });
  const containerWidth = containerRef.value.clientWidth - 64; // padding
  scale.value = Math.max(0.5, Math.min(3.0, containerWidth / viewportUnscaled.width));
  renderPDF();
};

const handlePreviousPage = () => {
  if (pageNumber.value <= 1) return;
  pageNumber.value -= 1;
  renderPDF();
};

const handleNextPage = () => {
  if (pageNumber.value >= pageCount.value) return;
  pageNumber.value += 1;
  renderPDF();
};

// Panning Handlers
const onMouseDown = (e: MouseEvent) => {
  if (!containerRef.value) return;
  isDragging.value = true;
  startX.value = e.pageX - containerRef.value.offsetLeft;
  startY.value = e.pageY - containerRef.value.offsetTop;
  scrollLeft.value = containerRef.value.scrollLeft;
  scrollTop.value = containerRef.value.scrollTop;
};

const onMouseMove = (e: MouseEvent) => {
  if (!isDragging.value || !containerRef.value) return;
  e.preventDefault();
  const x = e.pageX - containerRef.value.offsetLeft;
  const y = e.pageY - containerRef.value.offsetTop;
  const walkX = (x - startX.value) * 1.5; // Scroll speed multiplier
  const walkY = (y - startY.value) * 1.5;
  containerRef.value.scrollLeft = scrollLeft.value - walkX;
  containerRef.value.scrollTop = scrollTop.value - walkY;
};

const onMouseUp = () => {
  isDragging.value = false;
};

const onMouseLeave = () => {
  isDragging.value = false;
};

onMounted(() => {
  renderPDF();
});

watch(
  () => props.path,
  async () => {
    const request = ++renderGeneration;
    activeRenderTask?.cancel();
    activeRenderTask = null;
    const previousDocument = pdfDoc.value;
    pdfDoc.value = null;
    pageNumber.value = 1;
    scale.value = 1.0;
    if (previousDocument) {
      await previousDocument.loadingTask.destroy();
    }
    if (request === renderGeneration) {
      renderPDF();
    }
  }
);

onBeforeUnmount(() => {
  renderGeneration++;
  activeRenderTask?.cancel();
  void pdfDoc.value?.loadingTask.destroy();
});
</script>

<template>
  <div
    ref="containerRef"
    class="flex-1 overflow-auto bg-stone/20 relative custom-scrollbar cursor-grab active:cursor-grabbing flex"
    @mousedown="onMouseDown"
    @mousemove="onMouseMove"
    @mouseup="onMouseUp"
    @mouseleave="onMouseLeave"
  >
    <!-- Floating Toolbar -->
    <div
      class="fixed bottom-6 left-1/2 -translate-x-1/2 bg-paper shadow-xl border border-ink/10 rounded-full px-4 py-2 flex items-center gap-4 z-50 backdrop-blur-md"
      @mousedown.stop
    >
      <button
        class="text-ink/60 hover:text-ink hover:scale-110 transition-all font-mono text-xs disabled:opacity-30"
        title="Previous Page"
        :disabled="pageNumber <= 1"
        @click="handlePreviousPage"
      >
        <BaseIcon name="arrowLeft" size="16" />
      </button>
      <span class="text-[10px] font-bold text-ink/40 min-w-10 text-center">
        {{ pageNumber }} / {{ pageCount }}
      </span>
      <button
        class="text-ink/60 hover:text-ink hover:scale-110 transition-all font-mono text-xs disabled:opacity-30"
        title="Next Page"
        :disabled="pageNumber >= pageCount"
        @click="handleNextPage"
      >
        <BaseIcon name="arrowLeft" size="16" class="rotate-180" />
      </button>
      <div class="h-4 w-px bg-ink/10"></div>
      <button
        class="text-ink/60 hover:text-ink hover:scale-110 transition-all font-mono text-xs"
        title="Zoom Out"
        @click="handleZoomOut"
      >
        <BaseIcon name="minus" size="16" />
      </button>
      <span class="text-[10px] font-bold text-ink/40 w-8 text-center"
        >{{ Math.round(scale * 100) }}%</span
      >
      <button
        class="text-ink/60 hover:text-ink hover:scale-110 transition-all font-mono text-xs"
        title="Zoom In"
        @click="handleZoomIn"
      >
        <BaseIcon name="plus" size="16" />
      </button>
      <!-- Fit Width -->
      <button
        class="text-ink/60 hover:text-ink hover:scale-110 transition-all font-mono text-xs border-l border-ink/10 pl-4 ml-1"
        title="Fit Width"
        @click="handleFitWidth"
      >
        <BaseIcon name="maximize" size="16" />
      </button>
    </div>

    <div
      v-if="isLoading"
      class="absolute inset-0 flex flex-col items-center justify-center bg-paper/80 backdrop-blur-sm z-10"
    >
      <div class="w-12 h-1 border-2 border-ink bg-ink/10 relative overflow-hidden mb-4">
        <div
          class="absolute inset-0 bg-accent -translate-x-full animate-[progress_1.5s_infinite]"
        ></div>
      </div>
      <span class="text-[10px] font-black uppercase tracking-widest text-ink/40"
        >Loading Document</span
      >
    </div>

    <div
      v-if="error"
      class="max-w-md w-full p-6 border-4 border-ink bg-paper shadow-[8px_8px_0_var(--color-ink)] m-auto"
    >
      <div class="text-2xl mb-2">⚠️</div>
      <div class="text-xs font-black uppercase tracking-widest text-red-500 mb-2">
        Error Loading PDF
      </div>
      <div class="text-sm font-medium text-ink/60">{{ error }}</div>
    </div>

    <div v-show="!isLoading && !error" class="relative group m-auto p-8">
      <div class="absolute inset-0 bg-ink translate-x-2 translate-y-2 opacity-5"></div>
      <canvas
        ref="canvasRef"
        class="relative border-2 border-ink shadow-2xl bg-paper"
        :style="{ maxWidth: 'none' }"
      ></canvas>
    </div>
  </div>
</template>

<style scoped>
@keyframes progress {
  0% {
    transform: translateX(-100%);
  }
  50% {
    transform: translateX(0);
  }
  100% {
    transform: translateX(100%);
  }
}

.custom-scrollbar::-webkit-scrollbar {
  width: 10px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: var(--stone);
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: var(--ink);
  opacity: 0.2;
}
</style>
