import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import SceneEditor from '../SceneEditor.vue';
import ConfirmationModal from '../../base/ConfirmationModal.vue';
import { useProjectStore } from '../../../stores/project';
import { useProjectSync } from '../../../composables/domain/project/useProjectSync';
import { useTimeline } from '../../../composables/timeline/useTimeline';

const instant = '2026-01-02T23:04:05.000Z';
let wrapper: VueWrapper | undefined;

describe('scene timeline form', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    useProjectStore().setProjectData(
      'scene-form',
      '/scene-form',
      [
        {
          id: 'scene',
          name: 'Scene',
          chronological_date: instant,
          abstract_timeframe: 'Day 10',
          duration: '3 hours',
          word_count: 7,
        },
      ],
      { daily_target: 0, word_target: 0 }
    );
  });

  afterEach(() => {
    wrapper?.unmount();
    wrapper = undefined;
    useProjectSync().cancel('scene-form');
  });

  it('displays the saved instant as an accepted local datetime value', () => {
    wrapper = mount(SceneEditor, { props: { sceneId: 'scene' } });
    const date = new Date(instant);
    const input = wrapper.get<HTMLInputElement>('input[type="datetime-local"]').element;
    expect(input.valueAsNumber).toBe(date.getTime() - date.getTimezoneOffset() * 60_000);
  });

  it('returns dated and abstract scenes to the unplaced list without removing duration or words', async () => {
    const { assignedScenes, unassignedScenes } = useTimeline();
    wrapper = mount(SceneEditor, {
      props: { sceneId: 'scene' },
      global: { stubs: { Teleport: true } },
    });
    expect(assignedScenes.value.map((scene) => scene.id)).toEqual(['scene']);
    await wrapper.get('button.danger').trigger('click');
    wrapper.getComponent(ConfirmationModal).vm.$emit('confirm');
    await flushPromises();
    expect(assignedScenes.value.map((scene) => scene.id)).toEqual([]);
    expect(unassignedScenes.value.map((scene) => scene.id)).toEqual(['scene']);
    expect(useProjectStore().nodeMap.get('scene')).toMatchObject({
      duration: '3 hours',
      word_count: 7,
    });
  });
});
