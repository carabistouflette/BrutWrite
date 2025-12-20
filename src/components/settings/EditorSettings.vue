<script setup lang="ts">
import { useSettings } from '../../composables/useSettings';

const { settings } = useSettings();
</script>

<template>
  <div class="space-y-8">
    
    <!-- Typography -->
    <div class="space-y-4">
        <h3 class="text-[10px] font-sans font-bold uppercase tracking-[0.2em] text-accent/80 border-b border-accent/10 pb-2">Typography</h3>
        
        <!-- Font Family -->
        <div class="space-y-2">
            <label class="block text-xs font-semibold text-ink/80">Font Family</label>
            <div class="grid grid-cols-3 gap-2">
                <button 
                    @click="settings.editor.fontFamily = 'serif'"
                    class="px-4 py-4 border rounded-xl text-center transition-all group relative overflow-hidden active:scale-95"
                    :class="settings.editor.fontFamily === 'serif' ? 'bg-accent/10 border-accent shadow-[0_0_15px_rgba(255,95,31,0.1)]' : 'bg-white/70 dark:bg-white/5 border-white dark:border-white/10 hover:border-accent/40 shadow-sm'"
                >
                    <span class="font-serif text-xl relative z-10" :class="settings.editor.fontFamily === 'serif' ? 'text-accent' : 'text-ink/70'">Ag</span>
                    <div class="text-[9px] font-bold mt-1 uppercase tracking-widest relative z-10" :class="settings.editor.fontFamily === 'serif' ? 'text-accent/60' : 'text-ink/30'">Serif</div>
                </button>
                <button 
                    @click="settings.editor.fontFamily = 'sans'"
                    class="px-4 py-4 border rounded-xl text-center transition-all group relative overflow-hidden active:scale-95"
                    :class="settings.editor.fontFamily === 'sans' ? 'bg-accent/10 border-accent shadow-[0_0_15px_rgba(255,95,31,0.1)]' : 'bg-white/70 dark:bg-white/5 border-white dark:border-white/10 hover:border-accent/40 shadow-sm'"
                >
                    <span class="font-sans text-xl relative z-10" :class="settings.editor.fontFamily === 'sans' ? 'text-accent' : 'text-ink/70'">Ag</span>
                    <div class="text-[9px] font-bold mt-1 uppercase tracking-widest relative z-10" :class="settings.editor.fontFamily === 'sans' ? 'text-accent/60' : 'text-ink/30'">Sans</div>
                </button>
                <button 
                    @click="settings.editor.fontFamily = 'mono'"
                    class="px-4 py-4 border rounded-xl text-center transition-all group relative overflow-hidden active:scale-95"
                    :class="settings.editor.fontFamily === 'mono' ? 'bg-accent/10 border-accent shadow-[0_0_15px_rgba(255,95,31,0.1)]' : 'bg-white/70 dark:bg-white/5 border-white dark:border-white/10 hover:border-accent/40 shadow-sm'"
                >
                    <span class="font-mono text-xl relative z-10" :class="settings.editor.fontFamily === 'mono' ? 'text-accent' : 'text-ink/70'">Ag</span>
                    <div class="text-[9px] font-bold mt-1 uppercase tracking-widest relative z-10" :class="settings.editor.fontFamily === 'mono' ? 'text-accent/60' : 'text-ink/30'">Mono</div>
                </button>
            </div>
        </div>

        <!-- Font Size & Line Height -->
        <div class="grid grid-cols-2 gap-6">
            <div class="space-y-3">
                <div class="flex justify-between items-end">
                    <label class="text-xs font-semibold text-ink/80">Font Size</label>
                    <span class="text-[10px] font-mono text-accent font-bold">{{ settings.editor.fontSize }}px</span>
                </div>
                <input 
                    v-model.number="settings.editor.fontSize"
                    type="range" 
                    min="12" 
                    max="32" 
                    class="w-full h-1.5 bg-ink/10 rounded-lg appearance-none cursor-pointer accent-accent"
                />
            </div>
            <div class="space-y-3">
                <div class="flex justify-between items-end">
                    <label class="text-xs font-semibold text-ink/80">Line Height</label>
                    <span class="text-[10px] font-mono text-accent font-bold">{{ settings.editor.lineHeight }}</span>
                </div>
                <input 
                    v-model.number="settings.editor.lineHeight"
                    type="range" 
                    min="1" 
                    max="2.5" 
                    step="0.1"
                    class="w-full h-1.5 bg-ink/10 rounded-lg appearance-none cursor-pointer accent-accent"
                />
            </div>
        </div>
    </div>

    <!-- Layout & Behavior -->
    <div class="space-y-4">
        <h3 class="text-[10px] font-sans font-bold uppercase tracking-[0.2em] text-accent/80 border-b border-accent/10 pb-2">Layout & Focus</h3>
        
        <div class="space-y-2">
            <label class="block text-xs font-semibold text-ink/80">Max Content Width</label>
            <div class="flex items-center gap-3">
                <div class="relative flex-1 max-w-[160px]">
                    <input 
                        v-model.number="settings.editor.maxWidth"
                        type="number"
                        step="50"
                        min="400"
                        class="w-full bg-white/70 dark:bg-white/5 border border-white dark:border-white/10 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-accent focus:ring-4 focus:ring-accent/10 transition-all shadow-[0_2px_10px_-4px_rgba(0,0,0,0.05)] input-with-unit"
                    />
                    <div class="absolute right-3 top-1/2 -translate-y-1/2 text-[10px] font-bold text-ink/30 uppercase tracking-tighter">px</div>
                </div>
            </div>
        </div>

        <div class="flex items-center justify-between p-4 bg-white/70 dark:bg-white/5 border border-white dark:border-white/10 rounded-xl shadow-[0_2px_10px_-4px_rgba(0,0,0,0.05)]">
            <div class="space-y-0.5">
                <div class="text-sm font-semibold text-ink/80">Focus Mode</div>
                <div class="text-[10px] text-ink/40 font-medium">Highlight only the active paragraph</div>
            </div>
            <button 
                @click="settings.editor.focusMode = !settings.editor.focusMode"
                class="w-11 h-6 rounded-full relative transition-all duration-300"
                :class="settings.editor.focusMode ? 'bg-accent shadow-[0_0_10px_rgba(255,95,31,0.4)]' : 'bg-white/80 dark:bg-white/10'"
            >
                <div 
                    class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform duration-300 shadow-sm"
                    :class="settings.editor.focusMode ? 'translate-x-5' : 'translate-x-0'"
                ></div>
            </button>
        </div>
    </div>

  </div>
</template>
