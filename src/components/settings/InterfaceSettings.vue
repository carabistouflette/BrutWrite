<script setup lang="ts">
import { useSettings } from '../../composables/useSettings';

const { settings } = useSettings();
</script>

<template>
  <div class="space-y-8 animate-enter">
    
    <!-- Appearance -->
    <div class="space-y-4">
        <h3 class="text-sm font-sans font-bold uppercase tracking-widest text-ink/40 border-b border-ink/5 pb-2">Appearance</h3>
        
        <div class="grid grid-cols-3 gap-3">
             <button 
                @click="settings.interface.theme = 'light'"
                class="group p-3 border rounded-lg text-center transition-all hover:border-ink/20"
                :class="settings.interface.theme === 'light' ? 'border-accent bg-accent/5 ring-1 ring-accent/20' : 'border-ink/5 bg-stone'"
            >
                <div class="mx-auto w-12 h-8 bg-paper border border-stone mb-2 rounded shadow-sm relative overflow-hidden">
                    <div class="absolute top-0 left-0 w-3 h-full bg-stone/50"></div>
                </div>
                <div class="text-xs font-medium" :class="settings.interface.theme === 'light' ? 'text-accent' : 'text-ink/60'">Light</div>
            </button>

            <button 
                @click="settings.interface.theme = 'dark'"
                class="group p-3 border rounded-lg text-center transition-all hover:border-ink/20"
                :class="settings.interface.theme === 'dark' ? 'border-accent bg-accent/5 ring-1 ring-accent/20' : 'border-ink/5 bg-stone'"
            >
                <div class="mx-auto w-12 h-8 bg-ink border border-black mb-2 rounded shadow-sm relative overflow-hidden">
                    <div class="absolute top-0 left-0 w-3 h-full bg-white/10"></div>
                </div>
                <div class="text-xs font-medium" :class="settings.interface.theme === 'dark' ? 'text-accent' : 'text-ink/60'">Dark</div>
            </button>

            <button 
                @click="settings.interface.theme = 'system'"
                class="group p-3 border rounded-lg text-center transition-all hover:border-ink/20"
                :class="settings.interface.theme === 'system' ? 'border-accent bg-accent/5 ring-1 ring-accent/20' : 'border-ink/5 bg-stone'"
            >
                <div class="mx-auto w-12 h-8 bg-linear-to-br from-paper to-ink border border-ink/10 mb-2 rounded shadow-sm"></div>
                <div class="text-xs font-medium" :class="settings.interface.theme === 'system' ? 'text-accent' : 'text-ink/60'">System</div>
            </button>
        </div>
    </div>

    <!-- Effects -->
    <div class="space-y-4">
        <h3 class="text-sm font-sans font-bold uppercase tracking-widest text-ink/40 border-b border-ink/5 pb-2">Cyber-Glass Effects</h3>
        
        <div class="space-y-3">
             <div class="flex justify-between">
                <label class="text-xs font-medium text-ink/70">Intensity</label>
                <span class="text-xs font-mono text-ink/40">{{ settings.interface.cyberGlassIntensity }}%</span>
             </div>
             <input 
                v-model.number="settings.interface.cyberGlassIntensity"
                type="range" 
                min="0" 
                max="100" 
                class="w-full accent-accent"
            />
            <p class="text-[10px] text-ink/40 italic">Controls the blur strength and transparency of UI elements.</p>
        </div>
    </div>

  </div>
</template>
