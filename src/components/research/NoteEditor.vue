<template>
  <div class="flex flex-col h-full bg-zinc-900/50">
    <!-- Toolbar -->
    <div class="flex items-center justify-between px-4 py-2 border-b border-white/5 bg-zinc-900/50">
      <div class="flex items-center gap-2">
        <span class="text-xs font-bold uppercase tracking-widest text-zinc-500">Editing</span>
        <span class="text-sm font-medium text-zinc-300">{{ store.activeArtifact?.name }}</span>
      </div>
      <div class="flex items-center gap-2">
        <span v-if="saving" class="text-xs text-zinc-500 animate-pulse">Saving...</span>
        <span v-else-if="saved" class="text-xs text-green-500">Saved</span>
      </div>
    </div>

    <!-- Editor Area -->
    <textarea
      v-model="content"
      class="flex-1 w-full bg-transparent p-6 resize-none outline-none text-zinc-300 font-mono text-sm leading-relaxed custom-scrollbar placeholder-zinc-700"
      placeholder="Start typing your note here..."
      @input="handleInput"
    ></textarea>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUpdated, onUnmounted } from 'vue';
import { useResearchStore } from '../../stores/research';
import { readFile } from '@tauri-apps/plugin-fs';

const props = defineProps<{
  path: string;
  id: string; // Artifact ID
}>();

const store = useResearchStore();
const content = ref('');
const saving = ref(false);
const saved = ref(false);
let saveTimeout: ReturnType<typeof setTimeout>;

onMounted(() => {});
onUpdated(() => {});
onUnmounted(() => {});

const loadContent = async () => {
  try {
    const raw = await readFile(props.path);
    content.value = new TextDecoder().decode(raw);
  } catch (e) {
    console.error('Failed to load note content', e);
  }
};

const handleInput = () => {
  saved.value = false;
  saving.value = true;
  clearTimeout(saveTimeout);
  saveTimeout = setTimeout(async () => {
    await store.saveNoteContent(props.id, content.value);
    saving.value = false;
    saved.value = true;
    setTimeout(() => (saved.value = false), 2000);
  }, 1000); // Auto-save after 1s
};

watch(() => props.path, loadContent, { immediate: true });
</script>

<style scoped>
/* Ultra-minimal scrollbar */
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 99px;
}
</style>
