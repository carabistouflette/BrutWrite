<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue';
import FileTree from './FileTree.vue';
import { useResizable } from '../composables/useResizable';
import { useProjectData } from '../composables/useProjectData';

// --- Logic Extraction ---
const { width: sidebarWidth, isResizing, startResize } = useResizable({
  initialWidth: 256,
  minWidth: 200,
  maxWidth: 600
});

const { 
  projectData, 
  activeId, 
  selectNode: handleSelect, 
  addChapter: addChapterLogic, 
  addSection: addSectionLogic, 
  deleteNode: handleDelete,
  renameNode: handleRenameLogic
} = useProjectData();

const editingId = ref<string | null>(null);

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

// --- UI / Interaction Props ---
const isAdding = ref(false);

// Note: sidebarScrollRef wasn't in original MainLayout but it's good UX, I'll add the ref too.
const sidebarScrollRef = ref<HTMLElement | null>(null);

// --- Context Menu Logic ---
const showMenu = ref(false);
const menuPos = ref({ x: 0, y: 0 });
const targetNodeId = ref<string | null>(null);

const handleContextMenu = ({ e, id }: { e: MouseEvent, id: string }) => {
  e.preventDefault();
  targetNodeId.value = id;
  menuPos.value = { x: e.clientX, y: e.clientY };
  showMenu.value = true;
};

const closeMenu = () => {
  showMenu.value = false;
};

// Wrappers for UI interactions
const addSection = () => {
  if (targetNodeId.value) {
    addSectionLogic(targetNodeId.value);
  }
  closeMenu();
};

const startRenameTarget = () => {
    if (targetNodeId.value) {
        handleRenameRequest(targetNodeId.value);
    }
    closeMenu();
}

const addChapter = () => {
  addChapterLogic();

  // Trigger feedback animation
  isAdding.value = true;
  setTimeout(() => {
    isAdding.value = false;
  }, 600);
  
  // Scroll to bottom logic if the element exists
  nextTick(() => {
    // We need to attach the ref to the scroll container in template if not already there
    if (sidebarScrollRef.value) {
        sidebarScrollRef.value.scrollTo({
            top: sidebarScrollRef.value.scrollHeight,
            behavior: 'smooth'
        });
    } else {
        // Fallback or try to find it if ref not attached yet
        const el = document.querySelector('aside > div.overflow-y-auto');
        if (el) el.scrollTo({ top: el.scrollHeight, behavior: 'smooth' });
    }
  });
};

const deleteTarget = () => {
  if (targetNodeId.value) {
    handleDelete(targetNodeId.value);
  }
  closeMenu();
};

// Close menu on click elsewhere
onMounted(() => {
  document.addEventListener('click', closeMenu);
});
onUnmounted(() => {
  document.removeEventListener('click', closeMenu);
});
</script>

<template>
  <div class="flex flex-1 w-full text-ink font-sans overflow-hidden relative">
    
    <!-- Sidebar -->
    <aside 
        class="flex flex-col border-r border-stone/60 h-full bg-white/40 backdrop-blur-xl relative z-10 shadow-[4px_0_24px_rgba(0,0,0,0.02)] group"
        :style="{ width: `${sidebarWidth}px` }"
    >
      <div class="p-6 font-serif italic font-bold text-2xl tracking-tight text-ink select-none relative group/logo cursor-default">
        <span class="inline-block transition-transform duration-500 group-hover/logo:scale-105 group-hover/logo:-rotate-1">BrutWrite</span>
        <span class="absolute -bottom-1 left-6 w-8 h-0.5 bg-accent/60 transition-all duration-500 group-hover/logo:w-16 group-hover/logo:bg-accent"></span>
      </div>
      
      <div ref="sidebarScrollRef" class="flex-1 overflow-y-auto px-3 py-2 scroll-smooth">
          <FileTree 
            v-model="projectData" 
            :active-id="activeId"
            :editing-id="editingId"
            @select="handleSelect"
            @delete="handleDelete"
            @context-menu="handleContextMenu"
            @request-rename="handleRenameRequest"
            @submit-rename="handleRenameSubmit"
            @cancel-rename="handleRenameCancel"
          />
        
        <!-- Root Add Button (Clean Glass Design) -->
        <button 
          @click="addChapter"
          class="group relative w-full overflow-hidden rounded-xl border px-4 py-3 mt-8 transition-all duration-500 hover:shadow-sm active:scale-[0.98]"
          :class="isAdding ? 'border-accent/40 bg-accent/5 ring-1 ring-accent/20' : 'border-stone/30 bg-white/5 hover:border-stone/50 hover:bg-white/10'"
        >
          <!-- Subtle Shimmer (Reduced opacity) -->
          <div class="absolute inset-0 -translate-x-[150%] skew-x-12 bg-gradient-to-r from-transparent via-white/10 to-transparent transition-transform duration-700 ease-out group-hover:translate-x-[150%]"></div>
          
          <div class="relative flex items-center justify-center gap-3">
             <!-- Icon: High contrast instead of neon glow -->
            <div class="flex h-5 w-5 items-center justify-center rounded-md bg-stone/20 text-ink/40 transition-all duration-300 group-hover:bg-ink group-hover:text-white group-hover:rotate-90">
              <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                <line x1="12" y1="5" x2="12" y2="19"></line>
                <line x1="5" y1="12" x2="19" y2="12"></line>
              </svg>
            </div>
            
            <span class="text-[11px] font-bold uppercase tracking-[0.2em] text-ink/40 transition-all duration-300 group-hover:text-ink group-hover:tracking-[0.25em]">New Chapter</span>
          </div>
        </button>
      </div>

       <!-- Context Menu -->
       <Teleport to="#app">
         <div 
           v-if="showMenu"
           class="context-menu-glass fixed z-[9999] min-w-[180px] py-2 rounded-xl transition-all duration-200"
           :style="{ 
             top: `${menuPos.y}px`, 
             left: `${menuPos.x}px`,
           }"
         >
            <div 
               @click.stop="addSection" 
               class="menu-item menu-item-default"
            >
              Add Section
            </div>
            <div 
               @click.stop="startRenameTarget" 
               class="menu-item menu-item-default"
            >
              Rename
            </div>
            <div class="h-px bg-ink/5 my-1 mx-2"></div>
            <div 
               @click.stop="deleteTarget" 
               class="menu-item menu-item-danger"
            >
              Delete
            </div>
         </div>
       </Teleport>

      <div class="p-4 border-t border-stone/50">
        <!-- Settings -->
        <button class="w-full py-2 text-xs font-medium text-ink/50 hover:text-accent transition-colors uppercase tracking-wider text-left">
          Settings
        </button>
      </div>

      <!-- Resize Handle -->
      <div 
        @mousedown="startResize"
        class="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-accent/50 transition-colors z-20"
        :class="{ 'bg-accent/50': isResizing }"
      ></div>
    </aside>

    <!-- Main Content Area -->
    <main class="flex-1 flex flex-col h-full bg-white relative">
      <!-- Top Bar / Header -->
      <div class="h-16 px-8 flex justify-between items-center bg-transparent">
        <h1 class="font-normal text-sm text-ink/40 uppercase tracking-widest">Editor</h1>
        <div class="space-x-2">
            <!-- Toolbar -->
        </div>
      </div>

      <!-- Editor Canvas -->
      <div class="flex-1 w-full overflow-hidden relative">
        <slot></slot>
      </div>
    </main>
  </div>
</template>

<style scoped>
/* Brutalist specific overrides if tailwind isn't enough */
</style>
