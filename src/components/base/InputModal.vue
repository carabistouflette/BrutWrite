<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';

const props = defineProps<{
  show: boolean;
  title?: string;
  message?: string;
  initialValue?: string;
  placeholder?: string;
  confirmLabel?: string;
  cancelLabel?: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'cancel'): void;
  (e: 'confirm', value: string): void;
}>();

const inputValue = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

watch(
  () => props.show,
  async (newShow) => {
    if (newShow) {
      inputValue.value = props.initialValue || '';
      await nextTick();
      if (inputRef.value) {
        inputRef.value.focus();
        inputRef.value.select();
      }
    }
  }
);

const close = () => {
  emit('cancel');
  emit('close');
};

const confirm = () => {
  if (!inputValue.value.trim()) return;
  emit('confirm', inputValue.value);
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
              {{ title || 'Input Required' }}
            </h3>
          </div>

          <p v-if="message" class="text-ink/80 mb-4 leading-relaxed text-sm">
            {{ message }}
          </p>

          <form @submit.prevent="confirm">
            <input
              ref="inputRef"
              v-model="inputValue"
              type="text"
              class="w-full px-4 py-2 bg-white/50 border border-black/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-accent text-ink placeholder-ink/40 mb-6 transition-all"
              :placeholder="placeholder"
              @keydown.esc="close"
            />

            <div class="flex justify-end gap-3">
              <button
                type="button"
                class="px-4 py-2 rounded-lg text-sm font-medium text-ink/60 hover:bg-black/5 hover:text-ink transition-colors"
                @click="close"
              >
                {{ cancelLabel || 'Cancel' }}
              </button>
              <button
                type="submit"
                class="px-5 py-2 rounded-lg text-sm font-medium bg-accent text-white hover:bg-accent-dark shadow-lg shadow-accent/20 transition-all"
                :disabled="!inputValue.trim()"
              >
                {{ confirmLabel || 'Confirm' }}
              </button>
            </div>
          </form>
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
