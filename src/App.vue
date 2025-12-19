<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();

const minimize = () => appWindow.minimize();
const toggleMaximize = () => appWindow.toggleMaximize();
const close = () => appWindow.close();
</script>

<template>
  <div class="h-screen w-screen bg-black text-white flex flex-col font-mono border-4 border-white overflow-hidden">
    <!-- Custom Brutalist Titlebar -->
    <div data-tauri-drag-region class="h-10 flex items-center justify-between px-4 border-b-4 border-white select-none">
      <div class="font-bold tracking-tighter uppercase italic">BrutWrite // v0.1.0</div>
      <div class="flex h-full">
        <button @click="minimize" class="h-full px-4 hover:bg-white hover:text-black border-l-4 border-white transition-colors">_</button>
        <button @click="toggleMaximize" class="h-full px-4 hover:bg-white hover:text-black border-l-4 border-white transition-colors">[]</button>
        <button @click="close" class="h-full px-4 hover:bg-red-600 hover:text-white border-l-4 border-white transition-colors">X</button>
      </div>
    </div>

    <!-- Main Content Area -->
    <div class="flex-1 p-8">
      <div class="border-4 border-white p-6 h-full flex flex-col justify-center items-center text-center">
        <h1 class="text-6xl font-black mb-4 uppercase leading-none">Initialization Complete</h1>
        <p class="text-xl mb-8">MILESTONE 1: FOUNDATION // ISSUE 1.1 SETTLED</p>
        <div class="grid grid-cols-2 gap-4 w-full max-w-2xl">
          <div class="border-2 border-white p-4">TAURI 2.0 // STABLE</div>
          <div class="border-2 border-white p-4">VUE 3 // REACTIVE</div>
          <div class="border-2 border-white p-4">TYPESCRIPT // STRICT</div>
          <div class="border-2 border-white p-4">TAILWIND // BRUTAL</div>
        </div>
      </div>
    </div>

    <!-- Minimal Status Bar Placeholder -->
    <div class="h-8 border-t-4 border-white px-4 flex items-center text-sm font-bold justify-between">
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