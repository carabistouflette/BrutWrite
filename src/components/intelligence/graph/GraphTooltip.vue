<script setup lang="ts">
import type { D3Node } from '../../../utils/graphs/CharacterGraphEngine';

interface Props {
  node: D3Node;
  x: number;
  y: number;
  roleName: string;
}

const props = defineProps<Props>();
</script>

<template>
  <div
    class="absolute transform -translate-x-1/2 -translate-y-full p-3 px-4 bg-white border border-black/10 rounded-xl shadow-lg pointer-events-none z-tooltip min-w-[150px]"
    :style="{ left: `${props.x}px`, top: `${props.y}px` }"
  >
    <div class="font-serif text-sm font-semibold italic text-gray-900 mb-0.5">
      {{ props.node.label }}
    </div>
    <div class="text-[10px] font-bold uppercase tracking-widest text-orange-500 mb-2">
      {{ props.roleName }}
    </div>
    <div class="flex gap-2 text-xs text-black/60">
      <span>{{ props.node.mentionCount }} mentions</span>
    </div>
    <div class="text-[10px] text-black/40 mt-1.5">Valence: {{ props.node.valence.toFixed(2) }}</div>
  </div>
</template>
