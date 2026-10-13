import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { mount, type VueWrapper } from '@vue/test-utils';
import { nextTick } from 'vue';
import GamificationDashboard from '../GamificationDashboard.vue';
import { useSettingsStore } from '../../../stores/settings';

let wrapper: VueWrapper | undefined;

beforeEach(() => {
  setActivePinia(createPinia());
});

afterEach(() => {
  wrapper?.unmount();
  wrapper = undefined;
});

describe('goal adjustment', () => {
  it('edits the current daily goal after preferences change while the dashboard is mounted', async () => {
    const settingsStore = useSettingsStore();
    settingsStore.settings.general.dailyGoal = 12;
    wrapper = mount(GamificationDashboard, {
      props: { show: true },
      global: { stubs: { teleport: true } },
    });

    settingsStore.settings.general.dailyGoal = 15;
    await nextTick();
    const adjustGoals = wrapper
      .findAll('button')
      .find((button) => button.text() === 'Adjust Goals')!;
    await adjustGoals.trigger('click');

    expect((wrapper.get('input[type="number"]').element as HTMLInputElement).value).toBe('15');
  });
});
