<template>
  <div class="p-4 border-t border-stone/50 space-y-1">
    <!-- Change Project -->
    <button
      class="w-full py-2 text-xs font-medium text-ink/50 hover:text-accent transition-colors uppercase tracking-wider text-left flex items-center gap-2"
      @click="$emit('change-project')"
    >
      <svg class="w-4 h-4 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"
        />
      </svg>
      Change Project
    </button>

    <!-- Characters -->
    <button
      class="w-full py-2 text-xs font-medium text-ink/50 hover:text-accent transition-colors uppercase tracking-wider text-left flex items-center gap-2"
      @click="$emit('open-characters')"
    >
      <svg class="w-4 h-4 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
        />
      </svg>
      Characters
    </button>

    <!-- Character Graph -->
    <button
      class="w-full py-2 text-xs font-medium text-ink/50 hover:text-accent transition-colors uppercase tracking-wider text-left flex items-center gap-2"
      @click="$emit('open-character-graph')"
    >
      <svg class="w-4 h-4 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
        />
      </svg>
      Character Graph
    </button>

    <!-- Timeline -->
    <button
      class="w-full py-2 text-xs font-medium text-ink/50 hover:text-accent transition-colors uppercase tracking-wider text-left flex items-center gap-2"
      @click="$emit('open-timeline')"
    >
      <svg class="w-4 h-4 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
        />
      </svg>
      Timeline
    </button>

    <!-- Settings -->
    <button
      class="w-full py-2 text-xs font-medium text-ink/50 hover:text-accent transition-colors uppercase tracking-wider text-left flex items-center gap-2"
      @click="$emit('open-settings')"
    >
      <svg class="w-4 h-4 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z"
        />
      </svg>
      Settings
    </button>

    <!-- Research -->
    <button
      class="w-full py-2 text-xs font-medium text-ink/50 hover:text-accent transition-colors uppercase tracking-wider text-left flex items-center gap-2"
      @click="$emit('open-research')"
    >
      <svg class="w-4 h-4 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
        />
      </svg>
      Research
    </button>
  </div>
</template>

<script setup lang="ts">
defineEmits([
  'open-settings',
  'open-characters',
  'open-character-graph',
  'open-timeline',
  'change-project',
  'open-research',
]);
</script>
