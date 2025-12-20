<script setup lang="ts">
import Sidebar from './components/Sidebar.vue';
// Window controls temporarily removed for minimalist design
</script>

<template>
  <div class="h-screen w-screen bg-paper text-ink flex flex-col font-sans overflow-hidden relative">
    <!-- Invisible Titlebar for Dragging -->
    <div data-tauri-drag-region class="absolute top-0 left-0 w-full h-8 z-50"></div>

    <!-- Main Content (Sidebar + Editor) -->
    <Sidebar>
        <div class="h-full flex flex-col justify-center items-center text-ink opacity-40">
            <h2 class="text-3xl font-light tracking-wide">Start Writing</h2>
            <p class="mt-4 text-xs tracking-widest uppercase">Select a chapter</p>
        </div>
    </Sidebar>
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