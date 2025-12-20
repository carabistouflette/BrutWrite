<script setup lang="ts">
import { VueDraggableNext } from 'vue-draggable-next';
import { computed } from 'vue';
import type { FileNode } from '../types';

defineOptions({
  name: 'FileTree'
});

const props = defineProps<{
  modelValue: FileNode[]
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: FileNode[]): void;
  (e: 'select', id: string): void;
}>();

const list = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
});

const handleSelect = (id: string) => {
  emit('select', id);
};

const addItem = (parentId: string | null) => {
    // Determine where to add. If parentId matches an item in our list, we add to its children.
    // However, since this is recursive, 'list' here IS the children array of the parent context (or root).
    // The button is rendered ON the item, so we want to add to THAT item's children.
    
    // We can't easily modify the prop 'element' children directly if it's not reactive deep down or if we want to emit.
    // But v-model implies we might be able to.
    // Actually, the button is inside the v-for.
    // element.children.push(...)
    
    // Let's iterate to find the element (or if we trust the reference 'element' in the template):
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

</script>

<template>
  <VueDraggableNext
    v-model="list"
    group="files"
    :animation="200" 
    ghost-class="ghost"
    class="space-y-0.5"
  >
    <div
      v-for="element in list"
      :key="element.id"
      class="cursor-pointer select-none"
    >
      <div 
        class="group flex justify-between items-center py-1.5 px-2 rounded-md hover:bg-stone/50 transition-colors"
        @click.stop="handleSelect(element.id)"
      >
        <span class="text-sm text-ink group-hover:text-ink font-normal truncate">{{ element.name }}</span>
        
         <!-- Add Button (only visible on hover for cleanliness) -->
        <button 
            @click.stop="addItem(element.id)"
            class="opacity-0 group-hover:opacity-100 w-5 h-5 flex items-center justify-center text-ink/40 hover:text-accent transition-all"
            title="Add Section"
        >
            +
        </button>
      </div>

      <!-- Recursive Nesting -->
      <div v-if="element.children" class="pl-3 ml-2 border-l border-stone/30 mt-0.5">
        <FileTree 
          v-model="element.children"
          @select="handleSelect" 
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
