<script setup lang="ts">
const props = defineProps<{
  show: boolean;
  title?: string;
  message?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  isDestructive?: boolean;
}>();

const emit = defineEmits(['close', 'confirm', 'cancel']);

const close = () => {
  emit('cancel');
  emit('close');
};

const confirm = () => {
  emit('confirm');
  close();
};
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-300 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition duration-200 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div v-if="show" class="fixed inset-0 z-60 flex items-center justify-center p-4">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/40 backdrop-blur-sm" @click="close"></div>

        <!-- Modal Window -->
        <div
          class="relative w-full max-w-md bg-paper/90 backdrop-blur-xl border border-white/40 shadow-2xl rounded-2xl overflow-hidden text-ink modal-container p-6"
          role="dialog"
          aria-modal="true"
        >
          <div class="mb-4">
            <h3 class="text-lg font-bold font-serif italic text-ink">
              {{ title || 'Confirm Action' }}
            </h3>
          </div>

          <p class="text-ink/80 mb-8 leading-relaxed">
            {{ message || 'Are you sure you want to proceed?' }}
          </p>

          <div class="flex justify-end gap-3">
            <button
              class="px-4 py-2 rounded-lg text-sm font-medium text-ink/60 hover:bg-black/5 hover:text-ink transition-colors"
              @click="close"
            >
              {{ cancelLabel || 'Cancel' }}
            </button>
            <button
              class="px-5 py-2 rounded-lg text-sm font-medium shadow-lg transition-all"
              :class="
                isDestructive
                  ? 'bg-red-500 text-white hover:bg-red-600 shadow-red-500/20'
                  : 'bg-accent text-white hover:bg-accent-dark shadow-accent/20'
              "
              @click="confirm"
            >
              {{ confirmLabel || 'Confirm' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
@keyframes modal-pop {
  0% {
    transform: scale(0.95) translateY(10px);
    opacity: 0;
  }
  100% {
    transform: scale(1) translateY(0);
    opacity: 1;
  }
}

.modal-container {
  animation: modal-pop 0.3s cubic-bezier(0.34, 1.56, 0.64, 1) forwards;
  box-shadow:
    0 20px 50px -12px rgba(0, 0, 0, 0.3),
    0 0 0 1px rgba(255, 255, 255, 0.2) inset;
}
</style>
