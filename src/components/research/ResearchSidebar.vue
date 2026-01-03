<template>
  <aside
    class="flex flex-col h-full w-full bg-paper text-ink overflow-hidden font-sans border-r border-ink/5 relative"
    @dragover.prevent="isDragging = true"
    @dragleave.prevent="isDragging = false"
    @drop.prevent="handleDrop"
  >
    <!-- Drop Overlay -->
    <div
      v-if="isDragging"
      class="absolute inset-0 z-50 bg-accent/20 backdrop-blur-sm border-2 border-accent border-dashed m-2 rounded-xl flex items-center justify-center pointer-events-none"
    >
      <div class="bg-paper p-4 rounded-xl shadow-xl flex flex-col items-center gap-2">
        <BaseIcon name="download" size="32" class="text-accent" />
        <span class="text-xs font-bold uppercase tracking-widest text-ink">Drop to Import</span>
      </div>
    </div>

    <!-- Header Area -->
    <div class="pt-8 pb-6 px-8 flex flex-col gap-1 shrink-0">
      <div class="flex items-center justify-between">
        <h2 class="font-serif text-2xl text-ink tracking-tight font-medium">Research</h2>
        <div class="flex items-center gap-2">
          <span
            class="text-[10px] tracking-widest uppercase font-bold text-ink/40 border border-stone px-2 py-0.5 rounded-full"
          >
            {{ store.artifacts.length }}
          </span>
          <button
            class="p-1 rounded-md text-ink/30 hover:text-ink hover:bg-stone transition-colors"
            @click="$emit('close')"
          >
            <BaseIcon name="x" size="16" />
          </button>
        </div>
      </div>
      <p class="text-[11px] text-ink/40 uppercase tracking-widest font-semibold mt-1">
        Vault Content
      </p>
    </div>

    <!-- Search Area -->
    <div class="px-6 mb-4 shrink-0">
      <div class="relative group">
        <div class="absolute inset-y-0 left-3 flex items-center pointer-events-none">
          <BaseIcon
            name="search"
            size="14"
            class="text-ink/30 transition-colors duration-300 group-focus-within:text-accent"
          />
        </div>
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Filter sources or #tags..."
          class="w-full bg-stone/50 hover:bg-stone/80 transition-all duration-300 text-xs font-medium text-ink py-3 pl-10 pr-4 rounded-lg border border-transparent focus:border-accent/40 focus:bg-stone focus:outline-none placeholder-ink/20"
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
            ? 'bg-stone text-ink shadow-sm'
            : 'text-ink/60 hover:text-ink hover:bg-stone/40',
        ]"
        @click="store.setActiveArtifact(item)"
        @contextmenu.prevent="handleContextMenu($event, item)"
      >
        <!-- Active Indicator -->
        <div
          v-if="store.activeArtifact?.id === item.id"
          class="absolute left-0 top-1/2 -translate-y-1/2 h-8 w-1 bg-accent rounded-r-sm shadow-[0_0_10px_rgba(var(--color-accent-rgb),0.3)]"
        ></div>

        <!-- Start Icon -->
        <div
          class="ml-1 shrink-0 p-2 rounded-md transition-colors duration-300"
          :class="
            store.activeArtifact?.id === item.id
              ? 'bg-paper text-accent'
              : 'bg-stone/50 text-ink/30 group-hover:bg-stone group-hover:text-ink/60'
          "
        >
          <BaseIcon v-if="item.file_type === 'pdf'" name="file" size="16" />
          <BaseIcon
            v-else-if="['png', 'jpg', 'jpeg', 'webp'].includes(item.file_type)"
            name="image"
            size="16"
          />
          <BaseIcon v-else name="fileText" size="16" />
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
                  ? 'text-ink/40'
                  : 'text-ink/30 group-hover:text-ink/40'
              "
            >
              {{ item.file_type }}
            </span>
            <!-- Tags -->
            <span
              v-for="tag in item.tags.slice(0, 3)"
              :key="tag"
              class="text-[9px] font-bold px-1.5 py-px rounded-sm bg-stone text-ink/40"
            >
              #{{ tag }}
            </span>
            <span v-if="item.tags.length > 3" class="text-[9px] text-ink/20">
              +{{ item.tags.length - 3 }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Bottom Action -->
    <div class="p-6 shrink-0 border-t border-ink/5 bg-paper/80 backdrop-blur-sm flex gap-2">
      <button
        class="group relative flex-1 overflow-hidden rounded-xl bg-ink text-paper py-3.5 px-2 text-xs font-bold uppercase tracking-widest transition-all duration-300 hover:shadow-lg hover:-translate-y-0.5"
        @click="$emit('add')"
      >
        <span class="relative z-10 flex items-center justify-center gap-2">
          <BaseIcon name="plusSimple" size="14" stroke-width="2.5" />
          Add
        </span>
      </button>

      <button
        class="group relative flex-1 overflow-hidden rounded-xl bg-stone text-ink/70 border border-ink/10 py-3.5 px-2 text-xs font-bold uppercase tracking-widest transition-all duration-300 hover:bg-stone/80 hover:text-ink hover:-translate-y-0.5"
        @click="handleNewNote"
      >
        <span class="relative z-10 flex items-center justify-center gap-2">
          <BaseIcon name="edit" size="14" />
          Note
        </span>
      </button>
    </div>

    <!-- Context Menu -->
    <ContextMenu :show="showMenu" :x="menuPos.x" :y="menuPos.y" @close="closeMenu">
      <div
        class="px-3 py-2 text-[10px] font-black uppercase text-ink/40 border-b border-ink/10 mb-1"
      >
        ACTIONS
      </div>
      <div class="menu-item menu-item-default" @click="handleRename">Rename</div>
      <div class="menu-item menu-item-default" @click="handleEditTags">Edit Tags</div>
      <div class="border-t border-ink/5 my-1"></div>
      <div class="menu-item menu-item-danger" @click="handleDelete">Delete</div>
    </ContextMenu>

    <!-- Tag Manager Modal -->
    <TagManagerModal
      :show="showTagManager"
      :initial-tags="managingArtifact?.tags || []"
      :artifact-name="managingArtifact?.name || ''"
      @close="showTagManager = false"
      @save="handleSaveTags"
    />
  </aside>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useResearchStore } from '../../stores/research';
import type { ResearchArtifact } from '../../api/research';
import { listen } from '@tauri-apps/api/event';
import ContextMenu from '../base/ContextMenu.vue';
import TagManagerModal from './TagManagerModal.vue';
import { useDialogStore } from '../../stores/dialog';
import BaseIcon from '../base/BaseIcon.vue';
import { useContextMenu } from '../../composables/ui/useContextMenu';
import { APP_CONSTANTS } from '../../config/constants';

const store = useResearchStore();
const dialogStore = useDialogStore();
const searchQuery = ref('');
const isDragging = ref(false);

defineEmits(['close', 'add']);

// --- Context Menu Composable ---
const {
  showMenu,
  menuPos,
  contextData: targetArtifact,
  openMenu,
  closeMenu,
} = useContextMenu<ResearchArtifact>();

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
  interface TauriDragDropPayload {
    paths: string[];
  }

  unlisten = await listen<TauriDragDropPayload>('tauri://drag-drop', (event) => {
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

// --- Context Menu Handlers ---
const handleContextMenu = (e: MouseEvent, item: ResearchArtifact) => {
  openMenu(e, item);
};

// --- Modal State Handling via DialogStore ---

// --- Tag Management ---
const showTagManager = ref(false);
const managingArtifact = ref<ResearchArtifact | null>(null);

const handleEditTags = () => {
  if (!targetArtifact.value) return;
  managingArtifact.value = targetArtifact.value;
  showTagManager.value = true;
  closeMenu();
};

const handleSaveTags = (newTags: string[]) => {
  if (!managingArtifact.value) return;
  const item = JSON.parse(JSON.stringify(managingArtifact.value));
  item.tags = newTags;
  store.updateArtifact(item);
  showTagManager.value = false;
  managingArtifact.value = null;
};

// --- Handlers ---
const handleNewNote = async () => {
  const name = await dialogStore.prompt({
    title: APP_CONSTANTS.STRINGS.PROMPTS.NEW_NOTE_TITLE,
    message: APP_CONSTANTS.STRINGS.PROMPTS.NEW_NOTE_MESSAGE,
    initialValue: '',
    placeholder: APP_CONSTANTS.STRINGS.PROMPTS.NEW_NOTE_PLACEHOLDER,
  });
  if (name) {
    await store.createNote(name);
  }
};

const handleRename = async () => {
  if (!targetArtifact.value) return;
  const name = await dialogStore.prompt({
    title: APP_CONSTANTS.STRINGS.PROMPTS.RENAME_ARTIFACT_TITLE,
    message: APP_CONSTANTS.STRINGS.PROMPTS.RENAME_ARTIFACT_MESSAGE,
    initialValue: targetArtifact.value.name,
  });
  if (name) {
    await store.renameArtifact(targetArtifact.value.id, name);
  }
  closeMenu();
};

const handleDelete = async () => {
  if (!targetArtifact.value) return;
  if (
    await dialogStore.confirm({
      title: APP_CONSTANTS.STRINGS.PROMPTS.DELETE_ARTIFACT_TITLE,
      message: APP_CONSTANTS.STRINGS.PROMPTS.DELETE_ARTIFACT_MESSAGE(targetArtifact.value.name),
      isDestructive: true,
    })
  ) {
    await store.deleteArtifact(targetArtifact.value.id);
  }
  closeMenu();
};
</script>

<style scoped>
/* Scoped styles if any were present */
</style>
