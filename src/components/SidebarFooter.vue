<template>
  <div class="p-4 border-t border-stone/50">
    <!-- Settings -->
    <button 
      @click="$emit('open-settings')"
      class="w-full py-2 text-xs font-medium text-ink/50 hover:text-accent transition-colors uppercase tracking-wider text-left"
    >
      Settings
    </button>
  </div>
</template>

<script setup lang="ts">
const emit = defineEmits(['open-settings']);
</script>
