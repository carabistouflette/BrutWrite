<script setup lang="ts">
import { storeToRefs } from 'pinia';
import AppHeader from './components/layout/AppHeader.vue';
import MainLayout from './components/layout/MainLayout.vue';
import EditorContainer from './components/editor/EditorContainer.vue';
import WelcomeScreen from './components/WelcomeScreen.vue';
import AppNotifications from './components/base/AppNotifications.vue';
import { useProjectStore } from './stores/project';
import GlobalDialogs from './components/base/GlobalDialogs.vue';

const projectStore = useProjectStore();
const { activeId, projectId } = storeToRefs(projectStore);
</script>

<template>
  <div class="bg-paper text-ink flex flex-col font-sans overflow-hidden relative h-full w-full">
    <!-- Cyber-Glass App Header -->
    <AppHeader data-tauri-drag-region />

    <!-- Welcome Screen / Project Loader -->
    <WelcomeScreen v-if="!projectId" />

    <!-- Main Content (Sidebar + Editor) -->
    <MainLayout v-else>
      <EditorContainer
        v-if="activeId && projectId"
        :chapter-id="activeId"
        :project-id="projectId"
      />
      <div v-else class="h-full flex flex-col justify-center items-center text-ink/60 select-none">
        <h2 class="text-5xl font-serif font-bold italic tracking-tight mb-2">Start Writing</h2>
        <p
          class="text-xs font-sans tracking-[0.2em] uppercase text-ink/40 border-t border-accent/30 pt-4 mt-2"
        >
          Select a chapter from the sidebar
        </p>
      </div>
    </MainLayout>

    <AppNotifications />
    <GlobalDialogs />
  </div>
</template>
