<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useSnapshotStore } from '../../stores/snapshots';
import DiffViewer from './DiffViewer.vue';
import BaseButton from '../base/BaseButton.vue';

const props = defineProps<{
  chapterId: string;
  currentContent: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'restore', content: string): void;
}>();

const snapshotStore = useSnapshotStore();
const selectedSnapshot = ref<string | null>(null);
const selectedSnapshotContent = ref<string>('');
const loadingContent = ref(false);

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
  if (
    confirm('Are you sure you want to restore this version? Current unsaved changes might be lost.')
  ) {
    emit('restore', selectedSnapshotContent.value);
    emit('close');
  }
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-8">
    <div
      class="bg-white dark:bg-gray-900 w-full max-w-7xl h-full rounded-xl shadow-2xl flex overflow-hidden border border-gray-200 dark:border-gray-800 animate-in fade-in zoom-in-95 duration-200"
    >
      <!-- Sidebar -->
      <div
        class="w-80 border-r border-gray-200 dark:border-gray-800 flex flex-col bg-gray-50 dark:bg-gray-950/50"
      >
        <div
          class="p-4 border-b border-gray-200 dark:border-gray-800 flex justify-between items-center"
        >
          <h2 class="font-bold text-lg text-gray-800 dark:text-gray-100">History</h2>
          <span
            class="text-xs bg-gray-200 dark:bg-gray-800 px-2 py-1 rounded-full text-gray-600 dark:text-gray-400"
          >
            {{ snapshotStore.snapshots.length }}
          </span>
        </div>

        <div class="overflow-y-auto flex-1 p-3 space-y-2">
          <button
            v-for="snap in snapshotStore.snapshots"
            :key="snap"
            class="w-full text-left px-4 py-3 rounded-lg text-sm transition-all duration-200 border"
            :class="
              selectedSnapshot === snap
                ? 'bg-blue-600 text-white border-transparent shadow-lg shadow-blue-500/30'
                : 'bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 border-gray-200 dark:border-gray-700 hover:bg-gray-100 dark:hover:bg-gray-700 hover:border-gray-300'
            "
            @click="selectSnapshot(snap)"
          >
            <div class="font-medium">{{ formatDate(snap) }}</div>
            <div class="text-xs opacity-70 mt-1 font-mono truncate">
              {{ snap.split('_')[1]?.replace('.md', '') || 'Unknown' }}
            </div>
          </button>
        </div>
      </div>

      <!-- Main Content -->
      <div class="flex-1 flex flex-col h-full bg-white dark:bg-gray-900 relative">
        <!-- Header -->
        <div
          class="h-16 border-b border-gray-200 dark:border-gray-800 flex items-center justify-between px-6 bg-white dark:bg-gray-900 z-20"
        >
          <div class="flex items-center gap-4">
            <h3 class="font-semibold text-gray-800 dark:text-white">
              {{
                selectedSnapshot
                  ? `Comparing vs ${formatDate(selectedSnapshot)}`
                  : 'Select a snapshot to compare'
              }}
            </h3>
          </div>
          <div class="flex items-center gap-3">
            <BaseButton v-if="selectedSnapshot" variant="primary" @click="handleRestore">
              Restore This Version
            </BaseButton>
            <BaseButton variant="secondary" @click="$emit('close')"> Close </BaseButton>
          </div>
        </div>

        <!-- Body -->
        <div class="flex-1 overflow-hidden p-6 bg-gray-50 dark:bg-gray-950/30 relative">
          <div
            v-if="!selectedSnapshot"
            class="flex flex-col items-center justify-center h-full text-gray-400"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-16 w-16 mb-4 opacity-50"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <p class="text-lg">Select a version from the timeline to compare</p>
          </div>
          <div v-else-if="loadingContent" class="flex items-center justify-center h-full">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          </div>
          <DiffViewer v-else :original="selectedSnapshotContent" :modified="currentContent" />
        </div>
      </div>
    </div>
  </div>
</template>
