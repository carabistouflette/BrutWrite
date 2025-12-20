<script setup lang="ts">
import { ref } from 'vue';
import FileTree from './FileTree.vue';
import type { FileNode } from '../types';

// Mock data for initial development
const projectData = ref<FileNode[]>([
  {
    id: '1',
    name: 'Chapter 1: The Beginning',
    children: [
      { id: '1-1', name: 'Section 1.1: Intro' },
      { id: '1-2', name: 'Section 1.2: The Incident' }
    ]
  },
  {
    id: '2',
    name: 'Chapter 2: The Middle',
    children: []
  }
]);

const handleSelect = (id: string) => {
  console.log('Selected item:', id);
  // specific tauri logic will go here
};
</script>

<template>
  <div class="flex flex-1 w-full bg-white text-black font-mono overflow-hidden">
    <!-- Sidebar -->
    <aside class="w-64 flex flex-col border-r-4 border-black h-full">
      <div class="border-b-4 border-black p-4 font-bold text-xl uppercase tracking-tighter">
        BrutWrite
      </div>
      
      <div class="flex-1 overflow-y-auto p-2">
        <FileTree 
          v-model="projectData" 
          @select="handleSelect"
        />
      </div>

      <div class="border-t-4 border-black p-4">
        <!-- Settings or extra controls could go here -->
        <button class="w-full border-4 border-black p-2 hover:bg-black hover:text-white active:translate-x-1 active:translate-y-1 transition-none font-bold uppercase">
          Settings
        </button>
      </div>
    </aside>

    <!-- Main Content Area -->
    <main class="flex-1 flex flex-col h-full bg-gray-50 relative">
      <!-- Top Bar / Header -->
      <div class="border-b-4 border-black p-4 flex justify-between items-center bg-white">
        <h1 class="font-bold text-lg uppercase">Editor</h1>
        <div class="space-x-2">
            <!-- Toolbar placeholder -->
        </div>
      </div>

      <!-- Editor Canvas -->
      <div class="flex-1 p-8 overflow-y-auto">
        <slot></slot>
      </div>
    </main>
  </div>
</template>

<style scoped>
/* Brutalist specific overrides if tailwind isn't enough */
</style>
