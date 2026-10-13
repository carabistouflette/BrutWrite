import { describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import { CharacterRole, type Character } from '../../types';
import CharacterDetail from './CharacterDetail.vue';

function makeCharacter(): Character {
  return {
    id: 'character-1',
    name: 'Alice',
    role: CharacterRole.Protagonist,
    archetype: 'Mentor',
    description: 'Original description',
    engine: { desire: '', fear: '', wound: '', secret: '' },
    physical_features: '',
    traits: ['Brave'],
    arc: 'Original arc',
    notes: 'Original notes',
    aliases: [],
  };
}

describe('character details form', () => {
  it('preserves a trailing newline while typing successive traits and resets on character changes', async () => {
    const wrapper = mount(CharacterDetail, {
      props: { modelValue: makeCharacter(), hasChanges: true },
    });
    await wrapper.setProps({
      'onUpdate:modelValue': (updated: Character) => {
        void wrapper.setProps({ modelValue: updated });
      },
    });

    const traits = wrapper.get<HTMLTextAreaElement>('#character-traits');
    await traits.setValue('Brave\n');
    expect(traits.element.value).toBe('Brave\n');
    await traits.setValue(`${traits.element.value}Careful`);
    expect(wrapper.props('modelValue').traits).toEqual(['Brave', 'Careful']);
    expect(traits.element.value).toBe('Brave\nCareful');

    await wrapper.setProps({
      modelValue: { ...makeCharacter(), id: 'character-2', traits: ['Curious'] },
    });
    expect(traits.element.value).toBe('Curious');
  });
});
