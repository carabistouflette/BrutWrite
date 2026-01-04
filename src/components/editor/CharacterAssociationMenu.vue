<script setup lang="ts">
import { computed, ref } from 'vue';
import type { Character } from '../../types';
import ContextMenu from '../base/ContextMenu.vue';

const props = defineProps<{
  x: number;
  y: number;
  selectedText: string;
  characters: Character[];
}>();

const emit = defineEmits<{
  (e: 'select-character', characterId: string): void;
  (e: 'remove-association'): void;
  (e: 'close'): void;
}>();

// State for sub-menu navigation
const mode = ref<'main' | 'list'>('main');

// Check if current text is already associated with any character
const existingAssociation = computed(() => {
  const norm = props.selectedText.trim().toLowerCase();
  return props.characters.find((c) => (c.aliases || []).some((a) => a.toLowerCase() === norm));
});

const isAssociated = computed(() => !!existingAssociation.value);

const handleAssociateClick = () => {
  mode.value = 'list';
};

const handleRemoveClick = () => {
  emit('remove-association');
};

const handleBack = () => {
  mode.value = 'main';
};
</script>

<template>
  <ContextMenu :show="true" :x="x" :y="y" @close="emit('close')">
    <div class="px-4 py-2 border-b border-ink/5 mb-1 bg-paper-darker/50">
      <div
        class="text-[10px] uppercase tracking-widest text-ink/40 font-bold truncate max-w-[200px]"
      >
        "{{ selectedText }}"
      </div>
    </div>

    <!-- MAIN MENU -->
    <div v-if="mode === 'main'" class="min-w-[200px]">
      <!-- Option 1: Associate -->
      <button
        class="menu-item w-full text-left flex items-center justify-between group"
        @click.stop="handleAssociateClick"
      >
        <span>Associate with...</span>
        <svg
          class="w-4 h-4 text-ink/20 group-hover:text-ink/60"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
        </svg>
      </button>

      <!-- Option 2: Remove (Conditional) -->
      <button
        class="menu-item w-full text-left flex items-center gap-2 group"
        :class="isAssociated ? 'text-red-500 hover:bg-red-50' : 'text-ink/20 cursor-not-allowed'"
        :disabled="!isAssociated"
        @click.stop="handleRemoveClick"
      >
        <span>Remove Reference</span>
        <span v-if="existingAssociation" class="text-[10px] text-ink/40 ml-auto">
          ({{ existingAssociation.name }})
        </span>
      </button>
    </div>

    <!-- CHARACTER LIST -->
    <div v-else class="min-w-[200px] flex flex-col">
      <div class="px-2 py-1 mb-1 border-b border-ink/5 flex items-center">
        <button
          class="p-1 -ml-1 hover:bg-ink/5 rounded text-ink/40 hover:text-ink transition-colors"
          @click.stop="handleBack"
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M15 19l-7-7 7-7"
            />
          </svg>
        </button>
        <span class="text-xs font-bold text-ink/40 ml-2">Select Character</span>
      </div>

      <div class="max-h-60 overflow-y-auto custom-scrollbar">
        <button
          v-for="char in characters"
          :key="char.id"
          class="menu-item menu-item-default w-full text-left flex flex-col group mb-1"
          @click="emit('select-character', char.id)"
        >
          <span class="font-serif font-bold text-sm">{{ char.name }}</span>
          <span class="text-[10px] text-ink/40 group-hover:text-white/80 capitalize">{{
            char.role
          }}</span>
        </button>

        <div v-if="characters.length === 0" class="px-4 py-2 text-ink/40 italic text-xs">
          No characters created yet.
        </div>
      </div>
    </div>
  </ContextMenu>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.1);
  border-radius: 2px;
}
</style>
