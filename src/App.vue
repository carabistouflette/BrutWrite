<script setup lang="ts">
import AppHeader from './components/AppHeader.vue';
import MainLayout from './components/MainLayout.vue';
import EditorMain from './components/EditorMain.vue';
import { useProjectData } from './composables/useProjectData';

const { activeId } = useProjectData();
</script>

<template>
  <div class="h-screen w-screen bg-paper text-ink flex flex-col font-sans overflow-hidden relative">
    <!-- Cyber-Glass App Header -->
    <AppHeader data-tauri-drag-region />

    <!-- Main Content (Sidebar + Editor) -->
    <MainLayout>
        <EditorMain v-if="activeId" />
        <div v-else class="h-full flex flex-col justify-center items-center text-ink/60 select-none">
            <h2 class="text-5xl font-serif font-bold italic tracking-tight mb-2">Start Writing</h2>
            <p class="text-xs font-sans tracking-[0.2em] uppercase text-ink/40 border-t border-accent/30 pt-4 mt-2">Select a chapter from the sidebar</p>
        </div>
    </MainLayout>
  </div>
</template>