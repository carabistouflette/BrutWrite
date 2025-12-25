<template>
  <div class="flex flex-col h-full bg-paper">
    <!-- Toolbar -->
    <div class="flex items-center justify-between px-4 py-2 border-b border-ink/5 bg-stone/40">
      <div class="flex items-center gap-2">
        <span class="text-[10px] font-bold uppercase tracking-widest text-ink/40">Editing</span>
        <span class="text-sm font-medium text-ink">{{ store.activeArtifact?.name }}</span>
      </div>
      <div class="flex items-center gap-2">
        <span
          v-if="saving"
          class="text-[10px] uppercase font-bold tracking-tighter text-accent animate-pulse"
          >Saving...</span
        >
        <span
          v-else-if="saved"
          class="text-[10px] uppercase font-bold tracking-tighter text-green-600"
          >Saved</span
        >
      </div>
    </div>

    <!-- Editor Area -->
    <textarea
      v-model="content"
      class="flex-1 w-full bg-transparent p-6 resize-none outline-none text-ink font-mono text-sm leading-relaxed custom-scrollbar placeholder-ink/20"
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
  background: var(--stone);
  border-radius: 99px;
}
.custom-scrollbar:hover::-webkit-scrollbar-thumb {
  background: var(--ink-rgb);
  opacity: 0.2;
}
</style>
