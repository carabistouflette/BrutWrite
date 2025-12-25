<template>
  <button
    class="group relative w-full overflow-hidden rounded-xl border px-4 py-3 mt-8 transition-all duration-500 hover:shadow-sm active:scale-[0.98]"
    :class="
      isAdding
        ? 'border-accent/40 bg-accent/5 ring-1 ring-accent/20'
        : 'border-stone/30 bg-white/5 hover:border-stone/50 hover:bg-white/10'
    "
    @click="$emit('click')"
  >
    <!-- Subtle Shimmer -->
    <div
      class="absolute inset-0 -translate-x-[150%] skew-x-12 bg-linear-to-r from-transparent via-white/10 to-transparent transition-transform duration-700 ease-out group-hover:translate-x-[150%]"
    ></div>

    <div class="relative flex items-center justify-center gap-3">
      <!-- Icon: High contrast -->
      <div
        class="flex h-5 w-5 items-center justify-center rounded-md bg-accent/10 text-accent transition-all duration-300 group-hover:bg-accent group-hover:text-white group-hover:rotate-90"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="3"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
      </div>

      <span
        class="text-[11px] font-bold uppercase tracking-[0.2em] text-ink/60 transition-all duration-300 group-hover:text-ink group-hover:tracking-[0.25em]"
        >New Chapter</span
      >
    </div>
  </button>
</template>

<script setup lang="ts">
defineProps<{
  isAdding: boolean;
}>();

defineEmits<{
  (e: 'click'): void;
}>();
</script>
