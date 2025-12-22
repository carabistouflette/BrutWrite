<template>
  <aside
    class="flex flex-col h-full w-full bg-zinc-950 text-zinc-300 overflow-hidden font-sans border-r border-white/5 relative"
    @dragover.prevent="isDragging = true"
    @dragleave.prevent="isDragging = false"
    @drop.prevent="handleDrop"
  >
    <!-- Drop Overlay -->
    <div
      v-if="isDragging"
      class="absolute inset-0 z-50 bg-accent/20 backdrop-blur-sm border-2 border-accent border-dashed m-2 rounded-xl flex items-center justify-center pointer-events-none"
    >
      <div class="bg-zinc-950 p-4 rounded-xl shadow-xl flex flex-col items-center gap-2">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="32"
          height="32"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="text-accent"
        >
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="17 8 12 3 7 8" />
          <line x1="12" y1="3" x2="12" y2="15" />
        </svg>
        <span class="text-xs font-bold uppercase tracking-widest text-zinc-100"
          >Drop to Import</span
        >
      </div>
    </div>

    <!-- Header Area -->
    <div class="pt-8 pb-6 px-8 flex flex-col gap-1 shrink-0">
      <div class="flex items-center justify-between">
        <h2 class="font-serif text-2xl text-zinc-100 tracking-tight font-medium">Research</h2>
        <span
          class="text-[10px] tracking-widest uppercase font-bold text-zinc-600 border border-zinc-800 px-2 py-0.5 rounded-full"
        >
          {{ store.artifacts.length }}
        </span>
      </div>
      <p class="text-[11px] text-zinc-500 uppercase tracking-widest font-semibold mt-1">
        Vault Content
      </p>
    </div>

    <!-- Search Area -->
    <div class="px-6 mb-4 shrink-0">
      <div class="relative group">
        <div class="absolute inset-y-0 left-3 flex items-center pointer-events-none">
          <svg
            class="w-3.5 h-3.5 text-zinc-600 transition-colors duration-300 group-focus-within:text-zinc-300"
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <circle cx="11" cy="11" r="8"></circle>
            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
          </svg>
        </div>
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Filter sources or #tags..."
          class="w-full bg-zinc-900/50 hover:bg-zinc-900 transition-all duration-300 text-xs font-medium text-zinc-200 py-3 pl-10 pr-4 rounded-lg border border-transparent focus:border-zinc-700 focus:bg-zinc-900 focus:outline-none placeholder-zinc-700"
        />
      </div>
    </div>

    <!-- List Area -->
    <div class="flex-1 overflow-y-auto custom-scrollbar px-4 pb-4 space-y-1">
      <div
        v-for="item in filteredArtifacts"
        :key="item.id"
        class="group relative flex items-center gap-3 px-3 py-3 rounded-md cursor-pointer transition-all duration-300"
        :class="[
          store.activeArtifact?.id === item.id
            ? 'bg-zinc-900 text-zinc-100 shadow-sm'
            : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/40',
        ]"
        @click="store.setActiveArtifact(item)"
        @contextmenu.prevent="handleContextMenu($event, item)"
      >
        <!-- Active Indicator -->
        <div
          v-if="store.activeArtifact?.id === item.id"
          class="absolute left-0 top-1/2 -translate-y-1/2 h-8 w-1 bg-zinc-100 rounded-r-sm shadow-[0_0_10px_rgba(255,255,255,0.3)]"
        ></div>

        <!-- Start Icon -->
        <div
          class="ml-1 shrink-0 p-2 rounded-md transition-colors duration-300"
          :class="
            store.activeArtifact?.id === item.id
              ? 'bg-zinc-800 text-zinc-100'
              : 'bg-zinc-900/50 text-zinc-600 group-hover:bg-zinc-800 group-hover:text-zinc-400'
          "
        >
          <!-- PDF Icon -->
          <svg
            v-if="item.file_type === 'pdf'"
            xmlns="http://www.w3.org/2000/svg"
            class="w-4 h-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" />
            <polyline points="14 2 14 8 20 8" />
          </svg>
          <!-- Image Icon -->
          <svg
            v-else-if="['png', 'jpg', 'jpeg', 'webp'].includes(item.file_type)"
            xmlns="http://www.w3.org/2000/svg"
            class="w-4 h-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
            <circle cx="8.5" cy="8.5" r="1.5" />
            <polyline points="21 15 16 10 5 21" />
          </svg>
          <!-- Text/Default Icon -->
          <svg
            v-else
            xmlns="http://www.w3.org/2000/svg"
            class="w-4 h-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
            <line x1="16" y1="13" x2="8" y2="13" />
            <line x1="16" y1="17" x2="8" y2="17" />
            <polyline points="10 9 9 9 8 9" />
          </svg>
        </div>

        <!-- Label & Tags -->
        <div class="min-w-0 flex-1 flex flex-col justify-center gap-0.5">
          <span class="text-[13px] font-medium truncate leading-tight transition-colors">
            {{ item.name }}
          </span>
          <div class="flex items-center gap-1.5 flex-wrap">
            <span
              class="text-[9px] font-mono uppercase tracking-wider"
              :class="
                store.activeArtifact?.id === item.id
                  ? 'text-zinc-500'
                  : 'text-zinc-700 group-hover:text-zinc-600'
              "
            >
              {{ item.file_type }}
            </span>
            <!-- Tags -->
            <span
              v-for="tag in item.tags.slice(0, 3)"
              :key="tag"
              class="text-[9px] font-bold px-1.5 py-px rounded-sm bg-zinc-800 text-zinc-400"
            >
              #{{ tag }}
            </span>
            <span v-if="item.tags.length > 3" class="text-[9px] text-zinc-600"
              >+{{ item.tags.length - 3 }}</span
            >
          </div>
        </div>
      </div>
    </div>

    <!-- Bottom Action -->
    <div class="p-6 shrink-0 border-t border-white/5 bg-zinc-950/50 backdrop-blur-sm flex gap-2">
      <button
        class="group relative flex-1 overflow-hidden rounded-xl bg-zinc-100 text-zinc-950 py-3.5 px-2 text-xs font-bold uppercase tracking-widest transition-all duration-300 hover:shadow-[0_0_20px_rgba(255,255,255,0.1)] hover:-translate-y-0.5"
        @click="$emit('add')"
      >
        <span class="relative z-10 flex items-center justify-center gap-2">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="w-3.5 h-3.5"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
          Add
        </span>
        <div
          class="absolute inset-0 z-0 bg-white opacity-0 transition-opacity duration-300 group-hover:opacity-100 mix-blend-overlay"
        ></div>
      </button>

      <button
        class="group relative flex-1 overflow-hidden rounded-xl bg-zinc-800 text-zinc-300 border border-zinc-700 py-3.5 px-2 text-xs font-bold uppercase tracking-widest transition-all duration-300 hover:bg-zinc-700 hover:text-zinc-100 hover:-translate-y-0.5"
        @click="handleNewNote"
      >
        <span class="relative z-10 flex items-center justify-center gap-2">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="w-3.5 h-3.5"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
          </svg>
          Note
        </span>
      </button>
    </div>

    <!-- Context Menu -->
    <ContextMenu :show="showMenu" :x="menuPos.x" :y="menuPos.y" @close="closeMenu">
      <div
        class="px-3 py-2 text-[10px] font-black uppercase text-zinc-500 border-b border-zinc-800/50 mb-1"
      >
        ACTIONS
      </div>
      <div class="menu-item menu-item-default" @click="handleRename">Rename (Not Impl)</div>
      <div class="menu-item menu-item-default" @click="handleAddTag">Add Tag</div>
    </ContextMenu>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useResearchStore, type ResearchArtifact } from '../../stores/research';
import { listen } from '@tauri-apps/api/event';
import ContextMenu from '../base/ContextMenu.vue';

const store = useResearchStore();
const searchQuery = ref('');
const isDragging = ref(false);

defineEmits(['add']);

// --- Drag and Drop ---
const handleDrop = () => {
  isDragging.value = false;
  // Fallback if Tauri doesn't intercept
  // In pure webview, we might rely on the listen('tauri://drag-drop')
};

let unlisten: (() => void) | undefined;

onMounted(async () => {
  // Listen for file drops globally and check if we are the target (UI check is hard, so we just accept global drops for now as "Add to Vault")
  // Or we rely on the fact that if the user drops here, it triggers the event.
  // Actually, tauri://drag-drop payload contains paths.
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  unlisten = await listen('tauri://drag-drop', (event: any) => {
    const paths = event.payload.paths;
    if (paths && paths.length > 0) {
      store.addFiles(paths);
    }
    isDragging.value = false;
  });

  // We also need drag-enter to show UI. 'tauri://drag-enter' ?
  // Tauri v2 might not have 'tauri://drag-enter' exposed easily without plugins.
  // So we use the HTML5 events on the container for visual feedback (which works if the webview allows DnD)
});

onUnmounted(() => {
  if (unlisten) {
    unlisten();
  }
});

// --- Filtering ---
const filteredArtifacts = computed(() => {
  if (!searchQuery.value) return store.artifacts;
  const q = searchQuery.value.toLowerCase();

  return store.artifacts.filter((a) => {
    const matchesName = a.name.toLowerCase().includes(q);
    const matchesTag = a.tags && a.tags.some((t) => t.toLowerCase().includes(q.replace('#', '')));
    return matchesName || matchesTag;
  });
});

// --- Context Menu ---
const showMenu = ref(false);
const menuPos = ref({ x: 0, y: 0 });
const targetArtifact = ref<ResearchArtifact | null>(null);

const handleContextMenu = (e: MouseEvent, item: ResearchArtifact) => {
  e.preventDefault();
  targetArtifact.value = item;
  menuPos.value = { x: e.clientX, y: e.clientY };
  showMenu.value = true;
};

const closeMenu = () => {
  showMenu.value = false;
  targetArtifact.value = null;
};

const handleAddTag = () => {
  if (!targetArtifact.value) return;
  const tag = window.prompt('Enter tag:');
  if (tag && tag.trim()) {
    const item = targetArtifact.value;
    if (!item.tags.includes(tag.trim())) {
      item.tags.push(tag.trim());
      store.updateArtifact(JSON.parse(JSON.stringify(item))); // clone to be safe
    }
  }
  closeMenu();
};

const handleNewNote = async () => {
  const name = window.prompt('Note Name:');
  if (name && name.trim()) {
    await store.createNote(name.trim());
  }
};

const handleRename = () => {
  // TODO: Implement Rename
  closeMenu();
};
</script>

<style scoped>
/* Ultra-minimal scrollbar */
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 99px;
}
.custom-scrollbar:hover::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
}
</style>
