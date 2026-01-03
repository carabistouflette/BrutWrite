<script setup lang="ts">
import type { GraphMetrics, GraphAlert } from '../../../types/intelligence';

interface Props {
  metrics: GraphMetrics;
  alerts: GraphAlert[];
}

const props = defineProps<Props>();
</script>

<template>
  <footer
    class="absolute bottom-0 left-0 right-0 flex justify-between items-center p-3 px-6 bg-linear-to-t from-white to-transparent border-t border-black/5"
  >
    <div class="flex items-center gap-4">
      <div class="flex flex-col gap-0.5">
        <span class="text-[10px] font-bold uppercase tracking-widest text-black/40">Density</span>
        <span class="text-sm font-semibold font-serif text-gray-900"
          >{{ (props.metrics.networkDensity * 100).toFixed(0) }}%</span
        >
      </div>
      <div class="w-px h-6 bg-black/8"></div>
      <div class="flex flex-col gap-0.5">
        <span class="text-[10px] font-bold uppercase tracking-widest text-black/40"
          >Components</span
        >
        <span class="text-sm font-semibold font-serif text-gray-900">{{
          props.metrics.connectedComponents
        }}</span>
      </div>
      <div class="w-px h-6 bg-black/8"></div>
      <div class="flex flex-col gap-0.5">
        <span class="text-[10px] font-bold uppercase tracking-widest text-black/40">Isolated</span>
        <span class="text-sm font-semibold font-serif text-gray-900"
          >{{ (props.metrics.isolationRatio * 100).toFixed(0) }}%</span
        >
      </div>
    </div>

    <!-- Alert Badges -->
    <div v-if="props.alerts.length > 0" class="flex gap-2">
      <span
        v-for="alert in props.alerts"
        :key="alert.code"
        class="p-1.5 px-3 text-[10px] font-bold uppercase tracking-wide text-white bg-orange-500 rounded-lg cursor-help transition-all hover:-translate-y-px hover:shadow-lg hover:shadow-orange-500/30"
        :title="alert.tooltip"
      >
        {{ alert.primaryText }}
      </span>
    </div>
  </footer>
</template>
