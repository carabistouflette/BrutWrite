<template>
  <div class="flex flex-col h-full bg-paper">
    <!-- Toolbar -->
    <div class="flex items-center justify-between px-4 py-2 border-b border-ink/5 bg-stone/40">
      <div class="flex items-center gap-2">
        <span class="text-[10px] font-bold uppercase tracking-widest text-ink/40">Editing</span>
        <span class="text-sm font-medium text-ink">{{ store.activeArtifact?.name }}</span>
      </div>
      <div class="flex items-center gap-2">
        <span v-if="loading" class="text-[10px] uppercase font-bold tracking-tighter text-ink/40"
          >Loading...</span
        >
        <span
          v-else-if="saving"
          class="text-[10px] uppercase font-bold tracking-tighter text-accent animate-pulse"
          >Saving...</span
        >
        <span
          v-else-if="saved"
          class="text-[10px] uppercase font-bold tracking-tighter text-green-600"
          >Saved</span
        >
      </div>
    </div>

    <p v-if="loadFailed" role="alert" class="px-6 pt-4 text-sm text-red-600">
      Could not load this note. Editing is disabled to protect its existing content.
    </p>

    <!-- Editor Area -->
    <textarea
      v-model="content"
      :disabled="loading || loadFailed"
      class="flex-1 w-full bg-transparent p-6 resize-none outline-none text-ink font-mono text-sm leading-relaxed custom-scrollbar placeholder-ink/20 disabled:opacity-60"
      placeholder="Start typing your note here..."
      @input="handleInput"
    ></textarea>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onBeforeUnmount } from 'vue';
import { useResearchStore } from '../../stores/research';
import { useProjectStore } from '../../stores/project';
import { readFile } from '@tauri-apps/plugin-fs';
import { APP_CONSTANTS } from '../../config/constants';
import { useAppStatus } from '../../composables/ui/useAppStatus';
import { registerResearchNoteSession } from '../../composables/domain/research/useResearchNoteSession';

const props = defineProps<{
  path: string;
  id: string; // Artifact ID
}>();

const store = useResearchStore();
const projectStore = useProjectStore();
let projectId = projectStore.projectId;
let projectRoot = projectStore.path;
const { notifyError } = useAppStatus();
const content = ref('');
const loading = ref(true);
const loadFailed = ref(false);
const saving = ref(false);
const saved = ref(false);
let saveTimeout: ReturnType<typeof setTimeout> | undefined;
let savedTimeout: ReturnType<typeof setTimeout> | undefined;
interface PendingNoteSave {
  key: string;
  id: string;
  expectedPath: string;
  projectId: string | undefined;
  projectRoot: string | undefined;
  content: string;
}
const pendingSaves = new Map<string, PendingNoteSave>();
let saveInFlight: Promise<void> | null = null;
let sessionUnmounted = false;
let unregisterSession: (() => void) | undefined;
let loadRequest = 0;

const clearSaveTimeout = () => {
  if (saveTimeout !== undefined) {
    clearTimeout(saveTimeout);
    saveTimeout = undefined;
  }
};

const clearSavedTimeout = () => {
  if (savedTimeout !== undefined) {
    clearTimeout(savedTimeout);
    savedTimeout = undefined;
  }
};

const flushPendingSave = async () => {
  clearSaveTimeout();
  if (saveInFlight) {
    await saveInFlight;
    if (pendingSaves.size > 0) await flushPendingSave();
    if (sessionUnmounted && pendingSaves.size === 0) unregisterSession?.();
    return;
  }

  const pending = pendingSaves.values().next().value;
  if (!pending) {
    if (sessionUnmounted) unregisterSession?.();
    return;
  }

  const save = store.saveNoteContent(pending.id, pending.expectedPath, pending.content);
  saveInFlight = save;
  try {
    await save;
    if (pendingSaves.get(pending.key) === pending) {
      pendingSaves.delete(pending.key);
    }
    if (
      props.id === pending.id &&
      props.path === pending.expectedPath &&
      projectStore.projectId === pending.projectId &&
      projectStore.path === pending.projectRoot &&
      pendingSaves.size === 0
    ) {
      saving.value = false;
      saved.value = true;
      clearSavedTimeout();
      savedTimeout = setTimeout(() => {
        saved.value = false;
        savedTimeout = undefined;
      }, APP_CONSTANTS.TIMEOUTS.DEBOUNCE_DEFAULT * 6);
    }
  } catch (error) {
    if (
      props.id === pending.id &&
      props.path === pending.expectedPath &&
      projectStore.projectId === pending.projectId &&
      projectStore.path === pending.projectRoot &&
      pendingSaves.get(pending.key) === pending
    ) {
      saving.value = false;
      saved.value = false;
    }
    throw error;
  } finally {
    if (saveInFlight === save) saveInFlight = null;
  }

  if (pendingSaves.size > 0) await flushPendingSave();
  if (sessionUnmounted && pendingSaves.size === 0) unregisterSession?.();
};

const handleInput = () => {
  saved.value = false;
  saving.value = true;
  const key = `${props.id}\0${props.path}`;
  pendingSaves.set(key, {
    key,
    id: props.id,
    expectedPath: props.path,
    projectId,
    projectRoot,
    content: content.value,
  });
  clearSaveTimeout();
  saveTimeout = setTimeout(() => {
    saveTimeout = undefined;
    void flushPendingSave().catch(() => {
      // The store has already reported the save failure.
    });
  }, 1000);
};

watch(
  () => [props.id, props.path] as const,
  async ([, path]) => {
    const request = ++loadRequest;
    clearSaveTimeout();
    clearSavedTimeout();
    saving.value = false;
    saved.value = false;
    loading.value = true;
    loadFailed.value = false;
    content.value = '';

    try {
      await flushPendingSave();
    } catch {
      // Preserve the next artifact load after the store reports a pending-save failure.
    }

    try {
      const raw = await readFile(path);
      if (request === loadRequest) {
        content.value = new TextDecoder().decode(raw);
      }
    } catch (error) {
      if (request === loadRequest) {
        loadFailed.value = true;
        notifyError('Failed to load note content', error);
      }
    } finally {
      if (request === loadRequest) {
        loading.value = false;
      }
    }
  },
  { immediate: true }
);
const registerCurrentProjectSession = () => {
  if (!projectId || !projectRoot) {
    unregisterSession = undefined;
    return;
  }
  unregisterSession = registerResearchNoteSession(
    projectStore,
    projectId,
    projectRoot,
    flushPendingSave
  );
};

registerCurrentProjectSession();

watch(
  () => [projectStore.projectId, projectStore.path] as const,
  async ([nextProjectId, nextProjectRoot]) => {
    if (nextProjectId === projectId && nextProjectRoot === projectRoot) return;
    try {
      await flushPendingSave();
    } catch {
      return;
    }
    unregisterSession?.();
    projectId = nextProjectId;
    projectRoot = nextProjectRoot;
    registerCurrentProjectSession();
  }
);

onBeforeUnmount(() => {
  sessionUnmounted = true;
  loadRequest++;
  clearSaveTimeout();
  clearSavedTimeout();
  void flushPendingSave().then(
    () => unregisterSession?.(),
    () => {
      // Keep the registered flusher so a later project transition can retry the pending save.
    }
  );
});
</script>

<style scoped>
/* Ultra-minimal scrollbar */
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: var(--stone);
  border-radius: 99px;
}
.custom-scrollbar:hover::-webkit-scrollbar-thumb {
  background: var(--ink-rgb);
  opacity: 0.2;
}
</style>
