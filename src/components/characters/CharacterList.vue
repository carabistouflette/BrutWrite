<script setup lang="ts">
import type { Character } from '../../types';

defineProps<{
  characters: Character[];
  selectedId: string | null;
}>();

defineEmits(['select', 'create']);
</script>

<template>
  <div class="w-72 border-r border-ink/5 flex flex-col bg-stone/30">
    <div class="p-6 flex justify-between items-center">
      <h2 class="font-serif text-xl font-bold italic tracking-tight">Dramatis Personae</h2>
      <button
        class="p-2 hover:bg-accent/10 text-accent rounded-full transition-colors"
        title="Add Character"
        @click="$emit('create')"
      >
        <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 4v16m8-8H4"
          />
        </svg>
      </button>
    </div>

    <div class="flex-1 overflow-y-auto px-3 space-y-1 custom-scrollbar">
      <button
        v-for="char in characters"
        :key="char.id"
        class="w-full text-left px-4 py-3 rounded-xl transition-all duration-200 group relative border border-transparent"
        :class="
          selectedId === char.id
            ? 'bg-ink shadow-lg shadow-black/10 border-ink/10'
            : 'hover:bg-ink/5'
        "
        @click="$emit('select', char.id)"
      >
        <div
          class="font-medium transition-colors"
          :class="selectedId === char.id ? 'text-paper!' : 'text-ink group-hover:text-accent'"
        >
          {{ char.name }}
        </div>
        <div
          class="text-xs uppercase tracking-wider mt-0.5 flex justify-between transition-colors"
          :class="selectedId === char.id ? 'text-paper/60!' : 'text-ink/50'"
        >
          <span>{{ char.role }}</span>
          <span v-if="char.archetype" class="opacity-60 truncate ml-2 max-w-[80px]">{{
            char.archetype
          }}</span>
        </div>
      </button>

      <div v-if="characters.length === 0" class="text-center py-8 text-ink/30 italic text-sm">
        No characters yet.<br />Create one to start.
      </div>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: rgba(0, 0, 0, 0.1);
  border-radius: 3px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background-color: rgba(0, 0, 0, 0.2);
}
</style>
