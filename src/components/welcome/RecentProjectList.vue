<script setup lang="ts">
const props = defineProps<{
  recentProjects: string[];
}>();

const emit = defineEmits<{
  (e: 'open-project', path: string): void;
}>();

const getFileName = (path: string) => {
  return path.split(/[\\/]/).pop()?.replace('.json', '') || 'Untitled';
};
</script>

<template>
  <div class="space-y-4">
    <h2 class="text-xs uppercase tracking-[0.3em] opacity-40 font-medium">Recent Manuscripts</h2>
    <div v-if="recentProjects.length > 0" class="space-y-3">
      <button
        v-for="path in recentProjects"
        :key="path"
        class="w-full text-left p-4 rounded-xl border border-ink/5 hover:border-accent/30 hover:bg-white/50 transition-all group flex items-center justify-between cyber-glass"
        @click="emit('open-project', path)"
      >
        <div class="flex flex-col">
          <span class="text-sm font-medium">{{ getFileName(path) }}</span>
          <span class="text-[10px] opacity-30 truncate max-w-[200px]">{{ path }}</span>
        </div>
        <svg
          class="w-4 h-4 opacity-0 group-hover:opacity-100 -translate-x-2 group-hover:translate-x-0 transition-all text-accent"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M14 5l7 7m0 0l-7 7m7-7H3"
          />
        </svg>
      </button>
    </div>
    <div
      v-else
      class="p-8 border border-dashed border-ink/10 rounded-2xl flex flex-col items-center justify-center text-center space-y-2"
    >
      <p class="text-sm opacity-30 italic">No recent projects found</p>
    </div>
  </div>
</template>

<style scoped>
.cyber-glass {
  backdrop-filter: blur(12px) saturate(180%);
  -webkit-backdrop-filter: blur(12px) saturate(180%);
}
</style>
