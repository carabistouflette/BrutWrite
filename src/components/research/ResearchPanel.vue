<script setup lang="ts">
import { onMounted } from 'vue';
import { useResearchStore } from '../../stores/research';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import ResearchSidebar from './ResearchSidebar.vue';
import ResearchContent from './ResearchContent.vue';

const store = useResearchStore();

onMounted(() => {
  store.fetchArtifacts();
});

const openAddDialog = async () => {
  try {
    const selected = await openDialog({
      multiple: true,
      filters: [
        {
          name: 'Research',
          extensions: ['pdf', 'png', 'jpg', 'jpeg', 'webp', 'md', 'txt'],
        },
      ],
    });

    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      await store.addFiles(paths as string[]);
    }
  } catch (e) {
    console.error('Dialog error', e);
  }
};
</script>

<template>
  <div class="h-full w-full bg-zinc-950 overflow-hidden relative">
    <Transition name="fade-slide" mode="out-in">
      <!-- View 1: List / Sidebar -->
      <ResearchSidebar v-if="!store.activeArtifact" class="w-full h-full" @add="openAddDialog" />

      <!-- View 2: Content Viewer -->
      <ResearchContent v-else class="w-full h-full" @add="openAddDialog" />
    </Transition>
  </div>
</template>
