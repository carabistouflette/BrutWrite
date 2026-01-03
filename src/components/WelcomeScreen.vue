<template>
  <Transition name="exit" @after-leave="onAfterLeave">
    <div
      v-if="isVisible"
      class="flex-1 w-full flex flex-col items-center justify-center bg-paper text-ink selection:bg-accent/20 overflow-hidden relative"
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
          <RecentProjectList :recent-projects="recentProjects" @open-project="handleRecent" />

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
          <BrutWriteLogo />

          <!-- Main Actions -->
          <WelcomeActions
            @new-project="handleNewProject"
            @open-project="handleOpenProject"
            @demo-project="handleDemoProject"
          />

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
                {{ formatProjectName(path) }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Minimal Bottom Bar -->
      <div
        class="absolute bottom-8 left-0 right-0 px-12 flex justify-between items-end opacity-20 text-[10px] uppercase tracking-widest font-bold"
      >
        <span>© {{ currentYear }} BrutWrite Studio</span>
        <div class="flex gap-8">
          <span class="hover:text-ink cursor-pointer transition-colors">Documentation</span>
          <span class="hover:text-ink cursor-pointer transition-colors">Release Notes</span>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { open, save } from '@tauri-apps/plugin-dialog';
import { useRecentProjects } from '../composables/domain/project/useRecentProjects';
import { useProjectLoader } from '../composables/domain/project/useProjectLoader';
import { useAppStatus } from '../composables/ui/useAppStatus';
import { formatProjectName } from '../utils/paths';

// Sub-components
import RecentProjectList from './welcome/RecentProjectList.vue';
import WelcomeActions from './welcome/WelcomeActions.vue';
import BrutWriteLogo from './common/BrutWriteLogo.vue';

const { loadProject, createProject, createDemoProject } = useProjectLoader();
const { recentProjects, loadRecentProjects } = useRecentProjects();
const { notifyError } = useAppStatus();

const isVisible = ref(true);
const currentYear = new Date().getFullYear();

const pendingAction = ref<(() => Promise<void>) | null>(null);

const onAfterLeave = async () => {
  if (pendingAction.value) {
    await pendingAction.value();
    pendingAction.value = null;
  }
};

const handleRecent = (path: string) => {
  pendingAction.value = async () => {
    try {
      await loadProject(path);
    } catch (e) {
      notifyError('Failed to load project', e);
    }
  };
  isVisible.value = false;
};

const handleOpenProject = async () => {
  try {
    const selected = await open({
      title: 'Open Project',
      filters: [{ name: 'BrutWrite Project', extensions: ['json'] }],
    });

    if (selected && typeof selected === 'string') {
      pendingAction.value = async () => {
        try {
          await loadProject(selected);
        } catch (e) {
          notifyError('Failed to load project', e);
        }
      };
      isVisible.value = false;
    }
  } catch (e) {
    notifyError('Failed to open project dialog', e);
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
      const name = formatProjectName(selected);
      pendingAction.value = async () => {
        await createProject(selected, name, 'New Author');
      };
      isVisible.value = false;
    }
  } catch (e) {
    notifyError('Failed to create new project', e);
  }
};

const handleDemoProject = async () => {
  try {
    const selected = await save({
      title: 'Create Example Project',
      filters: [{ name: 'BrutWrite Project', extensions: ['json'] }],
      defaultPath: 'TheAlgorithmsOfBetrayal.json',
    });

    if (selected && typeof selected === 'string') {
      pendingAction.value = async () => {
        try {
          await createDemoProject(selected);
        } catch (e) {
          notifyError('Failed to create demo project', e);
        }
      };
      isVisible.value = false;
    }
  } catch (e) {
    notifyError('Failed to create demo project', e);
  }
};

// Ensure list is fresh
loadRecentProjects();
</script>

<style scoped>
.exit-leave-active {
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
