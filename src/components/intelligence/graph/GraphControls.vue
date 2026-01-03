<script setup lang="ts">
const emit = defineEmits<{
  (e: 'zoomIn'): void;
  (e: 'zoomOut'): void;
  (e: 'reset'): void;
}>();
</script>

<template>
  <div
    class="absolute top-4 left-4 flex items-center gap-1 p-1 bg-white border border-black/10 rounded-lg shadow-sm"
  >
    <button
      class="flex items-center justify-center w-7 h-7 rounded text-black/60 transition-all hover:bg-black/5 hover:text-gray-900"
      title="Zoom in"
      @click="emit('zoomIn')"
    >
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
    </button>
    <button
      class="flex items-center justify-center w-7 h-7 rounded text-black/60 transition-all hover:bg-black/5 hover:text-gray-900"
      title="Zoom out"
      @click="emit('zoomOut')"
    >
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
      </svg>
    </button>
    <button
      class="flex items-center justify-center w-7 h-7 rounded text-black/60 transition-all hover:bg-black/5 hover:text-gray-900"
      title="Reset zoom"
      @click="emit('reset')"
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
  </div>
</template>
