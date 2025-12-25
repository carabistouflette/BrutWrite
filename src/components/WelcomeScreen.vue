<template>
  <div
    class="flex-1 w-full flex flex-col items-center justify-center bg-paper text-ink selection:bg-accent/20 overflow-hidden relative"
    :class="{ 'exit-active': isExiting }"
  >
    <!-- Background Decorative Elements -->
    <div class="absolute inset-0 z-0 pointer-events-none opacity-30">
      <div
        class="absolute top-[10%] left-[5%] w-[40vw] h-[40vw] rounded-full bg-accent/5 blur-[120px] animate-pulse"
      ></div>
      <div
        class="absolute bottom-[10%] right-[5%] w-[30vw] h-[30vw] rounded-full bg-accent/10 blur-[100px] animate-pulse stagger-1"
      ></div>
    </div>

    <div
      class="relative z-10 w-full max-w-6xl px-12 grid grid-cols-1 lg:grid-cols-2 gap-24 items-center"
    >
      <!-- Left: Recent Projects & Inspiration -->
      <div class="hidden lg:flex flex-col space-y-8 animate-enter">
        <div class="space-y-4">
          <h2 class="text-xs uppercase tracking-[0.3em] opacity-40 font-medium">
            Recent Manuscripts
          </h2>
          <div v-if="recentProjects.length > 0" class="space-y-3">
            <button
              v-for="path in recentProjects"
              :key="path"
              class="w-full text-left p-4 rounded-xl border border-ink/5 hover:border-accent/30 hover:bg-white/50 transition-all group flex items-center justify-between cyber-glass"
              @click="handleRecent(path)"
            >
              <div class="flex flex-col">
                <span class="text-sm font-medium">{{ getFileName(path) }}</span>
                <span class="text-[10px] opacity-30 truncate max-w-[200px]">{{ path }}</span>
              </div>
              <svg
                class="w-4 h-4 opacity-0 group-hover:opacity-100 -translate-x-2 group-hover:translate-x-0 transition-all text-accent"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M14 5l7 7m0 0l-7 7m7-7H3"
                />
              </svg>
            </button>
          </div>
          <div
            v-else
            class="p-8 border border-dashed border-ink/10 rounded-2xl flex flex-col items-center justify-center text-center space-y-2"
          >
            <p class="text-sm opacity-30 italic">No recent projects found</p>
          </div>
        </div>

        <div class="pt-12 border-t border-ink/5">
          <p class="text-xs italic opacity-30 leading-relaxed max-w-sm">
            "The first draft is just you telling yourself the story."
            <br />— Terry Pratchett
          </p>
        </div>
      </div>

      <!-- Right: Branding & Core Actions -->
      <div
        class="flex flex-col items-center lg:items-end lg:text-right space-y-12 animate-enter stagger-1"
      >
        <!-- Logo Container -->
        <div class="space-y-4">
          <div class="relative inline-block mb-8">
            <h1 class="text-8xl font-serif font-bold italic tracking-tighter leading-none">
              Brut<span class="relative"
                >Write<span
                  class="absolute bottom-[2px] left-0 w-full h-[2px] bg-accent shadow-[0_0_12px_rgba(255,95,31,0.4)]"
                ></span
              ></span>
            </h1>
          </div>
          <p class="text-xs uppercase tracking-[0.4em] opacity-60 font-light">
            Distraction Free Writing Environment
          </p>
        </div>

        <!-- Main Actions -->
        <div class="w-full max-w-sm space-y-4">
          <button
            class="w-full group relative overflow-hidden bg-ink text-paper py-5 px-8 rounded-2xl font-semibold transition-all hover:shadow-[0_20px_40px_rgba(0,0,0,0.15)] hover:-translate-y-1 flex items-center justify-center gap-4"
            @click="handleNewProject"
          >
            <div class="relative z-10 flex items-center gap-3">
              <svg
                class="w-5 h-5 opacity-60 group-hover:rotate-90 transition-transform duration-500"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 4v16m8-8H4"
                />
              </svg>
              <span>Create New Project</span>
            </div>
            <div
              class="absolute inset-0 bg-accent translate-y-full group-hover:translate-y-0 transition-transform duration-500 opacity-10"
            ></div>
          </button>

          <button
            class="w-full group relative overflow-hidden bg-white/40 border border-ink/10 hover:border-accent/40 text-ink py-5 px-8 rounded-2xl font-medium transition-all hover:bg-white/80 cyber-glass flex items-center justify-center gap-4"
            @click="handleOpenProject"
          >
            <svg
              class="w-5 h-5 opacity-40 group-hover:text-accent group-hover:opacity-100 transition-all"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z"
              />
            </svg>
            <span>Open Existing Project</span>
          </button>
        </div>

        <!-- Footer Meta (Mobile only or additional) -->
        <div class="lg:hidden w-full space-y-4 pt-8">
          <h2 class="text-[10px] uppercase tracking-[0.2em] opacity-30 text-center font-bold">
            Recent Projects
          </h2>
          <div class="flex gap-2 overflow-x-auto pb-4 px-2">
            <button
              v-for="path in recentProjects"
              :key="path"
              class="whitespace-nowrap px-4 py-2 rounded-full border border-ink/10 text-xs hover:bg-ink hover:text-paper transition-all"
              @click="handleRecent(path)"
            >
              {{ getFileName(path) }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Minimal Bottom Bar -->
    <div
      class="absolute bottom-8 left-0 right-0 px-12 flex justify-between items-end opacity-20 text-[10px] uppercase tracking-widest font-bold"
    >
      <span>© 2025 BrutWrite Studio</span>
      <div class="flex gap-8">
        <span class="hover:text-ink cursor-pointer transition-colors">Documentation</span>
        <span class="hover:text-ink cursor-pointer transition-colors">Release Notes</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { open, save } from '@tauri-apps/plugin-dialog';
import { useProjectIO } from '../composables/domain/useProjectIO';
import { useRecentProjects } from '../composables/domain/useRecentProjects';

const { loadProject, createProject } = useProjectIO();
const { recentProjects, loadRecentProjects } = useRecentProjects();
const isExiting = ref(false);

const getFileName = (path: string) => {
  return path.split(/[\\/]/).pop()?.replace('.json', '') || 'Untitled';
};

const triggerExit = async () => {
  isExiting.value = true;
  await new Promise((resolve) => setTimeout(resolve, 600)); // Wait for animation
};

const handleRecent = async (path: string) => {
  await triggerExit();
  await loadProject(path);
};

const handleOpenProject = async () => {
  try {
    const selected = await open({
      title: 'Open Project',
      filters: [{ name: 'BrutWrite Project', extensions: ['json'] }],
    });

    if (selected && typeof selected === 'string') {
      await triggerExit();
      await loadProject(selected);
      // useProjectIO updates recent projects automatically
    }
  } catch (e) {
    console.error('Failed to open project dialog:', e);
  }
};

const handleNewProject = async () => {
  try {
    const selected = await save({
      title: 'Create New Project',
      filters: [{ name: 'BrutWrite Project', extensions: ['json'] }],
      defaultPath: 'MyStory.json',
    });

    if (selected && typeof selected === 'string') {
      const name = selected.split(/[\\/]/).pop()?.replace('.json', '') || 'Untitled Project';
      await triggerExit();
      await createProject(selected, name, 'Unknown Author');
      // useProjectIO updates recent projects automatically
    }
  } catch (e) {
    console.error('Failed to create project dialog:', e);
  }
};

// Ensure list is fresh
loadRecentProjects();
</script>

<style scoped>
.cyber-glass {
  backdrop-filter: blur(12px) saturate(180%);
  -webkit-backdrop-filter: blur(12px) saturate(180%);
}

.exit-active {
  animation: exit-scale 0.5s cubic-bezier(0.4, 0, 0.2, 1) forwards;
}

@keyframes exit-scale {
  0% {
    opacity: 1;
    transform: scale(1);
  }
  100% {
    opacity: 0;
    transform: scale(0.95);
  }
}
</style>
