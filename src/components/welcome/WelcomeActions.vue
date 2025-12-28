<script setup lang="ts">
import BaseButton from '../base/BaseButton.vue';

const emit = defineEmits<{
  (e: 'new-project'): void;
  (e: 'open-project'): void;
  (e: 'demo-project'): void;
}>();
</script>

<template>
  <div class="w-full max-w-sm space-y-4">
    <!-- Primary Action: New Project -->
    <BaseButton variant="primary" size="lg" block class="group" @click="emit('new-project')">
      <template #icon>
        <svg
          class="w-5 h-5 opacity-60 group-hover:rotate-90 transition-transform duration-500"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 4v16m8-8H4"
          />
        </svg>
      </template>
      Create New Project

      <!-- Decorative hover effect overlay -->
      <div
        class="absolute inset-0 bg-accent translate-y-full group-hover:translate-y-0 transition-transform duration-500 opacity-10 pointer-events-none"
      ></div>
    </BaseButton>

    <!-- Secondary Action: Open Project -->
    <BaseButton variant="secondary" size="lg" block class="group" @click="emit('open-project')">
      <template #icon>
        <svg
          class="w-5 h-5 opacity-40 group-hover:text-accent group-hover:opacity-100 transition-all"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z"
          />
        </svg>
      </template>
      Open Existing Project
    </BaseButton>

    <!-- Detailed Action: Demo Project -->
    <BaseButton
      variant="ghost"
      size="sm"
      block
      class="group pt-4 opacity-60 hover:opacity-100"
      @click="emit('demo-project')"
    >
      Create Example Project (Demo)
    </BaseButton>
  </div>
</template>
