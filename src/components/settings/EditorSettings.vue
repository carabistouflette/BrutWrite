<script setup lang="ts">
import { useSettings } from '../../composables/useSettings';

const { settings } = useSettings();
</script>

<template>
  <div class="space-y-8 animate-enter">
    
    <!-- Typography -->
    <div class="space-y-4">
        <h3 class="text-sm font-sans font-bold uppercase tracking-widest text-ink/40 border-b border-ink/5 pb-2">Typography</h3>
        
        <!-- Font Family -->
        <div class="space-y-2">
            <label class="block text-xs font-medium text-ink/70">Font Family</label>
            <div class="grid grid-cols-3 gap-2">
                <button 
                    @click="settings.editor.fontFamily = 'serif'"
                    class="px-4 py-3 border rounded-lg text-center transition-all"
                    :class="settings.editor.fontFamily === 'serif' ? 'bg-accent/10 border-accent text-accent' : 'bg-stone border-ink/5 hover:border-ink/20'"
                >
                    <span class="font-serif text-lg">Ag</span>
                    <div class="text-[10px] opacity-60 mt-1 uppercase tracking-wider font-sans">Serif</div>
                </button>
                <button 
                    @click="settings.editor.fontFamily = 'sans'"
                    class="px-4 py-3 border rounded-lg text-center transition-all"
                    :class="settings.editor.fontFamily === 'sans' ? 'bg-accent/10 border-accent text-accent' : 'bg-stone border-ink/5 hover:border-ink/20'"
                >
                    <span class="font-sans text-lg">Ag</span>
                    <div class="text-[10px] opacity-60 mt-1 uppercase tracking-wider font-sans">Sans</div>
                </button>
                <button 
                    @click="settings.editor.fontFamily = 'mono'"
                    class="px-4 py-3 border rounded-lg text-center transition-all"
                    :class="settings.editor.fontFamily === 'mono' ? 'bg-accent/10 border-accent text-accent' : 'bg-stone border-ink/5 hover:border-ink/20'"
                >
                    <span class="font-mono text-lg">Ag</span>
                    <div class="text-[10px] opacity-60 mt-1 uppercase tracking-wider font-sans">Mono</div>
                </button>
            </div>
        </div>

        <!-- Font Size & Line Height -->
        <div class="grid grid-cols-2 gap-4">
            <div class="space-y-2">
                <label class="block text-xs font-medium text-ink/70">Font Size: <span class="text-ink">{{ settings.editor.fontSize }}px</span></label>
                <input 
                    v-model.number="settings.editor.fontSize"
                    type="range" 
                    min="12" 
                    max="32" 
                    class="w-full accent-accent"
                />
            </div>
            <div class="space-y-2">
                <label class="block text-xs font-medium text-ink/70">Line Height: <span class="text-ink">{{ settings.editor.lineHeight }}</span></label>
                <input 
                    v-model.number="settings.editor.lineHeight"
                    type="range" 
                    min="1" 
                    max="2.5" 
                    step="0.1"
                    class="w-full accent-accent"
                />
            </div>
        </div>
    </div>

    <!-- Layout & Behavior -->
    <div class="space-y-4">
        <h3 class="text-sm font-sans font-bold uppercase tracking-widest text-ink/40 border-b border-ink/5 pb-2">Layout & Focus</h3>
        
        <div class="space-y-2">
            <label class="block text-xs font-medium text-ink/70">Max Content Width</label>
            <div class="flex items-center gap-3">
                <input 
                    v-model.number="settings.editor.maxWidth"
                    type="number"
                    step="50"
                    min="400"
                    class="w-32 bg-stone border border-ink/5 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-accent/50 focus:ring-2 focus:ring-accent/10 transition-all"
                />
                <span class="text-sm text-ink/40">px</span>
            </div>
        </div>

        <div class="flex items-center justify-between p-3 bg-stone border border-ink/5 rounded-lg">
            <div class="space-y-0.5">
                <div class="text-sm font-medium">Focus Mode</div>
                <div class="text-[10px] text-ink/40">Highlight only the active paragraph</div>
            </div>
            <button 
                @click="settings.editor.focusMode = !settings.editor.focusMode"
                class="w-10 h-5 rounded-full relative transition-colors duration-200"
                :class="settings.editor.focusMode ? 'bg-accent' : 'bg-ink/20'"
            >
                <div 
                    class="absolute top-1 left-1 w-3 h-3 bg-white rounded-full transition-transform duration-200 shadow-sm"
                    :class="settings.editor.focusMode ? 'translate-x-5' : 'translate-x-0'"
                ></div>
            </button>
        </div>
    </div>

  </div>
</template>
