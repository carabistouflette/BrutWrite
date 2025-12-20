<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window';
import Sidebar from './components/Sidebar.vue';

const appWindow = getCurrentWindow();

const minimize = () => appWindow.minimize();
const toggleMaximize = () => appWindow.toggleMaximize();
const close = () => appWindow.close();
</script>

<template>
  <div class="h-screen w-screen bg-black text-white flex flex-col font-mono border-4 border-white overflow-hidden">
    <!-- Custom Brutalist Titlebar -->
    <div data-tauri-drag-region class="h-10 flex items-center justify-between px-4 border-b-4 border-white select-none bg-black text-white">
      <div class="font-bold tracking-tighter uppercase italic">BrutWrite // v0.1.0</div>
      <div class="flex h-full">
        <button @click="minimize" class="h-full px-4 hover:bg-white hover:text-black border-l-4 border-white transition-colors">_</button>
        <button @click="toggleMaximize" class="h-full px-4 hover:bg-white hover:text-black border-l-4 border-white transition-colors">[]</button>
        <button @click="close" class="h-full px-4 hover:bg-red-600 hover:text-white border-l-4 border-white transition-colors">X</button>
      </div>
    </div>

    <!-- Main Content (Sidebar + Editor) -->
    <Sidebar>
        <div class="h-full flex flex-col justify-center items-center text-black">
            <h2 class="text-4xl font-black uppercase">Start Writing</h2>
            <p class="mt-4 text-sm font-bold">SELECT A CHAPTER FROM THE SIDEBAR</p>
        </div>
    </Sidebar>

    <!-- Minimal Status Bar -->
    <div class="h-8 border-t-4 border-white bg-black text-white px-4 flex items-center text-sm font-bold justify-between z-10">
      <div>READY_FOR_MANUSCRIPT</div>
      <div>0 WORDS // 0% GOAL</div>
    </div>
  </div>
</template>

<style>
/* Reset and custom scrollbar for brutalist feel */
::-webkit-scrollbar {
  width: 12px;
}
::-webkit-scrollbar-track {
  background: black;
  border-left: 4px solid white;
}
::-webkit-scrollbar-thumb {
  background: white;
}
::-webkit-scrollbar-thumb:hover {
  background: #ccc;
}

body {
  margin: 0;
  overflow: hidden;
  background: transparent;
}
</style>