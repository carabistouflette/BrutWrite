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
    :animation="0" 
    ghost-class="ghost"
    class="space-y-2"
  >
    <div
      v-for="element in list"
      :key="element.id"
      class="cursor-pointer"
    >
      <div 
        class="border-4 border-black p-2 bg-white hover:bg-gray-100 flex justify-between items-center transition-none"
        @click.stop="handleSelect(element.id)"
      >
        <span class="font-bold truncate">{{ element.name }}</span>
        
         <!-- Add Button (only for chapters/folders usually, but generic here for now) -->
        <button 
            @click.stop="addItem(element.id)"
            class="w-6 h-6 flex items-center justify-center border-2 border-black hover:bg-black hover:text-white transition-none"
        >
            +
        </button>
      </div>

      <!-- Recursive Nesting -->
      <div v-if="element.children" class="pl-4 border-l-4 border-black ml-4 mt-2">
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
  @apply opacity-50 bg-gray-300 border-dashed;
}
</style>
