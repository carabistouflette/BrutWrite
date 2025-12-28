<script setup lang="ts">
/**
 * CharacterGraphModal.vue
 *
 * Modal wrapper for the Character Graph visualization.
 * Styled to match CharacterSheet's warm, editorial aesthetic.
 */

import { defineAsyncComponent } from 'vue';
import { useProjectStore } from '../../stores/project';

const CharacterGraph = defineAsyncComponent(() => import('./CharacterGraph.vue'));

defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const projectStore = useProjectStore();

const close = () => {
  emit('close');
};

/**
 * Navigate to a specific chapter when double-clicking a character node.
 */
const handleNavigateToMention = (chapterId: string) => {
  projectStore.setActiveId(chapterId);
  close();
};
</script>

<template>
  <Teleport to="#app-scale-root">
    <Transition
      enter-active-class="transition duration-300 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition duration-200 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4 lg:p-8">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" @click="close"></div>

        <!-- Modal Container -->
        <div
          class="relative w-full max-w-6xl h-[85%] flex flex-col bg-paper/95 backdrop-blur-2xl border border-white/20 shadow-2xl rounded-2xl overflow-hidden text-ink modal-container"
          style="box-shadow: 0 24px 60px -12px rgba(0, 0, 0, 0.4)"
        >
          <!-- Header -->
          <header
            class="px-8 py-4 border-b border-ink/5 flex justify-between items-center bg-paper/50"
          >
            <div>
              <h2 class="text-2xl font-serif font-bold italic text-ink">Character Dynamics</h2>
              <p class="text-xs uppercase tracking-widest text-ink/40 font-bold mt-0.5">
                Narrative Gravity Visualization
              </p>
            </div>
            <div class="flex items-center gap-4">
              <span class="text-xs text-ink/30 hidden sm:inline">
                Double-click a character to jump to their first mention
              </span>
              <button
                class="w-8 h-8 flex items-center justify-center rounded-full hover:bg-black/5 text-ink/40 hover:text-ink transition-colors"
                @click="close"
              >
                <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            </div>
          </header>

          <!-- Graph Container -->
          <div class="flex-1 overflow-hidden bg-transparent">
            <CharacterGraph
              :width="1000"
              :height="550"
              @navigate-to-mention="handleNavigateToMention"
            />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-container {
  animation: modal-pop 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes modal-pop {
  0% {
    transform: scale(0.95);
    opacity: 0;
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}
</style>
