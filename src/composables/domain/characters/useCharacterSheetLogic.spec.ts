import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { effectScope, nextTick, type EffectScope } from 'vue';
import { useProjectStore } from '../../../stores/project';
import { CharacterRole, type Character } from '../../../types';
import { useCharacterSheetLogic } from './useCharacterSheetLogic';

const mocks = vi.hoisted(() => ({
  save: vi.fn(),
  delete: vi.fn(),
  notify: vi.fn(),
  notifyError: vi.fn(),
  notifySuccess: vi.fn(),
  analyze: vi.fn(),
  clear: vi.fn(),
}));

vi.mock('../../../api/characters', () => ({
  charactersApi: {
    save: mocks.save,
    delete: mocks.delete,
  },
}));

vi.mock('../../ui/useAppStatus', () => ({
  useAppStatus: () => ({
    notify: mocks.notify,
    notifyError: mocks.notifyError,
    notifySuccess: mocks.notifySuccess,
  }),
}));

vi.mock('../intelligence/useCharacterGraph', () => ({
  useCharacterGraph: () => ({ analyze: mocks.analyze, clear: mocks.clear }),
}));

function makeCharacter(id: string, name: string): Character {
  return {
    id,
    name,
    role: CharacterRole.Secondary,
    archetype: '',
    description: '',
    engine: { desire: '', fear: '', wound: '', secret: '' },
    physical_features: '',
    traits: [],
    arc: '',
    notes: '',
    aliases: [],
  };
}

describe('character sheet transitions', () => {
  const scopes: EffectScope[] = [];

  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();

    const projectStore = useProjectStore();
    projectStore.setProjectData(
      'project-1',
      '/project-1',
      [
        { id: 'chapter-1', name: 'Chapter 1', pov_character_id: 'CHARACTER-1' },
        { id: 'chapter-2', name: 'Chapter 2', pov_character_id: 'Narrator' },
      ],
      { daily_target: 0, word_target: 0 }
    );
    projectStore.setCharacters([
      makeCharacter('character-1', 'Alice'),
      makeCharacter('character-2', 'Bob'),
    ]);

    mocks.save.mockImplementation(async (_projectId: string, updatedCharacter: Character) => {
      const characters = [...useProjectStore().characters];
      const existingIndex = characters.findIndex(
        (character) => character.id === updatedCharacter.id
      );
      if (existingIndex === -1) characters.push(updatedCharacter);
      else characters[existingIndex] = updatedCharacter;
      return { characters };
    });
    mocks.delete.mockImplementation(async (_projectId: string, characterId: string) => ({
      characters: useProjectStore().characters.filter((character) => character.id !== characterId),
    }));
  });

  afterEach(() => {
    scopes.splice(0).forEach((scope) => scope.stop());
  });

  function setupLogic() {
    const scope = effectScope();
    scopes.push(scope);
    const close = vi.fn();
    const logic = scope.run(() => useCharacterSheetLogic(close))!;
    return { close, logic };
  }

  it('asks before switching away from edited character details', async () => {
    const { close, logic } = setupLogic();
    logic.selectedId.value = 'character-1';
    await nextTick();
    logic.handleChange();

    logic.requestSelectCharacter('character-2');
    expect(logic.showUnsavedConfirm.value).toBe(true);
    expect(logic.selectedId.value).toBe('character-1');

    logic.cancelUnsavedChanges();
    expect(logic.selectedId.value).toBe('character-1');
    expect(logic.hasChanges.value).toBe(true);

    logic.requestSelectCharacter('character-2');
    await logic.forceClose();
    await nextTick();

    expect(logic.selectedId.value).toBe('character-2');
    expect(logic.localCharacter.value?.name).toBe('Bob');
    expect(logic.hasChanges.value).toBe(false);
    expect(close).not.toHaveBeenCalled();
  });

  it('confirms before discarding edits to create a new character', async () => {
    const { logic } = setupLogic();
    logic.selectedId.value = 'character-1';
    await nextTick();
    logic.handleChange();

    await logic.createCharacter();
    expect(logic.showUnsavedConfirm.value).toBe(true);
    expect(mocks.save).not.toHaveBeenCalled();

    await logic.forceClose();
    await nextTick();

    expect(useProjectStore().characters).toHaveLength(3);
    expect(logic.localCharacter.value?.name).toBe('New Character');
    expect(logic.hasChanges.value).toBe(false);
  });
  it('clears cached graph references when deleting a character', async () => {
    const { logic } = setupLogic();
    logic.selectedId.value = 'character-1';
    await nextTick();

    logic.requestDelete();
    await logic.confirmDelete();

    expect(useProjectStore().characters.map((character) => character.id)).toEqual(['character-2']);
    expect(useProjectStore().chapterById('chapter-1')?.pov_character_id).toBeUndefined();
    expect(useProjectStore().chapterById('chapter-2')?.pov_character_id).toBe('Narrator');
    expect(logic.selectedId.value).toBeNull();
    expect(logic.localCharacter.value).toBeNull();
    expect(mocks.clear).toHaveBeenCalledOnce();
    expect(mocks.analyze).toHaveBeenCalledOnce();
  });

  it('clears the selected character and dirty state when the project changes', async () => {
    const { close, logic } = setupLogic();
    logic.selectedId.value = 'character-1';
    await nextTick();
    logic.handleChange();
    logic.requestSelectCharacter('character-2');

    const projectStore = useProjectStore();
    projectStore.setProjectData('project-2', '/project-2', [], { daily_target: 0, word_target: 0 });
    projectStore.setCharacters([makeCharacter('project-2-character', 'New project character')]);

    expect(logic.selectedId.value).toBeNull();
    expect(logic.localCharacter.value).toBeNull();
    expect(logic.hasChanges.value).toBe(false);
    expect(logic.showUnsavedConfirm.value).toBe(false);
    expect(close).toHaveBeenCalledOnce();
  });
});
