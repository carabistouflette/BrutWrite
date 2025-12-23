<template>
  <main class="flex-1 flex flex-col h-full bg-paper relative overflow-hidden">
    <!-- Header / Toolbar -->
    <header
      v-if="store.activeArtifact"
      class="h-20 px-4 flex items-center justify-between border-b border-ink/5 bg-paper/95 backdrop-blur z-20"
    >
      <div class="flex items-center gap-3">
        <!-- Back Button -->
        <button
          class="mr-2 w-8 h-8 flex items-center justify-center rounded-full hover:bg-stone text-ink/40 hover:text-ink transition-colors"
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
          <h1 class="font-serif text-xl text-ink tracking-wide truncate max-w-[180px]">
            {{ store.activeArtifact.name }}
          </h1>
          <div class="flex items-center gap-3 mt-1">
            <span class="text-[10px] font-black uppercase tracking-widest text-ink/40">
              {{ store.activeArtifact.file_type }}
            </span>
            <span class="text-[10px] font-bold uppercase tracking-widest text-ink/10"> • </span>
            <span class="text-[10px] font-mono text-ink/30 truncate max-w-lg">
              {{ store.activeArtifact.path }}
            </span>
          </div>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <button
          class="text-[10px] font-black uppercase tracking-widest text-ink/40 hover:text-ink transition-colors flex items-center gap-1 px-2 py-1"
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
        <div class="h-4 w-px bg-ink/10 mx-1"></div>
        <button
          class="p-1.5 rounded-md text-ink/30 hover:text-ink hover:bg-stone transition-colors"
          @click="$emit('close')"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
    </header>

    <!-- Content Area -->
    <div v-if="store.activeArtifact" class="flex-1 overflow-hidden relative bg-paper">
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
        <NoteEditor
          v-else-if="
            ['text', 'md'].includes(store.activeArtifact.file_type) ||
            store.activeArtifact.name.endsWith('.md') ||
            store.activeArtifact.name.endsWith('.txt')
          "
          :id="store.activeArtifact.id"
          :path="store.activeArtifact.path"
          class="flex-1 w-full h-full"
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
            class="mb-6 text-ink/20"
          >
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
          </svg>
          <span class="font-serif text-2xl text-ink/60">No Preview Available</span>
          <span class="text-sm font-mono mt-3 text-ink/20">Format not supported yet</span>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="h-full flex flex-col items-center justify-center p-12 bg-paper">
      <div
        class="w-40 h-40 rounded-full border-2 border-dashed border-ink/5 flex items-center justify-center mb-10 relative group"
      >
        <div
          class="absolute inset-0 bg-accent/5 blur-3xl rounded-full group-hover:bg-accent/10 transition-all duration-500"
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
          class="text-ink/20 group-hover:text-ink/40 transition-colors"
        >
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
          <polyline points="14 2 14 8 20 8"></polyline>
          <line x1="16" y1="13" x2="8" y2="13"></line>
          <line x1="16" y1="17" x2="8" y2="17"></line>
          <polyline points="10 9 9 9 8 9"></polyline>
        </svg>
      </div>

      <h2 class="font-serif text-4xl text-ink mb-4 tracking-wide">Select an Artifact</h2>
      <p class="text-ink/40 text-base max-w-sm text-center leading-relaxed font-medium">
        Choose a document from the sidebar to view its contents, or import new materials to your
        vault.
      </p>

      <button
        class="mt-10 px-8 py-4 border-2 border-ink/10 hover:border-accent/40 text-sm font-black uppercase tracking-widest text-ink/40 hover:text-accent hover:bg-accent/5 transition-all duration-300 transform hover:scale-105"
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
import NoteEditor from './NoteEditor.vue';

const store = useResearchStore();

defineEmits(['add', 'close']);
</script>
