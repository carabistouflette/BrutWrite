<script setup lang="ts">
import { ref } from 'vue';
import {
  useCalendar,
  type CalendarSystem,
  type MonthConfig,
} from '../../composables/timeline/useCalendar';
import CustomMonthEditor from './CustomMonthEditor.vue';

const emit = defineEmits(['close']);

const { currentSystem, startYear, customMonths, setSystem, setStartYear, setCustomMonths } =
  useCalendar();

const activeTab = ref<CalendarSystem>(currentSystem.value);
const localStartYear = ref(startYear.value);
// Deep copy for editing
const localCustomMonths = ref<MonthConfig[]>(JSON.parse(JSON.stringify(customMonths.value)));

function apply() {
  setSystem(activeTab.value);
  setStartYear(localStartYear.value);
  if (activeTab.value === 'custom') {
    setCustomMonths(localCustomMonths.value);
  }
  emit('close');
}

import './CalendarSettings.css';
</script>

<template>
  <div class="calendar-settings-overlay" @click.self="emit('close')">
    <div class="calendar-settings-modal">
      <header>
        <h2>Calendar Settings</h2>
        <button class="close-btn" @click="emit('close')">&times;</button>
      </header>

      <div class="modal-body">
        <div class="form-group">
          <label>System</label>
          <div class="tabs">
            <button :class="{ active: activeTab === 'gregorian' }" @click="activeTab = 'gregorian'">
              Gregorian
            </button>
            <button :class="{ active: activeTab === 'fixed360' }" @click="activeTab = 'fixed360'">
              Fixed 360
            </button>
            <button :class="{ active: activeTab === 'custom' }" @click="activeTab = 'custom'">
              Custom
            </button>
          </div>
        </div>

        <div class="form-group" v-if="activeTab !== 'gregorian'">
          <label>Start Year (Epoch)</label>
          <div class="help-text">What year is 1970-01-01 in your world?</div>
          <input type="number" v-model="localStartYear" />
        </div>

        <div class="desc" v-if="activeTab === 'fixed360'">
          <p><strong>Fixed 360</strong> uses 12 months of exactly 30 days each.</p>
          <p>
            Ideal for simple fantasy timelines where you don't want to track leap years or irregular
            months.
          </p>
        </div>

        <!-- Custom Editor -->
        <CustomMonthEditor v-if="activeTab === 'custom'" v-model="localCustomMonths" />
      </div>

      <footer>
        <button class="btn-primary" @click="apply">Apply Changes</button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
/* Styles moved to CalendarSettings.css */
</style>
