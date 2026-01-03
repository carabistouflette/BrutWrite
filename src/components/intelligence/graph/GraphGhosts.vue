<script setup lang="ts">
import type { D3Node } from '../../../composables/domain/intelligence/CharacterGraphEngine';

interface Props {
  ghosts: D3Node[];
}

const props = defineProps<Props>();
</script>

<template>
  <aside
    class="absolute top-4 right-4 max-w-[200px] p-4 px-5 bg-linear-to-br from-purple-500/5 to-transparent border border-purple-500/10 rounded-2xl backdrop-blur-md"
  >
    <div class="flex items-center gap-2 mb-3">
      <h4 class="text-xs uppercase tracking-widest text-purple-700/60 font-bold">
        Unmapped Characters
      </h4>
      <div class="h-px flex-1 bg-purple-500/10"></div>
    </div>
    <ul class="space-y-1.5">
      <li
        v-for="ghost in props.ghosts"
        :key="ghost.id"
        class="text-sm text-gray-900/60 font-medium flex items-center gap-2"
      >
        <span class="w-1.5 h-1.5 rounded-full bg-purple-400/50"></span>
        {{ ghost.label }}
      </li>
    </ul>
  </aside>
</template>
