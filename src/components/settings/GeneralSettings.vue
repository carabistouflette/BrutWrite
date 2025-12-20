<script setup lang="ts">
import { useSettings } from '../../composables/useSettings';

const { settings } = useSettings();
</script>

<template>
  <div class="space-y-8 animate-enter">
    <!-- Author Identity -->
    <div class="space-y-4">
        <h3 class="text-sm font-sans font-bold uppercase tracking-widest text-ink/40 border-b border-ink/5 pb-2">Identity</h3>
        <div class="space-y-2">
            <label class="block text-xs font-medium text-ink/70">Author Name</label>
            <input 
            v-model="settings.general.authorName"
            type="text" 
            placeholder="e.g. Ernest Hemingway"
            class="w-full bg-stone border border-ink/5 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-accent/50 focus:ring-2 focus:ring-accent/10 transition-all placeholder:text-ink/20"
            />
            <p class="text-[10px] text-ink/40">Used for metadata when exporting your book.</p>
        </div>
    </div>

    <!-- Goals -->
    <div class="space-y-4">
        <h3 class="text-sm font-sans font-bold uppercase tracking-widest text-ink/40 border-b border-ink/5 pb-2">Goals</h3>
        <div class="space-y-2">
            <label class="block text-xs font-medium text-ink/70">Daily Word Goal</label>
            <div class="flex items-center gap-3">
                <input 
                    v-model.number="settings.general.dailyGoal"
                    type="number" 
                    class="w-32 bg-stone border border-ink/5 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-accent/50 focus:ring-2 focus:ring-accent/10 transition-all"
                />
                <span class="text-sm text-ink/40">words</span>
            </div>
        </div>
    </div>

    <!-- Storage -->
    <div class="space-y-4">
        <h3 class="text-sm font-sans font-bold uppercase tracking-widest text-ink/40 border-b border-ink/5 pb-2">Data</h3>
        <div class="space-y-2">
            <label class="block text-xs font-medium text-ink/70">Auto-Save Interval</label>
            <div class="flex items-center gap-3">
                <input 
                    v-model.number="settings.general.autoSaveInterval"
                    type="number"
                    min="5" 
                    class="w-32 bg-stone border border-ink/5 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-accent/50 focus:ring-2 focus:ring-accent/10 transition-all"
                />
                <span class="text-sm text-ink/40">seconds</span>
            </div>
        </div>
    </div>
  </div>
</template>
