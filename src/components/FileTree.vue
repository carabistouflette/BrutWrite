<script setup lang="ts">
import { VueDraggableNext } from 'vue-draggable-next';
import { computed, ref } from 'vue';
import type { FileNode } from '../types';

defineOptions({
  name: 'FileTree'
});

const props = withDefaults(defineProps<{
  modelValue: FileNode[],
  activeId?: string,
  depth?: number,
  editingId?: string | null
}>(), {
  depth: 0,
  editingId: null
});

const isActive = (id: string) => id === props.activeId;

const emit = defineEmits<{
  (e: 'update:modelValue', value: FileNode[]): void;
  (e: 'select', id: string): void;
  (e: 'delete', id: string): void;
  (e: 'context-menu', payload: { e: MouseEvent, id: string }): void;
  (e: 'request-rename', id: string): void;
  (e: 'submit-rename', payload: { id: string, name: string }): void;
  (e: 'cancel-rename'): void;
}>();

const list = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
});

const editName = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

// Watch for editingId change to focus input
import { watch, nextTick } from 'vue';
watch(() => props.editingId, async (newVal) => {
  if (newVal) {
    const node = props.modelValue.find(n => n.id === newVal);
    if (node) {
      editName.value = node.name;
      await nextTick();
      if (inputRef.value) inputRef.value.focus(); 
      // Focus logic might need to handle v-for refs which return arrays
      // But since we render only one input at a time, we can use a function ref or just check the dom
    }
  }
});

const handleRenameSubmit = (id: string) => {
  if (editName.value.trim()) {
    emit('submit-rename', { id, name: editName.value });
  } else {
    emit('cancel-rename');
  }
};

const handleRenameCancel = () => {
    emit('cancel-rename');
}

const hoverId = ref<string | null>(null);

const handleSelect = (id: string) => {
  emit('select', id);
};

const deleteItem = (id: string) => {
  emit('delete', id);
}
</script>

<template>
  <VueDraggableNext
    v-model="list"
    tag="transition-group"
    :component-data="{ tag: 'div', name: 'list' }"
    group="files"
    :animation="200" 
    ghost-class="ghost"
    class="min-h-[10px] relative block"
  >
    <div
      v-for="element in list"
      :key="element.id"
      class="cursor-pointer select-none group/row"
      @mouseenter="hoverId = element.id"
      @mouseleave="hoverId = null"
    >
      <div 
        class="group relative flex justify-between items-center py-2 px-3 transition-all duration-300 ease-out active:scale-[0.98]"
        :class="{ 'active-pop': isActive(element.id) }"
        @click.stop="handleSelect(element.id)"
        @contextmenu.prevent="(e) => emit('context-menu', { e, id: element.id })"
      >
        <!-- Soft Background Highlight on Hover with scaling -->
        <div 
          class="absolute inset-0 bg-stone/20 transition-all duration-300 rounded-lg mx-1 -z-0"
          :class="hoverId === element.id && !isActive(element.id) ? 'opacity-100 scale-[1.02]' : 'opacity-0 scale-100'"
        ></div>

        <!-- Active Background Block (Smooth Animated Frame) -->
        <transition
          enter-active-class="transition-all duration-300 cubic-bezier(0.25, 0.8, 0.25, 1)"
          enter-from-class="opacity-0 scale-95"
          enter-to-class="opacity-100 scale-100"
          leave-active-class="transition-all duration-200 cubic-bezier(0.25, 0.8, 0.25, 1)"
          leave-from-class="opacity-100 scale-100"
          leave-to-class="opacity-0 scale-95"
        >
          <div 
             v-if="isActive(element.id)"
             class="absolute inset-0 bg-accent/5 border border-accent/30 rounded-lg mx-1 -z-0 shadow-[0_2px_12px_rgba(255,95,31,0.05)]"
          ></div>
        </transition>

        <div class="flex items-center gap-3 overflow-hidden z-10 flex-1 min-w-0 pr-8">
            <template v-if="editingId === element.id">
                <input
                    ref="inputRef"
                    v-model="editName"
                    @blur="handleRenameSubmit(element.id)"
                    @keydown.enter="handleRenameSubmit(element.id)"
                    @keydown.escape="handleRenameCancel"
                    @click.stop
                    class="bg-transparent border-b border-accent text-[14.5px] leading-tight text-ink w-full focus:outline-none"
                />
            </template>
            <span v-else 
                  class="text-[14.5px] leading-tight transition-all duration-500 flex-1 truncate select-none"
                  :class="{ 
                    'font-bold text-ink tracking-tight': depth === 0,
                    'font-medium text-ink/90': depth > 0 && isActive(element.id),
                    'font-normal text-ink/40': depth > 0 && !isActive(element.id),
                    'translate-x-1.5 text-ink/90': hoverId === element.id
                  }"
                  @dblclick.stop="emit('request-rename', element.id)">
              {{ element.name }}
            </span>
        </div>
        
        <div 
          class="transition-all duration-300 flex items-center z-20 absolute right-2"
          :class="hoverId === element.id ? 'opacity-100 translate-x-0' : 'opacity-0 translate-x-1 pointer-events-none'"
        >
             <!-- Elegant Delete Button with pop effect -->
            <button 
                @click.stop="deleteItem(element.id)"
                class="w-8 h-8 flex items-center justify-center text-ink/20 hover:text-red-500 hover:bg-white border border-transparent hover:border-black/5 rounded-full transition-all duration-200 shadow-none hover:shadow-md hover:scale-110 active:scale-90"
                title="Delete"
            >
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M18 6 6 18"/><path d="m6 6 12 12"/>
                </svg>
            </button>
        </div>
      </div>

      <!-- Recursive Nesting -->
      <transition
        enter-active-class="transition-all duration-500 ease-out"
        enter-from-class="opacity-0 -translate-y-2 scale-95"
        enter-to-class="opacity-100 translate-y-0 scale-100"
      >
        <div 
          v-if="element.children && element.children.length > 0"
          class="ml-6 pl-4 border-l border-ink/5 group-hover/item:border-ink/15 transition-all duration-500 ease-in-out"
        >
          <FileTree 
            v-model="element.children"
            :active-id="activeId"
            :editing-id="editingId"
            :depth="depth + 1"
            @select="handleSelect" 
            @delete="(id) => emit('delete', id)"
            @context-menu="(payload) => emit('context-menu', payload)"
            @request-rename="(id) => emit('request-rename', id)"
            @submit-rename="(payload) => emit('submit-rename', payload)"
            @cancel-rename="emit('cancel-rename')"
          />
        </div>
      </transition>
    </div>
  </VueDraggableNext>
</template>

<style scoped lang="postcss">
@reference "../style.css";

.ghost {
  opacity: 0.3;
  background-color: var(--color-stone);
  transform: scale(0.95);
  transition: transform 0.2s ease;
}

@keyframes spring-pop {
  0% { transform: scale(1); }
  50% { transform: scale(0.97); }
  100% { transform: scale(1); }
}

.active-pop {
  animation: spring-pop 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

/* List Transitions */
.list-move {
  transition: all 0.6s cubic-bezier(0.19, 1, 0.22, 1);
}

.list-enter-active {
  animation: elegant-entry 0.5s cubic-bezier(0.2, 0.8, 0.2, 1) both;
  z-index: 10;
}

.list-leave-active {
  transition: all 0.4s cubic-bezier(0.33, 1, 0.68, 1); /* Faster exit */
  position: absolute;
  width: 100%;
  z-index: 0;
}

.list-leave-to {
  opacity: 0;
  transform: scale(0.9) translateY(10px);
  filter: blur(4px);
}

@keyframes elegant-entry {
  0% {
    opacity: 0;
    transform: translateY(20px) scale(0.96);
  }
  60% {
    opacity: 1;
    transform: translateY(-2px) scale(1.01);
  }
  100% {
    transform: translateY(0) scale(1);
  }
}
</style>
