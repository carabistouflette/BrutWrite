<script setup lang="ts"></script>

<template>
  <div
    class="absolute bottom-14 left-4 flex gap-4 p-2 px-3 bg-white/80 border border-black/5 rounded-lg backdrop-blur-sm"
  >
    <div
      class="flex items-center gap-1.5 text-[10px] font-semibold text-black/60 uppercase tracking-wide"
    >
      <span class="w-2 h-2 rounded-full" style="background-color: var(--accent)"></span>
      <span>Protagonist</span>
    </div>
    <div
      class="flex items-center gap-1.5 text-[10px] font-semibold text-black/60 uppercase tracking-wide"
    >
      <span class="w-2 h-2 rounded-full" style="background-color: var(--color-antagonist)"></span>
      <span>Antagonist</span>
    </div>
    <div
      class="flex items-center gap-1.5 text-[10px] font-semibold text-black/60 uppercase tracking-wide"
    >
      <span class="w-2 h-2 rounded-full" style="background-color: var(--ink)"></span>
      <span>Secondary</span>
    </div>
    <div
      class="flex items-center gap-1.5 text-[10px] font-semibold text-black/60 uppercase tracking-wide"
    >
      <span class="w-2 h-2 rounded-full" style="background-color: var(--ink-muted)"></span>
      <span>Extra</span>
    </div>
  </div>
</template>
