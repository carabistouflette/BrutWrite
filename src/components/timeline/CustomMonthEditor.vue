<script setup lang="ts">
import type { MonthConfig } from '../../composables/timeline/useCalendar';

const props = defineProps<{
  modelValue: MonthConfig[];
}>();

const emit = defineEmits(['update:modelValue']);

function addMonth() {
  const newMonths = [...props.modelValue, { name: 'New Month', days: 30 }];
  emit('update:modelValue', newMonths);
}

function removeMonth(index: number) {
  const newMonths = [...props.modelValue];
  newMonths.splice(index, 1);
  emit('update:modelValue', newMonths);
}

function updateMonth(index: number, key: keyof MonthConfig, value: string | number) {
  const newMonths = [...props.modelValue];
  newMonths[index] = { ...newMonths[index], [key]: value };
  emit('update:modelValue', newMonths);
}
</script>

<template>
  <div class="custom-editor">
    <label>Months Configuration</label>
    <div class="months-list">
      <div v-for="(month, idx) in modelValue" :key="idx" class="month-row">
        <input
          type="text"
          :value="month.name"
          @input="(e) => updateMonth(idx, 'name', (e.target as HTMLInputElement).value)"
          placeholder="Name"
          class="month-name"
        />
        <input
          type="number"
          :value="month.days"
          @input="(e) => updateMonth(idx, 'days', parseInt((e.target as HTMLInputElement).value))"
          placeholder="Days"
          class="month-days"
        />
        <button class="remove-btn" @click="removeMonth(idx)" title="Remove">&times;</button>
      </div>
    </div>
    <button class="add-btn" @click="addMonth">+ Add Month</button>
  </div>
</template>

<style scoped>
.custom-editor {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  border-top: 1px solid var(--border-color);
  padding-top: 1rem;
}

label {
  font-weight: 600;
  font-size: 0.9rem;
}

.months-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  max-height: 350px;
  overflow-y: auto;
  padding-right: 4px;
}

.month-row {
  display: flex;
  gap: 0.5rem;
}

.month-name {
  flex: 2;
}
.month-days {
  flex: 1;
}

input {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  padding: 8px 12px;
  border-radius: 4px;
  color: var(--text-primary);
}

.remove-btn {
  background: transparent;
  border: none;
  color: var(--color-danger);
  font-size: 1.25rem;
  cursor: pointer;
  line-height: 1;
  padding: 0 4px;
}

.add-btn {
  align-self: flex-start;
  background: transparent;
  border: 1px dashed var(--border-color);
  color: var(--text-secondary);
  padding: 4px 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.8rem;
  margin-top: 0.5rem;
}

.add-btn:hover {
  border-color: var(--text-primary);
  color: var(--text-primary);
}
</style>
