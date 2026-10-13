<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue';
import { readFile } from '@tauri-apps/plugin-fs';

const props = defineProps<{
  path: string;
}>();

const src = ref('');
const error = ref<string | null>(null);
let generation = 0;

watch(
  () => props.path,
  async (path) => {
    const request = ++generation;
    if (src.value) URL.revokeObjectURL(src.value);
    src.value = '';
    error.value = null;
    try {
      const data = await readFile(path);
      if (request !== generation) return;
      const extension = path.split('.').pop()?.toLowerCase();
      const type =
        extension === 'jpg' || extension === 'jpeg' ? 'image/jpeg' : `image/${extension}`;
      src.value = URL.createObjectURL(new Blob([data], { type }));
    } catch (cause) {
      if (request === generation) {
        error.value = `Failed to load image: ${cause instanceof Error ? cause.message : String(cause)}`;
      }
    }
  },
  { immediate: true }
);

onBeforeUnmount(() => {
  generation += 1;
  if (src.value) URL.revokeObjectURL(src.value);
});
</script>

<template>
  <div class="flex-1 flex items-center justify-center bg-stone/20 overflow-hidden p-8">
    <p v-if="error" role="alert" class="text-red-500">{{ error }}</p>
    <div v-else-if="src" class="relative group">
      <div
        class="absolute inset-0 bg-ink translate-x-2 translate-y-2 opacity-10 group-hover:translate-x-3 group-hover:translate-y-3 transition-transform"
      ></div>
      <img
        :src="src"
        alt="Research image"
        class="relative border-4 border-ink max-w-full max-h-[calc(100vh-200px)] object-contain bg-paper transition-transform group-hover:-translate-x-1 group-hover:-translate-y-1 shadow-2xl"
      />
    </div>
  </div>
</template>
