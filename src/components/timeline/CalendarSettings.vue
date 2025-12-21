<script setup lang="ts">
import { ref } from 'vue';
import { useCalendar, type CalendarSystem, type MonthConfig } from '../../composables/useCalendar';

const emit = defineEmits(['close']);

const { 
    currentSystem, 
    startYear, 
    customMonths,
    setSystem, 
    setStartYear,
    setCustomMonths
} = useCalendar();

const activeTab = ref<CalendarSystem>(currentSystem.value);
const localStartYear = ref(startYear.value);
// Deep copy for editing
const localCustomMonths = ref<MonthConfig[]>(JSON.parse(JSON.stringify(customMonths.value)));

function addMonth() {
    localCustomMonths.value.push({ name: 'New Month', days: 30 });
}

function removeMonth(index: number) {
    localCustomMonths.value.splice(index, 1);
}

function apply() {
    setSystem(activeTab.value);
    setStartYear(localStartYear.value);
    if (activeTab.value === 'custom') {
        setCustomMonths(localCustomMonths.value);
    }
    emit('close');
}

</script>

<template>
    <div class="calendar-settings-overlay" @click.self="$emit('close')">
        <div class="calendar-settings-modal">
            <header>
                <h2>Calendar Settings</h2>
                <button class="close-btn" @click="$emit('close')">&times;</button>
            </header>

            <div class="modal-body">
                <div class="form-group">
                    <label>System</label>
                    <div class="tabs">
                        <button 
                            :class="{ active: activeTab === 'gregorian' }"
                            @click="activeTab = 'gregorian'"
                        >Gregorian</button>
                        <button 
                            :class="{ active: activeTab === 'fixed360' }"
                            @click="activeTab = 'fixed360'"
                        >Fixed 360</button>
                        <button 
                            :class="{ active: activeTab === 'custom' }"
                            @click="activeTab = 'custom'"
                        >Custom</button>
                    </div>
                </div>

                <div class="form-group" v-if="activeTab !== 'gregorian'">
                    <label>Start Year (Epoch)</label>
                    <div class="help-text">What year is 1970-01-01 in your world?</div>
                    <input type="number" v-model="localStartYear" />
                </div>

                <div class="desc" v-if="activeTab === 'fixed360'">
                    <p><strong>Fixed 360</strong> uses 12 months of exactly 30 days each.</p>
                    <p>Ideal for simple fantasy timelines where you don't want to track leap years or irregular months.</p>
                </div>

                <!-- Custom Editor -->
                <div class="custom-editor" v-if="activeTab === 'custom'">
                    <label>Months Configuration</label>
                    <div class="months-list">
                        <div 
                            v-for="(month, idx) in localCustomMonths" 
                            :key="idx"
                            class="month-row"
                        >
                            <input type="text" v-model="month.name" placeholder="Name" class="month-name" />
                            <input type="number" v-model="month.days" placeholder="Days" class="month-days" />
                            <button class="remove-btn" @click="removeMonth(idx)" title="Remove">&times;</button>
                        </div>
                    </div>
                    <button class="add-btn" @click="addMonth">+ Add Month</button>
                </div>
            </div>

            <footer>
                <button class="btn-primary" @click="apply">Apply Changes</button>
            </footer>
        </div>
    </div>
</template>

<style scoped>
.calendar-settings-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 2000;
    backdrop-filter: blur(4px);
}

.calendar-settings-modal {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    width: 600px;
    max-height: 90vh;
    box-shadow: 0 10px 25px rgba(0,0,0,0.2);
    display: flex;
    flex-direction: column;
}

header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid var(--border-color);
}

h2 {
    margin: 0;
    font-size: 1.25rem;
}

.modal-body {
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    overflow-y: auto;
}

.form-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
}

label {
    font-weight: 600;
    font-size: 0.9rem;
}

.help-text {
    font-size: 0.8rem;
    color: var(--text-tertiary);
    margin-top: -0.25rem;
}

.tabs {
    display: flex;
    gap: 0.5rem;
    background: var(--bg-tertiary);
    padding: 4px;
    border-radius: 6px;
}

.tabs button {
    flex: 1;
    background: transparent;
    border: none;
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
    color: var(--text-secondary);
    transition: all 0.2s;
}

.tabs button.active {
    background: var(--bg-primary);
    color: var(--text-primary);
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    font-weight: 600; /* Bolder active tab */
}

input {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    padding: 8px 12px;
    border-radius: 4px;
    color: var(--text-primary);
}

.desc {
    font-size: 0.875rem;
    color: var(--text-secondary);
    background: var(--bg-tertiary);
    padding: 1rem;
    border-radius: 4px;
}

.desc p { margin: 0 0 0.5rem 0; }
.desc p:last-child { margin: 0; }

/* Custom Editor Styles */
.custom-editor {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    border-top: 1px solid var(--border-color);
    padding-top: 1rem;
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

.month-name { flex: 2; }
.month-days { flex: 1; }

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

footer {
    padding: 1rem;
    border-top: 1px solid var(--border-color);
    display: flex;
    justify-content: flex-end;
}

.btn-primary {
    /* High Contrast Brutalist Button */
    background: var(--color-ink); /* Black (or Ink) background */
    color: var(--color-paper);   /* White (or Paper) text */
    border: none;
    padding: 10px 20px;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 600;
    font-size: 0.9rem;
    transition: transform 0.1s;
}

.btn-primary:hover {
    transform: translateY(-1px);
    box-shadow: 0 2px 5px rgba(0,0,0,0.2);
}

.close-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    color: var(--text-tertiary);
}
</style>
