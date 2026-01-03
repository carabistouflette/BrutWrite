<script setup lang="ts">
import type { D3Node } from '../../../composables/domain/intelligence/CharacterGraphEngine';

interface Props {
  node: D3Node;
  x: number;
  y: number;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'copy', node: D3Node): void;
}>();
</script>

<template>
  <Teleport to="body">
    <div
      class="fixed inset-0 z-90"
      @click="emit('close')"
      @contextmenu.prevent="emit('close')"
    ></div>
    <div
      class="fixed z-max p-2 bg-white border border-black/10 rounded-xl shadow-2xl min-w-[180px]"
      :style="{ left: `${props.x}px`, top: `${props.y}px` }"
    >
      <button
        class="flex items-center gap-2 w-full p-2 px-3 text-xs font-medium text-gray-900 rounded-lg transition-all text-left hover:bg-orange-500 hover:text-white"
        @click="emit('copy', props.node)"
      >
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002 2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2 2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3"
          />
        </svg>
        Copy @{{ props.node.label }}
      </button>
    </div>
  </Teleport>
</template>
