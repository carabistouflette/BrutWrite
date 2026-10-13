import { afterEach, describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import CalendarSettings from './CalendarSettings.vue';
import { useCalendar } from '../../composables/timeline/useCalendar';

const calendar = useCalendar();
const initialMonths = calendar.customMonths.value.map((month) => ({ ...month }));

afterEach(() => {
  calendar.setSystem('gregorian');
  calendar.setStartYear(1);
  calendar.setCustomMonths(initialMonths.map((month) => ({ ...month })));
});

function openCalendar() {
  calendar.setSystem('custom');
  calendar.setStartYear(7);
  calendar.setCustomMonths([{ name: 'Aurora', days: 42 }]);
  return mount(CalendarSettings);
}

describe('custom calendar authoring', () => {
  it.each(['0', '-1', '1.5', ''])(
    'keeps the active calendar intact when month days are %j',
    async (days) => {
      const wrapper = openCalendar();
      await wrapper.get('.month-days').setValue(days);
      expect(wrapper.get<HTMLButtonElement>('.btn-primary').element.disabled).toBe(true);
      expect(wrapper.get('[role="alert"]').text()).toContain('positive whole number');
      await wrapper.get('.btn-primary').trigger('click');
      expect(calendar.formatDate('1970-01-01T00:00:00Z')).toBe('Aurora 1, Year 7');
      wrapper.unmount();
    }
  );

  it('blocks an empty year and recovers after adding a valid month', async () => {
    const wrapper = openCalendar();
    await wrapper.get('.remove-btn').trigger('click');
    expect(wrapper.get<HTMLButtonElement>('.btn-primary').element.disabled).toBe(true);
    expect(wrapper.get('[role="alert"]').text()).toContain('at least one month');
    await wrapper.get('.add-btn').trigger('click');
    await wrapper.get('.month-name').setValue('Dawn');
    await wrapper.get('.month-days').setValue('10');
    await wrapper.get('.btn-primary').trigger('click');
    expect(calendar.formatDate('1970-01-11T00:00:00Z')).toBe('Dawn 1, Year 8');
    wrapper.unmount();
  });
});
