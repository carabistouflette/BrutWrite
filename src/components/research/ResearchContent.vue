<template>
  <main class="flex-1 flex flex-col h-full bg-zinc-950 relative overflow-hidden">
    <!-- Header / Toolbar -->
    <header
      v-if="store.activeArtifact"
      class="h-20 px-4 flex items-center justify-between border-b border-white/5 bg-zinc-950/95 backdrop-blur z-20"
    >
      <div class="flex items-center gap-3">
        <!-- Back Button -->
        <button
          class="mr-2 w-8 h-8 flex items-center justify-center rounded-full hover:bg-zinc-800 text-zinc-500 hover:text-zinc-200 transition-colors"
          @click="store.setActiveArtifact(null)"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M19 12H5M12 19l-7-7 7-7" />
          </svg>
        </button>

        <div class="flex flex-col min-w-0">
          <h1 class="font-serif text-xl text-zinc-200 tracking-wide truncate max-w-[180px]">
            {{ store.activeArtifact.name }}
          </h1>
          <div class="flex items-center gap-3 mt-1">
            <span class="text-[10px] font-black uppercase tracking-widest text-zinc-400">
              {{ store.activeArtifact.file_type }}
            </span>
            <span class="text-[10px] font-bold uppercase tracking-widest text-zinc-700"> • </span>
            <span class="text-[10px] font-mono text-zinc-600 truncate max-w-lg">
              {{ store.activeArtifact.path }}
            </span>
          </div>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <button
          class="text-[10px] font-black uppercase tracking-widest text-zinc-500 hover:text-zinc-200 transition-colors flex items-center gap-1 px-2 py-1"
          title="Open in System Explorer"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
            <polyline points="15 3 21 3 21 9"></polyline>
            <line x1="10" y1="14" x2="21" y2="3"></line>
          </svg>
          Open
        </button>
      </div>
    </header>

    <!-- Content Area -->
    <div v-if="store.activeArtifact" class="flex-1 overflow-hidden relative bg-zinc-900/50">
      <div class="h-full w-full flex flex-col">
        <PDFViewer
          v-if="store.activeArtifact.file_type === 'pdf'"
          :path="store.activeArtifact.path"
          class="flex-1 w-full h-full shadow-2xl"
        />
        <ImageViewer
          v-else-if="store.activeArtifact.file_type === 'image'"
          :path="store.activeArtifact.path"
          class="flex-1 w-full h-full object-contain p-8"
        />
        <!-- Unsupported Format -->
        <div v-else class="flex-1 flex flex-col items-center justify-center opacity-60">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="64"
            height="64"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="mb-6 text-zinc-600"
          >
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
          </svg>
          <span class="font-serif text-2xl text-zinc-300">No Preview Available</span>
          <span class="text-sm font-mono mt-3 text-zinc-600">Format not supported yet</span>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="h-full flex flex-col items-center justify-center p-12 bg-zinc-950">
      <div
        class="w-40 h-40 rounded-full border-2 border-dashed border-zinc-800 flex items-center justify-center mb-10 relative group"
      >
        <div
          class="absolute inset-0 bg-white/5 blur-3xl rounded-full group-hover:bg-white/10 transition-all duration-500"
        ></div>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="64"
          height="64"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="text-zinc-700 group-hover:text-zinc-500 transition-colors"
        >
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
          <polyline points="14 2 14 8 20 8"></polyline>
          <line x1="16" y1="13" x2="8" y2="13"></line>
          <line x1="16" y1="17" x2="8" y2="17"></line>
          <polyline points="10 9 9 9 8 9"></polyline>
        </svg>
      </div>

      <h2 class="font-serif text-4xl text-zinc-200 mb-4 tracking-wide">Select an Artifact</h2>
      <p class="text-zinc-500 text-base max-w-sm text-center leading-relaxed font-medium">
        Choose a document from the sidebar to view its contents, or import new materials to your
        vault.
      </p>

      <button
        class="mt-10 px-8 py-4 border-2 border-zinc-800 hover:border-zinc-600 text-sm font-black uppercase tracking-widest text-zinc-400 hover:text-zinc-100 hover:bg-zinc-900 transition-all duration-300 transform hover:scale-105"
        @click="$emit('add')"
      >
        Import Material
      </button>
    </div>
  </main>
</template>

<script setup lang="ts">
import { useResearchStore } from '../../stores/research';
import PDFViewer from './PDFViewer.vue';
import ImageViewer from './ImageViewer.vue';

const store = useResearchStore();

defineEmits(['add']);
</script>
