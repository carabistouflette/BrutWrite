<script setup lang="ts">
import { computed } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/core';

const props = defineProps<{
  path: string;
}>();

const src = computed(() => convertFileSrc(props.path));
</script>

<template>
  <div class="flex-1 flex items-center justify-center bg-surface-200 overflow-hidden p-8">
    <div class="relative group">
      <div
        class="absolute inset-0 bg-ink translate-x-2 translate-y-2 group-hover:translate-x-3 group-hover:translate-y-3 transition-transform"
      ></div>
      <img
        :src="src"
        class="relative border-4 border-ink max-w-full max-h-[calc(100vh-200px)] object-contain bg-white transition-transform group-hover:-translate-x-1 group-hover:-translate-y-1"
      />
    </div>
  </div>
</template>
