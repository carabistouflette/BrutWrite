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
        <div
          class="absolute inset-0 bg-black/60 shadow-[inset_0_0_100px_rgba(0,0,0,0.5)]"
          @click="close"
        ></div>

        <!-- Modal Window -->
        <div
          class="relative w-full max-w-md bg-[var(--paper)] border border-[var(--stone)] shadow-[var(--shadow-brut)] text-[var(--ink)] modal-container p-6 animate-in fade-in zoom-in-95 duration-200"
          role="dialog"
          aria-modal="true"
        >
          <div class="mb-6 border-b border-[var(--stone)] pb-2">
            <h3 class="text-xl font-bold font-serif italic text-[var(--ink)]">
              {{ title || 'Confirm Action' }}
            </h3>
          </div>

          <p class="text-[var(--ink)] opacity-80 mb-8 leading-relaxed font-sans">
            {{ message || 'Are you sure you want to proceed?' }}
          </p>

          <div class="flex justify-end gap-3">
            <button
              class="px-4 py-2 text-sm font-bold font-mono text-[var(--ink)] border border-transparent hover:border-[var(--ink)] transition-colors"
              @click="close"
            >
              {{ cancelLabel || 'Cancel' }}
            </button>
            <button
              class="px-5 py-2 text-sm font-bold font-mono shadow-[2px_2px_0_0_black] transition-transform active:translate-y-px active:shadow-none border border-black"
              :class="
                isDestructive
                  ? 'bg-red-600 text-white hover:bg-red-700'
                  : 'bg-[var(--ink)] text-[var(--paper)] hover:opacity-90'
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
.modal-container {
  box-shadow: 8px 8px 0px 0px rgba(0, 0, 0, 1);
}
</style>
