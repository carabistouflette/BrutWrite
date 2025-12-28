<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useSettingsStore } from '../../stores/settings';

const settingsStore = useSettingsStore();
const { settings } = storeToRefs(settingsStore);
</script>

<template>
  <div class="space-y-8">
    <!-- Graph Analysis -->
    <div class="space-y-4">
      <h3
        class="text-[10px] font-sans font-bold uppercase tracking-[0.2em] text-accent/80 border-b border-accent/10 pb-2"
      >
        Character Graph Analysis
      </h3>

      <!-- Proximity Window -->
      <div class="space-y-2">
        <label class="block text-xs font-semibold text-ink/80">Proximity Window</label>
        <div class="flex items-center gap-3">
          <div class="relative group flex-1 max-w-[160px]">
            <input
              v-model.number="settings.intelligence.proximityWindow"
              type="number"
              min="10"
              max="200"
              step="10"
              class="w-full bg-white/70 dark:bg-white/5 border border-white dark:border-white/10 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-accent focus:ring-4 focus:ring-accent/10 transition-all shadow-[0_2px_10px_-4px_rgba(0,0,0,0.05)] input-with-unit"
            />
            <div
              class="absolute right-3 top-1/2 -translate-y-1/2 text-[10px] font-bold text-ink/30 uppercase tracking-tighter"
            >
              words
            </div>
          </div>
        </div>
        <p class="text-[10px] text-ink/40 flex items-center gap-1.5 px-1">
          <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          Characters mentioned within this word distance are considered related.
        </p>
      </div>

      <!-- Prune Threshold -->
      <div class="space-y-2">
        <label class="block text-xs font-semibold text-ink/80">Edge Threshold</label>
        <div class="flex items-center gap-3">
          <div class="relative group flex-1 max-w-[160px]">
            <input
              v-model.number="settings.intelligence.pruneThreshold"
              type="number"
              min="0.01"
              max="0.5"
              step="0.01"
              class="w-full bg-white/70 dark:bg-white/5 border border-white dark:border-white/10 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-accent focus:ring-4 focus:ring-accent/10 transition-all shadow-[0_2px_10px_-4px_rgba(0,0,0,0.05)]"
            />
          </div>
        </div>
        <p class="text-[10px] text-ink/40 flex items-center gap-1.5 px-1">
          <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          Minimum interaction weight to show a connection. Lower = more edges.
        </p>
      </div>
    </div>

    <!-- Automation -->
    <div class="space-y-4">
      <h3
        class="text-[10px] font-sans font-bold uppercase tracking-[0.2em] text-accent/80 border-b border-accent/10 pb-2"
      >
        Automation
      </h3>

      <!-- Auto-Analyze Toggle -->
      <div class="flex items-center justify-between">
        <div>
          <label class="block text-xs font-semibold text-ink/80">Auto-Analyze on Save</label>
          <p class="text-[10px] text-ink/40 mt-0.5">
            Automatically re-analyze character graph when saving a chapter.
          </p>
        </div>
        <button
          type="button"
          role="switch"
          :aria-checked="settings.intelligence.autoAnalyzeOnSave"
          class="relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2"
          :class="settings.intelligence.autoAnalyzeOnSave ? 'bg-accent' : 'bg-ink/20'"
          @click="
            settings.intelligence.autoAnalyzeOnSave = !settings.intelligence.autoAnalyzeOnSave
          "
        >
          <span
            aria-hidden="true"
            class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
            :class="settings.intelligence.autoAnalyzeOnSave ? 'translate-x-5' : 'translate-x-0'"
          />
        </button>
      </div>
    </div>
  </div>
</template>
