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
        class="group hover-trigger relative flex justify-between items-center py-1.5 px-2 rounded-md transition-all duration-300 ease-out hover:translate-x-1 animate-enter"
        :class="{ 'bg-stone/50': !isActive(element.id), 'bg-accent/10 text-accent font-medium': isActive(element.id) }"
        @click.stop="handleSelect(element.id)"
      >
        <!-- Custom Ink Bleed Background on Hover (Pseudo-element approach via absolute div) -->
        <div class="absolute inset-0 bg-stone/50 opacity-0 group-hover:opacity-100 transition-opacity duration-300 rounded-md -z-10 mix-blend-multiply"></div>

        <div class="flex items-center gap-3 overflow-hidden z-10">
             <!-- Dynamic Marker -->
             <span class="w-1 h-1 rounded-full bg-accent transition-all duration-300"
                   :class="{ 'opacity-100 scale-100': isActive(element.id), 'opacity-0 scale-0 group-hover:opacity-40 group-hover:scale-100': !isActive(element.id) }">
             </span>

            <span class="text-sm font-normal truncate transition-all duration-300"
                  :class="{ 'text-ink': !isActive(element.id), 'text-accent translate-x-1': isActive(element.id) }">
              {{ element.name }}
            </span>
        </div>
        
        <div class="flex items-center gap-1 z-10">
             <!-- Delete Button -->
            <button 
                @click.stop="deleteItem(element.id)"
                class="hover-target w-6 h-6 flex items-center justify-center text-ink/30 hover:text-red-500 rounded-full hover:bg-white/50 transition-all duration-300 stagger-1"
                title="Delete"
            >
                <span class="transform scale-75">&times;</span>
            </button>
             <!-- Add Button -->
            <button 
                @click.stop="addItem(element.id)"
                class="hover-target w-6 h-6 flex items-center justify-center text-ink/40 hover:text-accent rounded-full hover:bg-white/50 transition-all duration-300 stagger-2"
                title="Add Section"
            >
                <span class="transform scale-75">+</span>
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
