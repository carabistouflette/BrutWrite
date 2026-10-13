import { afterEach, describe, expect, it } from 'vitest';
import { useCalendar } from '../useCalendar';

const calendar = useCalendar();
const initialMonths = calendar.customMonths.value.map((month) => ({ ...month }));

afterEach(() => {
  calendar.setSystem('gregorian');
  calendar.setStartYear(1);
  calendar.setCustomMonths(initialMonths.map((month) => ({ ...month })));
});

describe('calendar dates before the epoch', () => {
  it.each([
    ['1969-12-31T00:00:00Z', 'Month 12 30, Year 0', 'Year 0'],
    ['1969-01-06T00:00:00Z', 'Month 1 1, Year 0', 'Year 0'],
    ['1969-01-05T00:00:00Z', 'Month 12 30, Year -1', 'Year -1'],
    ['1970-01-01T00:00:00Z', 'Month 1 1, Year 1', 'Year 1'],
  ])('keeps fixed-calendar month and year labels consistent for %s', (date, label, year) => {
    calendar.setSystem('fixed360');
    expect(calendar.formatDate(date)).toBe(label);
    expect(calendar.getYear(date)).toBe(year);
  });

  it('uses the preceding custom year at a negative day with a zero epoch year', () => {
    calendar.setSystem('custom');
    calendar.setStartYear(0);
    calendar.setCustomMonths([{ name: 'Aurora', days: 42 }]);
    expect(calendar.formatDate('1969-12-31T23:59:59Z')).toBe('Aurora 42, Year -1');
    expect(calendar.getYear('1969-12-31T23:59:59Z')).toBe('Year -1');
    expect(calendar.formatDate('1970-01-01T00:00:00Z')).toBe('Aurora 1, Year 0');
  });
});
