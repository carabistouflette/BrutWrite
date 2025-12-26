<script setup lang="ts">
import { computed } from 'vue';
import { diff_match_patch } from 'diff-match-patch';

const props = defineProps<{
  original: string;
  modified: string;
}>();

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
      html += `<span class="bg-red-100 text-red-800 decoration-clone px-1 rounded mx-0.5">${escapeHtml(text)}</span>`;
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
      html += `<span class="bg-orange-100 text-orange-800 decoration-clone px-1 rounded mx-0.5 font-medium">${escapeHtml(text)}</span>`;
    } else if (type === 0) {
      // Equal
      html += escapeHtml(text);
    }
  });
  return html.replace(/\n/g, '<br>');
});
</script>

<template>
  <div class="grid grid-cols-2 gap-4 h-full select-text">
    <!-- Left: Original -->
    <div
      class="flex flex-col h-full bg-ink/3 dark:bg-ink/5 rounded-xl border border-ink/5 overflow-hidden"
    >
      <div
        class="bg-ink/3 dark:bg-ink/5 p-3 text-xs font-bold uppercase tracking-widest text-ink/40 sticky top-0 z-10 border-b border-ink/5 backdrop-blur-sm"
      >
        Original
      </div>
      <div
        class="flex-1 overflow-auto p-6 font-mono text-sm leading-relaxed text-ink/80 whitespace-pre-wrap"
        v-html="leftHtml"
      ></div>
    </div>

    <!-- Right: Modified -->
    <div
      class="flex flex-col h-full bg-ink/3 dark:bg-ink/5 rounded-xl border border-ink/5 overflow-hidden"
    >
      <div
        class="bg-ink/3 dark:bg-ink/5 p-3 text-xs font-bold uppercase tracking-widest text-ink/40 sticky top-0 z-10 flex justify-between border-b border-ink/5 backdrop-blur-sm"
      >
        <span>Current Draft</span>
        <span class="text-accent flex items-center gap-1.5"
          ><span class="w-1.5 h-1.5 rounded-full bg-accent animate-pulse"></span> Live</span
        >
      </div>
      <div
        class="flex-1 overflow-auto p-6 font-mono text-sm leading-relaxed text-ink/80 whitespace-pre-wrap"
        v-html="rightHtml"
      ></div>
    </div>
  </div>
</template>
