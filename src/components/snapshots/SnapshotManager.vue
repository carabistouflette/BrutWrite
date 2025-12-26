<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useSnapshotStore } from '../../stores/snapshots';
import DiffViewer from './DiffViewer.vue';
import ConfirmationModal from '../base/ConfirmationModal.vue';

const props = defineProps<{
  chapterId: string;
  currentContent: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'restore', content: string): void;
  (e: 'branch', content: string): void;
}>();

const snapshotStore = useSnapshotStore();
const selectedSnapshot = ref<string | null>(null);
const selectedSnapshotContent = ref<string>('');
const loadingContent = ref(false);

const showRestoreConfirm = ref(false);
const showBranchConfirm = ref(false);

onMounted(() => {
  snapshotStore.fetchSnapshots(props.chapterId);
});

async function selectSnapshot(filename: string) {
  selectedSnapshot.value = filename;
  loadingContent.value = true;
  try {
    selectedSnapshotContent.value = await snapshotStore.loadSnapshotContent(
      props.chapterId,
      filename
    );
  } finally {
    loadingContent.value = false;
  }
}

function formatDate(filename: string) {
  const match = filename.match(/^(\d{4}-\d{2}-\d{2}T\d{2}\d{2}\d{2})/);
  if (match) {
    const iso = match[1];
    const parsable = iso.replace(/(\d{2})(\d{2})(\d{2})$/, '$1:$2:$3');
    const d = new Date(parsable);
    return d.toLocaleString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }
  return filename;
}

function handleRestore() {
  if (!selectedSnapshotContent.value) return;
  showRestoreConfirm.value = true;
}

function confirmRestore() {
  if (!selectedSnapshotContent.value) return;
  emit('restore', selectedSnapshotContent.value);
  emit('close');
}

function handleBranch() {
  if (!selectedSnapshotContent.value) return;
  showBranchConfirm.value = true;
}

function confirmBranch() {
  if (!selectedSnapshotContent.value) return;
  emit('branch', selectedSnapshotContent.value);
  emit('close');
}
</script>

<template>
  <Teleport to="#app-scale-root">
    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 lg:p-12">
      <!-- Backdrop -->
      <div
        class="absolute inset-0 bg-black/40 backdrop-blur-md"
        style="will-change: opacity, backdrop-filter"
        @click="$emit('close')"
      ></div>

      <!-- Window Container -->
      <div
        class="relative w-full max-w-7xl h-[85%] flex bg-paper/95 backdrop-blur-xl border border-white/40 shadow-2xl rounded-2xl overflow-hidden text-ink modal-container"
        style="
          box-shadow:
            0 20px 50px -12px rgba(0, 0, 0, 0.2),
            0 0 0 1px rgba(255, 255, 255, 0.4) inset;
          will-change: transform, opacity;
        "
      >
        <!-- Sidebar -->
        <div class="w-80 cyber-glass border-r border-ink/5 flex flex-col">
          <div class="p-6 border-b border-ink/5 flex justify-between items-center">
            <h2 class="font-serif text-2xl italic font-bold">History</h2>
            <span class="text-xs bg-stone text-ink/60 px-2 py-1 rounded-full font-sans font-medium">
              {{ snapshotStore.snapshots.length }}
            </span>
          </div>

          <div class="overflow-y-auto flex-1 p-3 space-y-1 custom-scrollbar">
            <button
              v-for="snap in snapshotStore.snapshots"
              :key="snap"
              class="w-full text-left px-4 py-3 rounded-lg text-sm transition-all duration-200 group relative"
              :class="
                selectedSnapshot === snap
                  ? 'bg-stone shadow-sm text-accent'
                  : 'text-ink/70 hover:bg-stone/50 hover:text-ink'
              "
              @click="selectSnapshot(snap)"
            >
              <div class="font-medium flex justify-between items-center">
                {{ formatDate(snap) }}
                <span
                  v-if="selectedSnapshot === snap"
                  class="w-2 h-2 rounded-full bg-accent"
                ></span>
              </div>
              <div class="text-xs opacity-60 mt-1 truncate font-mono">
                {{ snap.split('_')[1]?.replace('.md', '') || 'Unknown' }}
              </div>
            </button>
          </div>
        </div>

        <!-- Main Content -->
        <div class="flex-1 flex flex-col h-full bg-transparent relative">
          <!-- Header -->
          <div
            class="h-16 border-b border-ink/5 flex items-center justify-between px-6 bg-transparent"
          >
            <div class="flex items-center gap-4">
              <h3 class="font-medium text-ink/80 text-lg tracking-tight">
                {{
                  selectedSnapshot ? `Comparing vs ${formatDate(selectedSnapshot)}` : 'Time Travel'
                }}
              </h3>
            </div>
            <div class="flex items-center gap-2">
              <template v-if="selectedSnapshot">
                <button
                  class="px-3 py-1.5 rounded-lg text-sm font-medium text-ink/70 hover:bg-stone/50 hover:text-ink transition-colors"
                  @click="handleBranch"
                >
                  Branch
                </button>
                <button
                  class="px-3 py-1.5 rounded-lg text-sm font-medium bg-accent text-white shadow-lg shadow-accent/20 hover:bg-accent-dark transition-colors"
                  @click="handleRestore"
                >
                  Restore This Version
                </button>
              </template>
              <button
                class="w-8 h-8 flex items-center justify-center rounded-full hover:bg-black/5 text-ink/40 hover:text-ink transition-colors ml-2"
                @click="$emit('close')"
              >
                <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            </div>
          </div>

          <!-- Body -->
          <div class="flex-1 overflow-hidden p-6 relative">
            <div
              v-if="!selectedSnapshot"
              class="flex flex-col items-center justify-center h-full text-ink/40"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-16 w-16 mb-4 opacity-50 stroke-[1.5]"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
              <p class="text-xl font-serif italic text-center max-w-md">
                "History is a vast early warning system." <br />
                <span class="text-sm font-sans not-italic mt-2 block opacity-70"
                  >- Select a snapshot from the timeline</span
                >
              </p>
            </div>
            <div v-else-if="loadingContent" class="flex items-center justify-center h-full">
              <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-accent"></div>
            </div>
            <DiffViewer v-else :original="selectedSnapshotContent" :modified="currentContent" />
          </div>
        </div>
      </div>
    </div>

    <ConfirmationModal
      :show="showRestoreConfirm"
      title="Restore Version"
      message="Are you sure you want to restore this version? Your current draft will be overwritten, but we'll save a snapshot of it just in case."
      confirm-label="Restore"
      :is-destructive="true"
      @confirm="confirmRestore"
      @cancel="showRestoreConfirm = false"
      @close="showRestoreConfirm = false"
    />

    <ConfirmationModal
      :show="showBranchConfirm"
      title="Branch Scene"
      message="This will create a new scene copy from this snapshot. You can then work on this alternate version alongside your original."
      confirm-label="Create Branch"
      @confirm="confirmBranch"
      @cancel="showBranchConfirm = false"
      @close="showBranchConfirm = false"
    />
  </Teleport>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: rgba(0, 0, 0, 0.1);
  border-radius: 2px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background-color: rgba(0, 0, 0, 0.2);
}

@keyframes modal-in {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(20px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

.modal-container {
  animation: modal-in 0.4s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  will-change: transform, opacity;
  backface-visibility: hidden;
  transform: translateZ(0);
}
</style>
