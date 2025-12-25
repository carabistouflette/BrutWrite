<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useSettingsStore } from '../../stores/settings';

const settingsStore = useSettingsStore();
const { settings } = storeToRefs(settingsStore);
</script>

<template>
  <div class="space-y-8">
    <!-- Author Identity -->
    <div class="space-y-4">
      <h3
        class="text-[10px] font-sans font-bold uppercase tracking-[0.2em] text-accent/80 border-b border-accent/10 pb-2"
      >
        Identity
      </h3>
      <div class="space-y-2">
        <label class="block text-xs font-semibold text-ink/80">Author Name</label>
        <input
          v-model="settings.general.authorName"
          type="text"
          placeholder="e.g. Ernest Hemingway"
          class="w-full bg-white/70 dark:bg-white/5 border border-white dark:border-white/10 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-accent focus:ring-4 focus:ring-accent/10 transition-all placeholder:text-ink/20 shadow-[0_2px_10px_-4px_rgba(0,0,0,0.05)]"
        />
        <p class="text-[10px] text-ink/40 flex items-center gap-1.5 px-1">
          <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          Used for metadata when exporting your book.
        </p>
      </div>
    </div>

    <!-- Goals -->
    <div class="space-y-4">
      <h3
        class="text-[10px] font-sans font-bold uppercase tracking-[0.2em] text-accent/80 border-b border-accent/10 pb-2"
      >
        Goals
      </h3>
      <div class="space-y-2">
        <label class="block text-xs font-semibold text-ink/80">Daily Word Goal</label>
        <div class="flex items-center gap-3">
          <div class="relative group flex-1 max-w-[160px]">
            <input
              v-model.number="settings.general.dailyGoal"
              type="number"
              class="w-full bg-white/70 dark:bg-white/5 border border-white dark:border-white/10 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-accent focus:ring-4 focus:ring-accent/10 transition-all shadow-[0_2px_10px_-4px_rgba(0,0,0,0.05)] input-with-unit"
            />
            <div
              class="absolute right-3 top-1/2 -translate-y-1/2 text-[10px] font-bold text-ink/30 uppercase tracking-tighter"
            >
              words
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Storage -->
    <div class="space-y-4">
      <h3
        class="text-[10px] font-sans font-bold uppercase tracking-[0.2em] text-accent/80 border-b border-accent/10 pb-2"
      >
        Data Management
      </h3>
      <div class="space-y-2">
        <label class="block text-xs font-semibold text-ink/80">Auto-Save Interval</label>
        <div class="flex items-center gap-3">
          <div class="relative group flex-1 max-w-[160px]">
            <input
              v-model.number="settings.general.autoSaveInterval"
              type="number"
              min="5"
              class="w-full bg-white/70 dark:bg-white/5 border border-white dark:border-white/10 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-accent focus:ring-4 focus:ring-accent/10 transition-all shadow-[0_2px_10px_-4px_rgba(0,0,0,0.05)] input-with-unit"
            />
            <div
              class="absolute right-3 top-1/2 -translate-y-1/2 text-[10px] font-bold text-ink/30 uppercase tracking-tighter"
            >
              sec
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
