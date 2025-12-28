<script setup lang="ts">
/**
 * CharacterGraphModal.vue
 *
 * Modal wrapper for the Character Graph visualization.
 */

import { defineAsyncComponent } from 'vue';

const CharacterGraph = defineAsyncComponent(() => import('./CharacterGraph.vue'));

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const close = () => {
  emit('close');
};
</script>

<template>
  <Teleport to="#app-scale-root">
    <Transition name="dialog">
      <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4 lg:p-8">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/40" @click="close"></div>

        <!-- Modal Container -->
        <div
          class="relative w-full max-w-5xl h-[80%] flex flex-col bg-paper/95 backdrop-blur-xl border border-white/40 shadow-2xl rounded-2xl overflow-hidden modal-container"
          style="
            box-shadow:
              0 20px 50px -12px rgba(0, 0, 0, 0.25),
              0 0 0 1px rgba(255, 255, 255, 0.4) inset;
          "
        >
          <!-- Header -->
          <header class="px-6 py-4 border-b border-stone/50 flex justify-between items-center">
            <div>
              <h2 class="font-serif text-xl font-bold italic">Character Dynamics</h2>
              <p class="text-xs text-ink/40 mt-0.5">Narrative gravity visualization</p>
            </div>
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
          </header>

          <!-- Graph Container -->
          <div class="flex-1 overflow-hidden">
            <CharacterGraph :width="900" :height="500" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
/* Dialog Transition */
.dialog-enter-active,
.dialog-leave-active {
  transition: opacity 0.2s ease-out;
}

.dialog-enter-from,
.dialog-leave-to {
  opacity: 0;
}

.dialog-enter-active .modal-container,
.dialog-leave-active .modal-container {
  transition:
    transform 0.3s cubic-bezier(0.16, 1, 0.3, 1),
    opacity 0.3s ease-out;
}

.dialog-enter-from .modal-container,
.dialog-leave-to .modal-container {
  transform: translateY(8px);
  opacity: 0;
}

.modal-container {
  transform: translateZ(0);
  backface-visibility: hidden;
}
</style>
