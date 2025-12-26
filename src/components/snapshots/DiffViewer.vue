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
      html += `<span class="bg-red-500/10 dark:bg-red-900/20 text-red-700 dark:text-red-400 decoration-2 line-through decoration-red-500/50">${escapeHtml(text)}</span>`;
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
      html += `<span class="bg-[var(--accent)]/10 text-[var(--ink)] font-bold decoration-[var(--accent)] underline decoration-2 underline-offset-2">${escapeHtml(text)}</span>`;
    } else if (type === 0) {
      // Equal
      html += escapeHtml(text);
    }
  });
  return html.replace(/\n/g, '<br>');
});
</script>

<template>
  <div
    class="grid grid-cols-2 gap-px bg-[var(--stone)] h-full overflow-hidden select-text border border-[var(--stone)]"
  >
    <!-- Left: Original -->
    <div class="flex flex-col h-full bg-[var(--paper)]">
      <div
        class="bg-[var(--stone)] p-3 text-xs font-bold uppercase tracking-widest text-[var(--text-secondary)] sticky top-0 z-10"
      >
        Original
      </div>
      <div
        class="flex-1 overflow-auto p-6 font-mono text-sm leading-relaxed text-[var(--ink)] whitespace-pre-wrap"
        v-html="leftHtml"
      ></div>
    </div>

    <!-- Right: Modified -->
    <div class="flex flex-col h-full bg-[var(--paper)]">
      <div
        class="bg-[var(--stone)] p-3 text-xs font-bold uppercase tracking-widest text-[var(--text-secondary)] sticky top-0 z-10 flex justify-between"
      >
        <span>Current Draft</span>
        <span class="text-[var(--accent)]">● Live</span>
      </div>
      <div
        class="flex-1 overflow-auto p-6 font-mono text-sm leading-relaxed text-[var(--ink)] whitespace-pre-wrap"
        v-html="rightHtml"
      ></div>
    </div>
  </div>
</template>
