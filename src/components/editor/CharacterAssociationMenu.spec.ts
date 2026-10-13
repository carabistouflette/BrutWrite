import { describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import { CharacterRole, type Character } from '../../types';
import CharacterAssociationMenu from './CharacterAssociationMenu.vue';

const characters: Character[] = [
  {
    id: 'character-1',
    name: 'Alice',
    role: CharacterRole.Protagonist,
    archetype: '',
    description: '',
    engine: { desire: '', fear: '', wound: '', secret: '' },
    physical_features: '',
    traits: [],
    arc: '',
    notes: '',
    aliases: [],
  },
  {
    id: 'character-2',
    name: 'Bob',
    role: CharacterRole.Secondary,
    archetype: '',
    description: '',
    engine: { desire: '', fear: '', wound: '', secret: '' },
    physical_features: '',
    traits: [],
    arc: '',
    notes: '',
    aliases: [],
  },
];

function mountMenu(hasCharacterMark = false, markedCharacterId = 'character-2') {
  return mount(CharacterAssociationMenu, {
    props: {
      x: 0,
      y: 0,
      selectedText: 'Bob',
      characters,
      hasCharacterMark,
      markedCharacterId: hasCharacterMark ? markedCharacterId : null,
    },
    global: {
      stubs: { ContextMenu: { template: '<div><slot /></div>' } },
    },
  });
}

describe('editor character association menu', () => {
  it('lets a deleted-character mention be removed without resolving its missing character', async () => {
    const wrapper = mountMenu(true, 'deleted-character');
    const removeButton = wrapper
      .findAll('button')
      .find((button) => button.text().includes('Remove Reference'));

    expect(removeButton).toBeDefined();
    expect(removeButton?.element.disabled).toBe(false);
    expect(removeButton?.text()).not.toContain('Bob');
    await removeButton?.trigger('click');
    expect(wrapper.emitted('remove-association')).toHaveLength(1);
  });
  it('emits the selected character from a deleted mention to a current character', async () => {
    const wrapper = mountMenu(true, 'deleted-character');
    await wrapper.findAll('button')[0].trigger('click');
    const bobButton = wrapper.findAll('button').find((button) => button.text().includes('Bob'));

    expect(bobButton).toBeDefined();
    await bobButton?.trigger('click');
    expect(wrapper.emitted('select-character')).toEqual([['character-2']]);
  });
});
