<script setup lang="ts">
import { computed } from 'vue';
import { diff_match_patch } from 'diff-match-patch';

const props = defineProps<{
  original: string;
  modified: string;
}>();

// diff-match-patch usually exports the class.
// Note: Depending on tsconfig, it might be import DiffMatchPatch from ... or just diff_match_patch
const dmp = new diff_match_patch();

const diffs = computed(() => {
  const d = dmp.diff_main(props.original, props.modified);
  dmp.diff_cleanupSemantic(d);
  return d;
});

function escapeHtml(text: string) {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

const leftHtml = computed(() => {
  let html = '';
  diffs.value.forEach((part) => {
    const [type, text] = part;
    if (type === -1) {
      // Delete
      html += `<span class="bg-red-200 dark:bg-red-900/50 text-red-800 dark:text-red-200 decoration-clone p-0.5 rounded-sm">${escapeHtml(text)}</span>`;
    } else if (type === 0) {
      // Equal
      html += escapeHtml(text);
    }
  });
  return html.replace(/\n/g, '<br>');
});

const rightHtml = computed(() => {
  let html = '';
  diffs.value.forEach((part) => {
    const [type, text] = part;
    if (type === 1) {
      // Insert
      html += `<span class="bg-green-200 dark:bg-green-900/50 text-green-800 dark:text-green-200 decoration-clone p-0.5 rounded-sm">${escapeHtml(text)}</span>`;
    } else if (type === 0) {
      // Equal
      html += escapeHtml(text);
    }
  });
  return html.replace(/\n/g, '<br>');
});
</script>

<template>
  <div class="grid grid-cols-2 gap-4 h-full overflow-hidden select-text">
    <!-- Left: Original -->
    <div
      class="flex flex-col h-full border border-gray-200 dark:border-gray-700 rounded-md bg-white dark:bg-gray-950"
    >
      <div
        class="bg-gray-50 dark:bg-gray-800 p-2 text-xs font-bold uppercase tracking-wider text-gray-500 border-b border-gray-200 dark:border-gray-700 sticky top-0 z-10"
      >
        Original
      </div>
      <div
        class="flex-1 overflow-auto p-4 font-mono text-sm leading-relaxed text-gray-800 dark:text-gray-300"
        v-html="leftHtml"
      ></div>
    </div>

    <!-- Right: Modified -->
    <div
      class="flex flex-col h-full border border-gray-200 dark:border-gray-700 rounded-md bg-white dark:bg-gray-950"
    >
      <div
        class="bg-gray-50 dark:bg-gray-800 p-2 text-xs font-bold uppercase tracking-wider text-gray-500 border-b border-gray-200 dark:border-gray-700 sticky top-0 z-10"
      >
        Current Draft
      </div>
      <div
        class="flex-1 overflow-auto p-4 font-mono text-sm leading-relaxed text-gray-800 dark:text-gray-300"
        v-html="rightHtml"
      ></div>
    </div>
  </div>
</template>
