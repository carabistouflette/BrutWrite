<script setup lang="ts">
import { useSettings } from '../../composables/logic/useSettings';

const { settings } = useSettings();
</script>

<template>
  <div class="space-y-8">
    <!-- Appearance -->
    <div class="space-y-4">
      <h3
        class="text-[10px] font-sans font-bold uppercase tracking-[0.2em] text-accent/80 border-b border-accent/10 pb-2"
      >
        Appearance
      </h3>

      <div class="grid grid-cols-3 gap-4">
        <button
          class="group p-4 border rounded-xl text-center transition-all relative overflow-hidden active:scale-95"
          :class="
            settings.interface.theme === 'light'
              ? 'border-accent bg-accent/10 ring-4 ring-accent/5'
              : 'border-white dark:border-white/10 bg-white/70 dark:bg-white/5 shadow-sm hover:border-accent/40'
          "
          @click="settings.interface.theme = 'light'"
        >
          <div
            class="mx-auto w-full aspect-video bg-[#fafaf6] border border-stone/50 mb-3 rounded-lg shadow-sm relative overflow-hidden transition-transform group-hover:scale-105"
          >
            <div class="absolute inset-x-0 top-0 h-3 bg-stone/20 border-b border-stone/10"></div>
          </div>
          <div
            class="text-xs font-bold tracking-tight"
            :class="settings.interface.theme === 'light' ? 'text-accent' : 'text-ink/60'"
          >
            Light
          </div>
        </button>

        <button
          class="group p-4 border rounded-xl text-center transition-all relative overflow-hidden active:scale-95"
          :class="
            settings.interface.theme === 'dark'
              ? 'border-accent bg-accent/10 ring-4 ring-accent/5'
              : 'border-white dark:border-white/10 bg-white/70 dark:bg-white/5 shadow-sm hover:border-accent/40'
          "
          @click="settings.interface.theme = 'dark'"
        >
          <div
            class="mx-auto w-full aspect-video bg-[#1a1a1a] border border-white/10 mb-3 rounded-lg shadow-sm relative overflow-hidden transition-transform group-hover:scale-105"
          >
            <div class="absolute inset-x-0 top-0 h-3 bg-white/5 border-b border-white/5"></div>
          </div>
          <div
            class="text-xs font-bold tracking-tight"
            :class="settings.interface.theme === 'dark' ? 'text-accent' : 'text-ink/60'"
          >
            Dark
          </div>
        </button>

        <button
          class="group p-4 border rounded-xl text-center transition-all relative overflow-hidden active:scale-95"
          :class="
            settings.interface.theme === 'system'
              ? 'border-accent bg-accent/10 ring-4 ring-accent/5'
              : 'border-white dark:border-white/10 bg-white/70 dark:bg-white/5 shadow-sm hover:border-accent/40'
          "
          @click="settings.interface.theme = 'system'"
        >
          <div
            class="mx-auto w-full aspect-video mb-3 rounded-lg shadow-sm relative overflow-hidden transition-transform group-hover:scale-105 border border-stone/30"
          >
            <div class="absolute inset-0 flex">
              <div class="flex-1 bg-[#fafaf6]"></div>
              <div class="flex-1 bg-[#1a1a1a]"></div>
            </div>
            <div
              class="absolute inset-x-0 top-0 h-3 flex border-b border-black/5 dark:border-white/5"
            >
              <div class="flex-1 bg-stone/20"></div>
              <div class="flex-1 bg-white/5"></div>
            </div>
          </div>
          <div
            class="text-xs font-bold tracking-tight"
            :class="settings.interface.theme === 'system' ? 'text-accent' : 'text-ink/60'"
          >
            System
          </div>
        </button>
      </div>
    </div>

    <!-- Effects -->
    <div class="space-y-4">
      <h3
        class="text-[10px] font-sans font-bold uppercase tracking-[0.2em] text-accent/80 border-b border-accent/10 pb-2"
      >
        Cyber-Glass Effects
      </h3>

      <div
        class="space-y-4 p-4 bg-white/70 dark:bg-white/5 border border-white dark:border-white/10 rounded-xl shadow-[0_2px_10px_-4px_rgba(0,0,0,0.05)]"
      >
        <div class="flex justify-between items-end">
          <label class="text-xs font-semibold text-ink/80">Effect Intensity</label>
          <span class="text-[10px] font-mono text-accent font-bold"
            >{{ settings.interface.cyberGlassIntensity }}%</span
          >
        </div>
        <input
          v-model.number="settings.interface.cyberGlassIntensity"
          type="range"
          min="0"
          max="100"
          class="w-full h-1.5 bg-ink/10 rounded-lg appearance-none cursor-pointer accent-accent"
        />
        <p class="text-[10px] text-ink/40 italic font-medium">
          Controls the blur strength and transparency of UI elements.
        </p>
      </div>
    </div>

    <!-- UI Scaling -->
    <div class="space-y-4">
      <h3
        class="text-[10px] font-sans font-bold uppercase tracking-[0.2em] text-accent/80 border-b border-accent/10 pb-2"
      >
        Interface Scaling
      </h3>

      <div
        class="space-y-4 p-4 bg-white/70 dark:bg-white/5 border border-white dark:border-white/10 rounded-xl shadow-[0_2px_10px_-4px_rgba(0,0,0,0.05)]"
      >
        <div class="flex justify-between items-end">
          <label class="text-xs font-semibold text-ink/80">UI Scale</label>
          <div class="flex items-center gap-3">
            <button
              class="text-[9px] font-bold text-accent/60 hover:text-accent uppercase tracking-tighter transition-colors"
              @click="settings.interface.uiScaling = 100"
            >
              Reset
            </button>
            <div class="relative">
              <input
                v-model.number="settings.interface.uiScaling"
                type="number"
                class="w-16 bg-white/50 dark:bg-white/5 border border-white/50 dark:border-white/10 rounded-md px-1.5 py-0.5 text-[10px] font-mono font-bold text-accent text-right focus:outline-none focus:border-accent"
              />
              <span
                class="absolute right-1.5 top-1/2 -translate-y-1/2 text-[8px] font-bold text-accent/40 pointer-events-none"
                >%</span
              >
            </div>
          </div>
        </div>
        <input
          v-model.number="settings.interface.uiScaling"
          type="range"
          min="50"
          max="300"
          step="1"
          class="w-full h-1.5 bg-ink/10 rounded-lg appearance-none cursor-pointer accent-accent"
        />
        <p class="text-[10px] text-ink/40 italic font-medium">
          Adjust the size of the sidebar, headers, and buttons.
        </p>
      </div>
    </div>
  </div>
</template>
