<script setup lang="ts">
import { useAppStatus } from '../../composables/ui/useAppStatus';

const { notifications, removeNotification } = useAppStatus();
</script>

<template>
  <div class="fixed top-6 right-6 z-100 flex flex-col gap-3 pointer-events-none max-w-sm w-full">
    <TransitionGroup
      enter-active-class="transition duration-300 ease-out"
      enter-from-class="translate-x-12 opacity-0"
      enter-to-class="translate-x-0 opacity-100"
      leave-active-class="transition duration-200 ease-in"
      leave-from-class="translate-x-0 opacity-100"
      leave-to-class="translate-x-12 opacity-0"
    >
      <div 
        v-for="note in notifications" 
        :key="note.id"
        class="pointer-events-auto p-4 rounded-2xl shadow-2xl border flex items-center justify-between gap-4 glass-notification"
        :class="[
          note.type === 'error' ? 'bg-red-500/10 border-red-500/20 text-red-700' : 
          note.type === 'success' ? 'bg-green-500/10 border-green-500/20 text-green-700' : 
          'bg-white/80 border-ink/5 text-ink'
        ]"
      >
        <div class="flex items-center gap-3">
            <!-- Icon -->
            <div v-if="note.type === 'error'" class="w-2 h-2 rounded-full bg-red-500 animate-pulse"></div>
            <div v-else-if="note.type === 'success'" class="w-2 h-2 rounded-full bg-green-500"></div>
            <div v-else class="w-2 h-2 rounded-full bg-ink/20"></div>
            
            <p class="text-sm font-medium tracking-tight">{{ note.message }}</p>
        </div>

        <button 
          @click="removeNotification(note.id)"
          class="p-1 hover:bg-black/5 rounded-full transition-colors opacity-40 hover:opacity-100"
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.glass-notification {
    backdrop-filter: blur(12px) saturate(180%);
    -webkit-backdrop-filter: blur(12px) saturate(180%);
}
</style>
