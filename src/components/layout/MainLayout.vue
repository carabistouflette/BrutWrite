<script setup lang="ts">
import { ref, nextTick } from 'vue';
import FileTree from '../FileTree.vue';
import GamificationStatus from '../GamificationStatus.vue';
import ContextMenu from '../base/ContextMenu.vue';
import AppLogo from '../common/AppLogo.vue';
import SidebarFooter from './SidebarFooter.vue';
import AddChapterButton from '../AddChapterButton.vue';
import { useResizable } from '../../composables/useResizable';
import { useProjectData } from '../../composables/useProjectData';
import { useContextMenu } from '../../composables/useContextMenu';

import { defineAsyncComponent } from 'vue';

const SettingsModal = defineAsyncComponent(() => import('../SettingsModal.vue'));
const CharacterSheet = defineAsyncComponent(() => import('../characters/CharacterSheet.vue'));
const TimelineView = defineAsyncComponent(() => import('../timeline/Timeline.vue'));

// --- Composables ---
const { width: sidebarWidth, isResizing, startResize } = useResizable({
  initialWidth: 256,
  minWidth: 200,
  maxWidth: 600
});

const { 
  projectData, 
  activeId, 
  addChapter: addChapterLogic, 
  addSection: addSectionLogic, 
  deleteNode: handleDelete,
  renameNode: handleRenameLogic,
  updateStructure,
  closeProject
} = useProjectData();

const { showMenu, menuPos, targetNodeId, openMenu, closeMenu } = useContextMenu();

// --- Local State ---
const editingId = ref<string | null>(null);
const isAdding = ref(false);
const showSettings = ref(false);
const showCharacters = ref(false);
const showTimeline = ref(false);
const sidebarScrollRef = ref<HTMLElement | null>(null);

// --- Event Handlers ---

// Rename Logic
const handleRenameRequest = (id: string) => {
  editingId.value = id;
};

const handleRenameSubmit = ({ id, name }: { id: string, name: string }) => {
  handleRenameLogic(id, name);
  editingId.value = null;
};

const handleRenameCancel = () => {
    editingId.value = null;
};

// Context Menu Interface
const startRenameTarget = () => {
    if (targetNodeId.value) {
        handleRenameRequest(targetNodeId.value);
    }
    closeMenu();
}

const deleteTarget = () => {
  if (targetNodeId.value) {
    handleDelete(targetNodeId.value);
  }
  closeMenu();
};

const addSection = () => {
  if (targetNodeId.value) {
    addSectionLogic(targetNodeId.value);
  }
  closeMenu();
};

const handleContextMenu = ({ e, id }: { e: MouseEvent, id: string }) => {
    openMenu(e, id);
};

// Add Chapter Animation & Scroll
const addChapter = () => {
  addChapterLogic();

  isAdding.value = true;
  setTimeout(() => isAdding.value = false, 600);
  
  nextTick(() => {
    if (sidebarScrollRef.value) {
        sidebarScrollRef.value.scrollTo({
            top: sidebarScrollRef.value.scrollHeight,
            behavior: 'smooth'
        });
    }
  });
};

const isExiting = ref(false);

const handleChangeProject = async () => {
    isExiting.value = true;
    await new Promise(resolve => setTimeout(resolve, 500));
    closeProject();
};
</script>

<template>
  <div class="animate-enter flex flex-1 w-full h-full text-ink font-sans overflow-hidden relative transition-all duration-500" :class="{'opacity-0 scale-95': isExiting}">
    
    <!-- Sidebar -->
    <aside 
        class="flex flex-col border-r border-stone/60 h-full cyber-glass relative z-10 shadow-[4px_0_24px_rgba(0,0,0,0.02)] group"
        :style="{ width: `${sidebarWidth}px` }"
    >
      <AppLogo />
      
      <div ref="sidebarScrollRef" class="flex-1 overflow-y-auto px-3 py-2 scroll-smooth">
          <FileTree 
            :model-value="projectData"
            @update:model-value="updateStructure" 
            :active-id="activeId"
            :editing-id="editingId"
            @context-menu="handleContextMenu"
            @request-rename="handleRenameRequest"
            @submit-rename="handleRenameSubmit"
            @cancel-rename="handleRenameCancel"
          />
        
        <AddChapterButton :is-adding="isAdding" @click="addChapter" />
      </div>

      <!-- Context Menu -->
      <ContextMenu 
        :show="showMenu" 
        :x="menuPos.x" 
        :y="menuPos.y" 
        @close="closeMenu"
      >
        <div @click.stop="addSection" class="menu-item menu-item-default">
          Add Section
        </div>
        <div @click.stop="startRenameTarget" class="menu-item menu-item-default">
          Rename
        </div>
        <div class="h-px bg-ink/5 my-1 mx-2"></div>
        <div @click.stop="deleteTarget" class="menu-item menu-item-danger">
          Delete
        </div>
      </ContextMenu>

      <SidebarFooter 
        class="mt-auto"
        @open-settings="showSettings = true" 
        @open-characters="showCharacters = true"
        @open-timeline="showTimeline = !showTimeline"
        @change-project="handleChangeProject"
      />
      
      <SettingsModal :show="showSettings" @close="showSettings = false" />
      <CharacterSheet :show="showCharacters" @close="showCharacters = false" />

      <!-- Resize Handle -->
      <div 
        @mousedown="startResize"
        class="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-accent/50 transition-colors z-20"
        :class="{ 'bg-accent/50': isResizing }"
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
  </div>
</template>

<style scoped>
/* Brutalist specific overrides if tailwind isn't enough */
</style>
