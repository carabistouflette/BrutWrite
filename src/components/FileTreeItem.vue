<script lang="ts">
const numberFormatter = new Intl.NumberFormat();
</script>

<script setup lang="ts">
import { ref } from 'vue';
import type { FileNode } from '../types';
import { useProjectData } from '../composables/logic/useProjectData';

const props = defineProps<{
  element: FileNode;
  isActive: boolean;
  isEditing: boolean;
  depth: number;
  editName: string;
}>();

const emit = defineEmits<{
  (e: 'update:editName', value: string): void;
  (e: 'context-menu', payload: { e: MouseEvent; id: string }): void;
  (e: 'submit-rename', id: string): void;
  (e: 'cancel-rename'): void;
  (e: 'request-rename', id: string): void;
}>();

const { selectNode, deleteNode } = useProjectData();
const inputRef = ref<HTMLInputElement | null>(null);

defineExpose({
  focus: () => inputRef.value?.focus(),
});
</script>

<template>
  <div
    class="group relative flex justify-between items-center py-2 px-3 transition-all duration-300 ease-out active:scale-[0.98]"
    :class="{ 'active-pop': isActive }"
    @click.stop="selectNode(element.id)"
    @contextmenu.prevent="(e) => emit('context-menu', { e, id: element.id })"
  >
    <!-- Soft Background Highlight on Hover -->
    <div
      class="absolute inset-0 bg-stone/20 transition-all duration-300 rounded-lg mx-1 z-0 opacity-0 scale-100 group-hover/row:opacity-100 group-hover/row:scale-[1.02]"
      :class="{ 'opacity-0! scale-100!': isActive }"
    ></div>

    <!-- Active Background Block -->
    <transition
      enter-active-class="transition-all duration-300 cubic-bezier(0.25, 0.8, 0.25, 1)"
      enter-from-class="opacity-0 scale-95"
      enter-to-class="opacity-100 scale-100"
      leave-active-class="transition-all duration-200 cubic-bezier(0.25, 0.8, 0.25, 1)"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-95"
    >
      <div
        v-if="isActive"
        class="absolute inset-0 bg-accent/5 border border-accent/30 rounded-lg mx-1 z-0 shadow-[0_2px_12px_rgba(255,95,31,0.05)]"
      ></div>
    </transition>

    <div class="flex items-center gap-3 overflow-hidden z-10 flex-1 min-w-0 pr-8">
      <template v-if="isEditing">
        <input
          ref="inputRef"
          :value="editName"
          class="bg-transparent border-b border-accent text-[14.5px] leading-tight text-ink w-full focus:outline-none"
          @input="(e) => emit('update:editName', (e.target as HTMLInputElement).value)"
          @blur="emit('submit-rename', element.id)"
          @keydown.enter="emit('submit-rename', element.id)"
          @keydown.escape="emit('cancel-rename')"
          @click.stop
        />
      </template>
      <div v-else class="flex flex-1 items-center min-w-0 gap-2">
        <span
          class="text-[14.5px] leading-tight transition-all duration-500 truncate select-none group-hover/row:translate-x-1.5"
          :class="{
            'font-bold text-ink tracking-tight': depth === 0,
            'font-medium text-ink/90': depth > 0 && isActive,
            'font-normal text-ink/40': depth > 0 && !isActive,
            'text-ink/90': isActive,
          }"
          @dblclick.stop="emit('request-rename', element.id)"
        >
          {{ element.name }}
        </span>
        <span
          v-if="element.word_count && element.word_count > 0"
          class="text-[10px] bg-stone/50 text-ink/40 px-1.5 rounded-full mt-0.5 group-hover/row:translate-x-1.5 transition-transform duration-500"
        >
          {{ numberFormatter.format(element.word_count) }}
        </span>
      </div>
    </div>

    <div
      class="transition-all duration-300 flex items-center z-20 absolute right-2 opacity-0 translate-x-1 pointer-events-none group-hover/row:opacity-100 group-hover/row:translate-x-0 group-hover/row:pointer-events-auto"
    >
      <button
        class="w-8 h-8 flex items-center justify-center text-ink/20 hover:text-red-500 hover:bg-white border border-transparent hover:border-black/5 rounded-full transition-all duration-200 shadow-none hover:shadow-md hover:scale-110 active:scale-90"
        title="Delete"
        @click.stop="deleteNode(element.id)"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M18 6 6 18" />
          <path d="m6 6 12 12" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@keyframes spring-pop {
  0% {
    transform: scale(1);
  }
  50% {
    transform: scale(0.97);
  }
  100% {
    transform: scale(1);
  }
}

.active-pop {
  animation: spring-pop 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}
</style>
