<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useSnapshotStore } from '../../stores/snapshots';
import DiffViewer from './DiffViewer.vue';
import BaseButton from '../base/BaseButton.vue';
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
  // Format: 2024-01-01T120000_hash.md
  // Extract timestamp
  const match = filename.match(/^(\d{4}-\d{2}-\d{2}T\d{2}\d{2}\d{2})/);
  if (match) {
    const iso = match[1]; // 2024-01-01T120000
    // Formatting manually or using Date
    // Add colons for Date parse: 2024-01-01T12:00:00
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
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-8">
    <div
      class="bg-[var(--paper)] w-full max-w-7xl h-full shadow-[var(--shadow-brut)] flex overflow-hidden border border-[var(--stone)] animate-in fade-in zoom-in-95 duration-200"
    >
      <!-- Sidebar -->
      <div class="w-80 border-r border-[var(--stone)] flex flex-col bg-[var(--stone)]">
        <div
          class="p-4 border-b border-[var(--stone)] flex justify-between items-center bg-[var(--paper)]"
        >
          <h2 class="font-bold text-lg text-[var(--ink)] font-serif italic">History</h2>
          <span
            class="text-xs bg-[var(--ink)] text-[var(--paper)] px-2 py-1 rounded-full font-mono"
          >
            {{ snapshotStore.snapshots.length }}
          </span>
        </div>

        <div class="overflow-y-auto flex-1 p-3 space-y-2">
          <button
            v-for="snap in snapshotStore.snapshots"
            :key="snap"
            class="w-full text-left px-4 py-3 text-sm transition-all duration-200 border border-transparent font-mono group"
            :class="
              selectedSnapshot === snap
                ? 'bg-[var(--ink)] text-[var(--paper)] shadow-[4px_4px_0_0_black]'
                : 'bg-[var(--paper)] text-[var(--ink)] hover:border-[var(--ink)] border-[var(--stone)]'
            "
            @click="selectSnapshot(snap)"
          >
            <div class="font-bold flex justify-between">
              {{ formatDate(snap) }}
              <span v-if="selectedSnapshot === snap" class="text-[var(--accent)]">●</span>
            </div>
            <div
              class="text-xs opacity-60 mt-1 truncate group-hover:opacity-100 transition-opacity"
            >
              {{ snap.split('_')[1]?.replace('.md', '') || 'Unknown' }}
            </div>
          </button>
        </div>
      </div>

      <!-- Main Content -->
      <div class="flex-1 flex flex-col h-full bg-[var(--paper)] relative">
        <!-- Header -->
        <div
          class="h-16 border-b border-[var(--stone)] flex items-center justify-between px-6 bg-[var(--paper)] z-20"
        >
          <div class="flex items-center gap-4">
            <h3 class="font-semibold text-[var(--ink)] font-serif text-lg">
              {{
                selectedSnapshot ? `Comparing vs ${formatDate(selectedSnapshot)}` : 'Time Travel'
              }}
            </h3>
          </div>
          <div class="flex items-center gap-3">
            <BaseButton v-if="selectedSnapshot" variant="secondary" @click="handleBranch">
              Branch
            </BaseButton>
            <BaseButton v-if="selectedSnapshot" variant="primary" @click="handleRestore">
              Restore This Version
            </BaseButton>
            <BaseButton variant="ghost" @click="$emit('close')"> Close </BaseButton>
          </div>
        </div>

        <!-- Body -->
        <div class="flex-1 overflow-hidden p-6 bg-[var(--paper)] relative">
          <div
            v-if="!selectedSnapshot"
            class="flex flex-col items-center justify-center h-full text-[var(--ink)] opacity-40"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-24 w-24 mb-6 stroke-1"
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
            <p class="text-2xl font-serif italic text-center max-w-md">
              "History is a vast early warning system." <br />
              <span class="text-base font-sans not-italic mt-2 block opacity-70"
                >- Select a snapshot from the timeline</span
              >
            </p>
          </div>
          <div v-else-if="loadingContent" class="flex items-center justify-center h-full">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-[var(--ink)]"></div>
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
</template>
