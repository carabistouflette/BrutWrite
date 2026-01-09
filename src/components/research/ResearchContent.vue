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
          <BaseIcon name="arrowLeft" size="20" />
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
          <BaseIcon name="externalLink" size="12" />
          Open
        </button>
        <div class="h-4 w-px bg-ink/10 mx-1"></div>
        <button
          class="p-1.5 rounded-md text-ink/30 hover:text-ink hover:bg-stone transition-colors"
          @click="$emit('close')"
        >
          <BaseIcon name="x" size="16" />
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
          <BaseIcon name="alertCircle" size="64" class="mb-6 text-ink/20" stroke-width="1.5" />
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
        <BaseIcon
          name="fileText"
          size="64"
          class="text-ink/20 group-hover:text-ink/40 transition-colors"
          stroke-width="1.5"
        />
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
import { defineAsyncComponent } from 'vue';
import { useResearchStore } from '../../stores/research';
import ImageViewer from './ImageViewer.vue';
import NoteEditor from './NoteEditor.vue';
import BaseIcon from '../base/BaseIcon.vue';

const PDFViewer = defineAsyncComponent(() => import('./PDFViewer.vue'));

const store = useResearchStore();

defineEmits(['add', 'close']);
</script>
