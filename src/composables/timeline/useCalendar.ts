import { ref, computed } from 'vue';

export type CalendarSystem = 'gregorian' | 'fixed360' | 'custom';

export interface MonthConfig {
  name: string;
  days: number;
}

// State
const currentSystem = ref<CalendarSystem>('gregorian');
const startYear = ref(1); // The "Year" that corresponds to Unix Epoch (or project start)

// Preset: Fixed 360 (12 months of 30 days)
const fixed360Months: MonthConfig[] = [
  { name: 'Month 1', days: 30 },
  { name: 'Month 2', days: 30 },
  { name: 'Month 3', days: 30 },
  { name: 'Month 4', days: 30 },
  { name: 'Month 5', days: 30 },
  { name: 'Month 6', days: 30 },
  { name: 'Month 7', days: 30 },
  { name: 'Month 8', days: 30 },
  { name: 'Month 9', days: 30 },
  { name: 'Month 10', days: 30 },
  { name: 'Month 11', days: 30 },
  { name: 'Month 12', days: 30 },
];

const customMonths = ref<MonthConfig[]>([...fixed360Months]);

export function useCalendar() {
  const systemName = computed(() => {
    switch (currentSystem.value) {
      case 'gregorian':
        return 'Gregorian (Standard)';
      case 'fixed360':
        return 'Fixed 360 (12x30)';
      case 'custom':
        return 'Custom';
      default:
        return 'Unknown';
    }
  });

  function setSystem(sys: CalendarSystem) {
    currentSystem.value = sys;
  }

  function setStartYear(year: number) {
    startYear.value = year;
  }

  function setCustomMonths(months: MonthConfig[]) {
    customMonths.value = months;
  }

  // Main formatter function for vis-timeline
  function formatDate(date: Date | number | string, _format?: string): string {
    const d = date instanceof Date ? date : new Date(date);
    if (isNaN(d.getTime())) return 'Invalid Date';

    if (currentSystem.value === 'gregorian') {
      return d.toLocaleDateString();
    }

    // Custom Logic
    const epoch = new Date('1970-01-01T00:00:00Z').getTime();
    const diff = d.getTime() - epoch;
    const totalDays = Math.floor(diff / (1000 * 60 * 60 * 24));

    const months = currentSystem.value === 'fixed360' ? fixed360Months : customMonths.value;
    const daysInYear = months.reduce((sum, m) => sum + m.days, 0);

    let year = Math.floor(totalDays / daysInYear) + startYear.value;
    let dayOfYear = totalDays % daysInYear;

    if (totalDays < 0) {
      dayOfYear = daysInYear + (totalDays % daysInYear);
      if (dayOfYear === daysInYear) dayOfYear = 0;
      year = Math.floor(totalDays / daysInYear) + startYear.value - 1;
    }

    let currentMonth = months[0];
    let remainingDays = dayOfYear;

    for (const m of months) {
      if (remainingDays < m.days) {
        currentMonth = m;
        break;
      }
      remainingDays -= m.days;
    }

    return `${currentMonth.name} ${remainingDays + 1}, Year ${year}`;
  }

  // Helpers for specific parts
  function getYear(date: Date | number | string): string {
    const d = date instanceof Date ? date : new Date(date);
    if (isNaN(d.getTime())) return 'N/A';

    if (currentSystem.value === 'gregorian') return d.getFullYear().toString();

    const epoch = new Date('1970-01-01T00:00:00Z').getTime();
    const diff = d.getTime() - epoch;
    const totalDays = Math.floor(diff / (1000 * 60 * 60 * 24));

    const months = currentSystem.value === 'fixed360' ? fixed360Months : customMonths.value;
    const daysInYear = months.reduce((sum, m) => sum + m.days, 0);

    const year = Math.floor(totalDays / daysInYear) + startYear.value;
    return `Year ${year}`;
  }

  return {
    currentSystem,
    startYear,
    customMonths,
    systemName,
    setSystem,
    setStartYear,
    setCustomMonths,
    formatDate,
    getYear,
  };
}
