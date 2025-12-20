<script setup lang="ts">
import { ref } from 'vue';
import { useGamification } from '../composables/useGamification';
import GamificationDashboard from './gamification/GamificationDashboard.vue';

const { 
  dailyGoal, 
  todayWords, 
  progressDaily, 
} = useGamification();

const isOpen = ref(false);

const toggleDashboard = () => {
  isOpen.value = !isOpen.value;
};
</script>

<template>
  <div class="relative z-50">
    <!-- Trigger Pill -->
    <button 
      @click="toggleDashboard"
      class="group flex items-center gap-3 px-3 py-1.5 rounded-full bg-stone/5 hover:bg-stone/10 border border-stone/10 text-xs font-mono transition-all duration-300"
      :class="{ 'bg-stone/10 border-accent/20': isOpen }"
    >
        <!-- Mini Ring -->
        <div class="relative w-4 h-4">
           <svg class="w-full h-full -rotate-90" viewBox="0 0 36 36">
             <!-- Background Circle -->
             <path
               class="text-ink/10"
               d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
               fill="none"
               stroke="currentColor"
               stroke-width="5"
             />
             <!-- Progress Circle -->
             <path
               class="text-accent transition-all duration-500 ease-out"
               :stroke-dasharray="`${progressDaily}, 100`"
               d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
               fill="none"
               stroke="currentColor"
               stroke-width="5"
             />
           </svg>
        </div>
        
        <div class="flex flex-col items-start leading-none opacity-60 group-hover:opacity-100 transition-opacity">
            <span class="text-[9px] uppercase tracking-wider text-ink/50">Daily</span>
            <span class="font-bold">{{ todayWords }} / {{ dailyGoal }}</span>
        </div>
    </button>

    <GamificationDashboard :show="isOpen" @close="isOpen = false" />
  </div>
</template>

<style scoped>
/* Scoped styles mainly for specific overrides */
</style>
