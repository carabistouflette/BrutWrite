<script setup lang="ts">
import { VueDraggableNext } from 'vue-draggable-next';
import { ref, watch, nextTick } from 'vue';
import { useDragState } from '../composables/useDragState';
import type { FileNode } from '../types';
import FileTreeItem from './FileTreeItem.vue';

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

const emit = defineEmits<{
  (e: 'update:modelValue', value: FileNode[]): void;
  (e: 'select', id: string): void;
  (e: 'delete', id: string): void;
  (e: 'context-menu', payload: { e: MouseEvent, id: string }): void;
  (e: 'request-rename', id: string): void;
  (e: 'submit-rename', payload: { id: string, name: string }): void;
  (e: 'cancel-rename'): void;
}>();

const localList = ref<FileNode[]>([...props.modelValue]);

watch(() => props.modelValue, (newVal) => {
  localList.value = [...newVal];
});

const handleUpdate = (newList: FileNode[]) => {
  localList.value = newList;
  emit('update:modelValue', newList);
};

const handleNestedUpdate = (index: number, newChildren: FileNode[]) => {
  const updatedList = [...localList.value];
  updatedList[index] = { ...updatedList[index], children: newChildren };
  localList.value = updatedList;
  emit('update:modelValue', updatedList);
};

const editName = ref('');
const itemRefs = ref<any[]>([]);
const { isDragging, setDragging } = useDragState();

const isActive = (id: string) => id === props.activeId;

watch(() => props.editingId, async (newVal) => {
  if (newVal) {
    const node = props.modelValue.find(n => n.id === newVal);
    if (node) {
      editName.value = node.name;
      await nextTick();
      const item = itemRefs.value.find((ref: any) => ref?.element?.id === newVal);
      if (item && item.focus) item.focus();
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
</script>

<template>
  <VueDraggableNext
    :model-value="localList"
    @update:model-value="handleUpdate"
    group="files"
    :animation="200" 
    ghost-class="ghost"
    :force-fallback="true"
    :swap-threshold="0.9"
    class="min-h-[10px] relative block"
    tag="div"
    @start="setDragging(true)"
    @end="setDragging(false)"
  >
    <div
      v-for="(element, index) in localList"
      :key="element.id"
      class="cursor-pointer select-none group/row"
    >
      <FileTreeItem
        ref="itemRefs"
        :element="element"
        :is-active="isActive(element.id)"
        :is-editing="editingId === element.id"
        :depth="depth"
        v-model:edit-name="editName"
        @select="(id) => emit('select', id)"
        @context-menu="(p) => emit('context-menu', p)"
        @delete="(id) => emit('delete', id)"
        @submit-rename="handleRenameSubmit"
        @cancel-rename="emit('cancel-rename')"
        @request-rename="(id) => emit('request-rename', id)"
      />

      <transition
        enter-active-class="transition-all duration-300 ease-out"
        enter-from-class="opacity-0 -translate-y-2"
        enter-to-class="opacity-100 translate-y-0"
      >
        <div 
          v-if="element.children"
          v-show="isDragging || element.children.length > 0"
          class="ml-6 pl-4 border-l transition-all duration-300 ease-in-out"
          :class="[
            isDragging && element.children.length === 0 
              ? 'py-3 border-dashed border-accent/40 bg-accent/5 rounded-r-lg my-1' 
              : 'border-ink/5 hover:border-ink/15'
          ]"
        >
          <FileTree 
            :model-value="element.children"
            @update:model-value="(val) => handleNestedUpdate(index, val)"
            :active-id="activeId"
            :editing-id="editingId"
            :depth="depth + 1"
            @select="(id) => emit('select', id)" 
            @delete="(id) => emit('delete', id)"
            @context-menu="(p) => emit('context-menu', p)"
            @request-rename="(id) => emit('request-rename', id)"
            @submit-rename="(p) => emit('submit-rename', p)"
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
