<script setup lang="ts">
import { onMounted, shallowRef, ref, watch } from 'vue';
import * as pdfjsLib from 'pdfjs-dist';
import { readFile } from '@tauri-apps/plugin-fs';
import BaseIcon from '../base/BaseIcon.vue';

// Set worker source using a stable CDN that includes map and font files
pdfjsLib.GlobalWorkerOptions.workerSrc = `https://cdn.jsdelivr.net/npm/pdfjs-dist@${pdfjsLib.version}/build/pdf.worker.min.mjs`;

const props = defineProps<{
  path: string;
}>();

const containerRef = ref<HTMLElement | null>(null);
const canvasRef = ref<HTMLCanvasElement | null>(null);
const error = ref<string | null>(null);
const isLoading = ref(false);

const scale = ref(1.0);
const pdfDoc = shallowRef<pdfjsLib.PDFDocumentProxy | null>(null);

// Panning State
const isDragging = ref(false);
const startX = ref(0);
const startY = ref(0);
const scrollLeft = ref(0);
const scrollTop = ref(0);

const renderPDF = async () => {
  if (!containerRef.value || !canvasRef.value) return;

  const currentScale = scale.value;
  isLoading.value = true;
  error.value = null;

  try {
    let pdf = pdfDoc.value;
    if (!pdf) {
      const data = await readFile(props.path);
      const loadingTask = pdfjsLib.getDocument({
        data,
        cMapUrl: `https://cdn.jsdelivr.net/npm/pdfjs-dist@${pdfjsLib.version}/cmaps/`,
        cMapPacked: true,
        standardFontDataUrl: `https://cdn.jsdelivr.net/npm/pdfjs-dist@${pdfjsLib.version}/standard_fonts/`,
        disableFontFace: true,
      });
      pdf = await loadingTask.promise;
      pdfDoc.value = pdf;
    }

    const page = await pdf.getPage(1);
    const viewport = page.getViewport({ scale: currentScale });
    const canvas = canvasRef.value;
    if (!canvas) return;
    const context = canvas.getContext('2d');

    if (context) {
      canvas.height = viewport.height;
      canvas.width = viewport.width;

      const renderContext = {
        canvasContext: context,
        viewport: viewport,
        canvas,
      };

      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      await page.render(renderContext as any).promise;
    }
  } catch (e: unknown) {
    console.error('PDF Render Error', e);
    error.value = 'Failed to load PDF: ' + (e instanceof Error ? e.message : String(e));
  } finally {
    isLoading.value = false;
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
  if (!pdfDoc.value || !containerRef.value) return;
  const page = await pdfDoc.value.getPage(1);
  const viewportUnscaled = page.getViewport({ scale: 1.0 });
  const containerWidth = containerRef.value.clientWidth - 64; // padding
  scale.value = containerWidth / viewportUnscaled.width;
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
  () => {
    pdfDoc.value = null; // Clear cache on new file
    scale.value = 1.0; // Reset scale
    renderPDF();
  }
);
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
    >
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
