<script setup lang="ts">
import { VueDraggableNext } from 'vue-draggable-next';
import { computed } from 'vue';
import type { FileNode } from '../types';

defineOptions({
  name: 'FileTree'
});

const props = defineProps<{
  modelValue: FileNode[],
  activeId?: string
}>();

const isActive = (id: string) => id === props.activeId;

const emit = defineEmits<{
  (e: 'update:modelValue', value: FileNode[]): void;
  (e: 'select', id: string): void;
  (e: 'delete', id: string): void;
}>();

const list = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
});

const handleSelect = (id: string) => {
  emit('select', id);
};

const addItem = (parentId: string | null) => {
    const item = list.value.find((i: FileNode) => i.id === parentId);
    if (item) {
        if (!item.children) item.children = [];
        item.children.push({
            id: `${parentId}-${Date.now()}`,
            name: 'New Section',
            children: []
        });
    }
}

const deleteItem = (id: string) => {
  emit('delete', id);
}
</script>

<template>
  <VueDraggableNext
    v-model="list"
    group="files"
    :animation="200" 
    ghost-class="ghost"
    class="space-y-0.5 min-h-[10px] pb-2"
  >
    <div
      v-for="element in list"
      :key="element.id"
      class="cursor-pointer select-none"
    >
      <div 
        class="group flex justify-between items-center py-1.5 px-2 rounded-md transition-all duration-300 ease-out hover:translate-x-1"
        :class="{ 'bg-stone/50': !isActive(element.id), 'bg-accent/10 text-accent font-medium': isActive(element.id) }"
        @click.stop="handleSelect(element.id)"
      >
        <div class="flex items-center gap-2 overflow-hidden">
             <!-- Drag Handler Icon (optional, but helps imply drag) -->
             <!-- <span class="text-stone/50 text-xs">::</span> -->
            <span class="text-sm font-normal truncate transition-colors"
                  :class="{ 'text-ink': !isActive(element.id), 'text-accent': isActive(element.id) }">
              {{ element.name }}
            </span>
        </div>
        
        <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-all duration-200">
             <!-- Delete Button -->
            <button 
                @click.stop="deleteItem(element.id)"
                class="w-5 h-5 flex items-center justify-center text-ink/20 hover:text-red-500 transition-transform hover:scale-110 active:scale-95"
                title="Delete"
            >
                &times;
            </button>
             <!-- Add Button -->
            <button 
                @click.stop="addItem(element.id)"
                class="w-5 h-5 flex items-center justify-center text-ink/40 hover:text-accent transition-transform hover:scale-110 hover:rotate-90 active:scale-95"
                title="Add Section"
            >
                +
            </button>
        </div>
      </div>

      <!-- Recursive Nesting -->
      <div 
        v-if="element.children"
        class="pl-3 ml-2 mt-0.5 transition-all duration-300 ease-in-out origin-top"
        :class="{ 'border-l border-stone/30': element.children.length > 0 }"
      >
        <FileTree 
          v-model="element.children"
          :active-id="activeId"
          @select="handleSelect" 
          @delete="(id) => emit('delete', id)"
        />
      </div>
    </div>
  </VueDraggableNext>
</template>

<style scoped lang="postcss">
@reference "../style.css";

.ghost {
  opacity: 0.5;
  background-color: #e5e5e0;
}
</style>
