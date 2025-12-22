<script setup lang="ts">
import { ref, computed } from 'vue';
import { useGamification } from '../../composables/logic/useGamification';
import { useSettings } from '../../composables/logic/useSettings';

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const {
  dailyGoal,
  projectTarget,
  todayWords,
  totalProjectWords,
  progressDaily,
  progressProject,
  history,
  streak,
  bestDay,
  setProjectTarget,
} = useGamification();

const { settings } = useSettings();

const isEditing = ref(false);
const tempDailyGoal = ref(dailyGoal.value);
const tempProjectTarget = ref(projectTarget.value);

const saveGoals = () => {
  settings.value.general.dailyGoal = Number(tempDailyGoal.value);
  setProjectTarget(Number(tempProjectTarget.value));
  isEditing.value = false;
};

// History Chart Data (Last 7 days)
const chartData = computed(() => {
  const days = [];
  const today = new Date();
  for (let i = 6; i >= 0; i--) {
    const d = new Date(today);
    d.setDate(d.getDate() - i);
    const dateStr = d.toISOString().split('T')[0];
    const entry = history.value.find((h) => h.date === dateStr);
    days.push({
      day: d.toLocaleDateString('en-US', { weekday: 'narrow' }),
      count: entry ? entry.wordCount : 0,
      date: dateStr,
      perm: (entry?.wordCount || 0) / (bestDay.value || 1), // normalized height
    });
  }
  return days;
});
</script>

<template>
  <Teleport to="#app-scale-root">
    <Transition
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="opacity-0 translate-y-2 scale-95"
      enter-to-class="opacity-100 translate-y-0 scale-100"
      leave-active-class="transition duration-150 ease-in"
      leave-from-class="opacity-100 translate-y-0 scale-100"
      leave-to-class="opacity-0 translate-y-2 scale-95"
    >
      <div
        v-if="show"
        class="fixed top-20 right-8 w-80 bg-paper/90 backdrop-blur-xl border border-stone/20 shadow-2xl rounded-2xl p-6 z-100 text-ink"
      >
        <!-- Header -->
        <div class="flex justify-between items-center mb-6">
          <h3 class="font-serif italic font-bold text-lg">Goal Tracking</h3>
          <button @click="emit('close')" class="text-ink/40 hover:text-ink">
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>

        <!-- Stats Grid -->
        <div class="space-y-6">
          <!-- Daily Goal -->
          <div class="space-y-2">
            <div
              class="flex justify-between text-xs uppercase tracking-widest font-bold text-ink/50"
            >
              <span>Daily Goal</span>
              <span v-if="!isEditing">{{ Math.round(progressDaily) }}%</span>
            </div>
            <div v-if="isEditing" class="flex gap-2">
              <input
                v-model="tempDailyGoal"
                type="number"
                class="w-full bg-stone/10 rounded px-2 py-1 text-sm border-none focus:ring-1 focus:ring-accent"
              />
            </div>
            <div v-else class="relative h-2 bg-stone/10 rounded-full overflow-hidden">
              <div
                class="absolute top-0 left-0 h-full bg-accent transition-all duration-700 ease-out"
                :style="{ width: `${progressDaily}%` }"
              ></div>
            </div>
            <div v-if="!isEditing" class="text-xs font-mono text-ink/70 text-right">
              {{ todayWords }} / {{ dailyGoal }} words
            </div>
          </div>

          <!-- Project Goal -->
          <div class="space-y-2">
            <div
              class="flex justify-between text-xs uppercase tracking-widest font-bold text-ink/50"
            >
              <span>Project Target</span>
              <span v-if="!isEditing">{{ Math.round(progressProject) }}%</span>
            </div>
            <div v-if="isEditing" class="flex gap-2">
              <input
                v-model="tempProjectTarget"
                type="number"
                class="w-full bg-stone/10 rounded px-2 py-1 text-sm border-none focus:ring-1 focus:ring-accent"
              />
            </div>
            <div v-else class="relative h-2 bg-stone/10 rounded-full overflow-hidden">
              <div
                class="absolute top-0 left-0 h-full bg-ink transition-all duration-700 ease-out"
                :style="{ width: `${progressProject}%` }"
              ></div>
            </div>
            <div v-if="!isEditing" class="text-xs font-mono text-ink/70 text-right">
              {{ totalProjectWords }} / {{ projectTarget }} words
            </div>
          </div>

          <!-- History Chart -->
          <div class="pt-4 border-t border-stone/10">
            <div
              class="flex justify-between text-xs uppercase tracking-widest font-bold text-ink/50 mb-3"
            >
              <span>Last 7 Days</span>
            </div>
            <div class="flex items-end justify-between h-24 gap-1">
              <div
                v-for="day in chartData"
                :key="day.date"
                class="flex-1 flex flex-col items-center gap-1 group/bar relative"
              >
                <div
                  class="absolute bottom-full mb-1 bg-ink text-white text-[10px] px-2 py-1 rounded opacity-0 group-hover/bar:opacity-100 transition-opacity whitespace-nowrap z-10 pointer-events-none"
                >
                  {{ day.count }} words
                </div>
                <div
                  class="w-full bg-accent/20 rounded-t-sm transition-all duration-500 hover:bg-accent relative"
                  :style="{ height: `${Math.max(4, day.perm * 100)}%` }"
                >
                  <div
                    class="absolute bottom-0 w-full bg-accent/40"
                    :style="{ height: day.count > 0 ? '4px' : '0' }"
                  ></div>
                </div>
                <span class="text-[9px] text-ink/40 font-mono">{{ day.day }}</span>
              </div>
            </div>
          </div>

          <!-- Meta Stats -->
          <div class="grid grid-cols-2 gap-4 py-4 border-t border-stone/10">
            <div class="bg-stone/5 p-3 rounded-lg text-center">
              <div class="text-2xl font-bold font-serif italic">{{ streak }}</div>
              <div class="text-[9px] uppercase tracking-widest text-ink/40 mt-1">Day Streak</div>
            </div>
            <div class="bg-stone/5 p-3 rounded-lg text-center">
              <div class="text-2xl font-bold font-serif italic">{{ bestDay }}</div>
              <div class="text-[9px] uppercase tracking-widest text-ink/40 mt-1">Best Day</div>
            </div>
          </div>

          <!-- Edit Button -->
          <div class="flex justify-end">
            <button
              v-if="!isEditing"
              @click="isEditing = true"
              class="text-xs text-ink/40 hover:text-accent underline decoration-dotted underline-offset-4"
            >
              Adjust Goals
            </button>
            <button
              v-else
              @click="saveGoals"
              class="text-xs bg-ink text-white px-3 py-1 rounded hover:bg-accent transition-colors"
            >
              Save Changes
            </button>
          </div>
        </div>
      </div>
    </Transition>
    <!-- Backdrop -->
    <div v-if="show" @click="emit('close')" class="fixed inset-0 z-90"></div>
  </Teleport>
</template>
