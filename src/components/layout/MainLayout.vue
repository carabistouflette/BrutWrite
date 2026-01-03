<script setup lang="ts">
import { ref, defineAsyncComponent } from 'vue';
import GamificationStatus from '../gamification/GamificationStatus.vue';
import SidebarController from './SidebarController.vue';
import { useLayoutController } from '../../composables/ui/useLayoutController';
import { useProjectStore } from '../../stores/project';

const ResearchPanel = defineAsyncComponent(() => import('../research/ResearchPanel.vue'));
const SettingsModal = defineAsyncComponent(() => import('../settings/SettingsModal.vue'));
const CharacterSheet = defineAsyncComponent(() => import('../characters/CharacterSheet.vue'));
const TimelineView = defineAsyncComponent(() => import('../timeline/Timeline.vue'));
const CharacterGraphModal = defineAsyncComponent(
  () => import('../intelligence/CharacterGraphModal.vue')
);

// Extract layout logic
const {
  sidebarWidth,
  isResizingSidebar,
  startResizeSidebar,
  researchWidth,
  isResizingResearch,
  startResizeResearch,
  showSettings,
  showCharacters,
  showCharacterGraph,
  showTimeline,
  showResearch,
} = useLayoutController();

const projectStore = useProjectStore();
const { closeProject } = projectStore;

// --- Local State ---
const isExiting = ref(false);

// --- Event Handlers ---
const handleChangeProject = () => {
  isExiting.value = true;
};

const onExited = (e: TransitionEvent) => {
  // Only trigger if we are explicitly exiting and for the main opacity transition
  // to avoid multiple calls for different transitioning properties.
  if (isExiting.value && e.propertyName === 'opacity') {
    closeProject();
  }
};
</script>

<template>
  <div
    class="animate-enter flex flex-1 w-full h-full text-ink font-sans overflow-hidden relative transition-all duration-500"
    :class="{ 'opacity-0 scale-95': isExiting }"
    @transitionend="onExited"
  >
    <!-- Sidebar -->
    <aside
      class="flex flex-col border-r border-stone/60 h-full cyber-glass relative z-10 shadow-[4px_0_24px_rgba(0,0,0,0.02)] group"
      :style="{ width: `${sidebarWidth}px` }"
    >
      <SidebarController
        @open-settings="showSettings = true"
        @open-characters="showCharacters = true"
        @open-character-graph="showCharacterGraph = true"
        @open-timeline="showTimeline = !showTimeline"
        @open-research="showResearch = !showResearch"
        @change-project="handleChangeProject"
      />

      <!-- Resize Handle -->
      <div
        class="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-accent/50 transition-colors z-20"
        :class="{ 'bg-accent/50': isResizingSidebar }"
        @mousedown="startResizeSidebar"
      ></div>
    </aside>

    <!-- Main Content Area -->
    <main class="flex-1 flex flex-col h-full bg-transparent relative">
      <!-- Top Bar / Header -->
      <div class="h-16 px-8 flex justify-between items-center bg-transparent">
        <h1 class="font-normal text-sm text-ink/40 uppercase tracking-widest">Editor</h1>
        <div class="space-x-2 flex items-center">
          <GamificationStatus />
          <!-- Toolbar -->
        </div>
      </div>

      <!-- Editor Canvas or Timeline View -->
      <div class="flex-1 w-full overflow-hidden relative">
        <TimelineView v-if="showTimeline" />
        <slot v-else></slot>
      </div>
    </main>

    <!-- Research Sidebar -->
    <Transition name="slide-right">
      <aside
        v-if="showResearch"
        class="border-l border-stone/50 h-full cyber-glass relative z-10 shadow-[-4px_0_24px_rgba(0,0,0,0.02)]"
        :style="{ width: `${researchWidth}px` }"
      >
        <!-- Resize Handle -->
        <div
          class="absolute top-0 left-0 w-1 h-full cursor-col-resize hover:bg-accent/50 transition-colors z-20"
          :class="{ 'bg-accent/50': isResizingResearch }"
          @mousedown="startResizeResearch"
        ></div>

        <ResearchPanel @close="showResearch = false" />
      </aside>
    </Transition>

    <!-- Global Modals (Moved to root to prevent clipping/z-index issues) -->
    <SettingsModal :show="showSettings" @close="showSettings = false" />
    <CharacterSheet :show="showCharacters" @close="showCharacters = false" />
    <CharacterGraphModal :show="showCharacterGraph" @close="showCharacterGraph = false" />
  </div>
</template>

<style scoped>
/* Brutalist specific overrides if tailwind isn't enough */

/* Research Panel Animation */
.slide-right-enter-active,
.slide-right-leave-active {
  transition:
    transform 0.35s ease-out,
    opacity 0.35s ease;
  will-change: transform, opacity;
}

.slide-right-enter-from,
.slide-right-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>
