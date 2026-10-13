import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { useProjectStore } from '../../../stores/project';
import type { Character } from '../../../types';
import { useCharacters } from './useCharacters';

const mocks = vi.hoisted(() => ({
  save: vi.fn(),
  delete: vi.fn(),
  notify: vi.fn(),
  notifyError: vi.fn(),
  analyze: vi.fn(),
  flush: vi.fn(),
}));

vi.mock('../../../api/characters', () => ({
  charactersApi: {
    save: mocks.save,
    delete: mocks.delete,
  },
}));

vi.mock('../../ui/useAppStatus', () => ({
  useAppStatus: () => ({ notify: mocks.notify, notifyError: mocks.notifyError }),
}));

vi.mock('../intelligence/useCharacterGraph', () => ({
  useCharacterGraph: () => ({ analyze: mocks.analyze }),
}));
vi.mock('../project/useProjectSync', () => ({
  useProjectSync: () => ({ flush: mocks.flush }),
}));

function makeCharacter(id: string, name: string, aliases: string[] = []): Character {
  return {
    id,
    name,
    role: 'secondary' as Character['role'],
    archetype: '',
    description: '',
    engine: { desire: '', fear: '', wound: '', secret: '' },
    physical_features: '',
    traits: [],
    arc: '',
    notes: '',
    aliases,
  };
}

describe('character text associations', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();

    const projectStore = useProjectStore();
    projectStore.setProjectData('project-1', '/project-1', [], { daily_target: 0, word_target: 0 });
    projectStore.setCharacters([
      makeCharacter('character-1', 'Alice', ['Al']),
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
    mocks.delete.mockResolvedValue({ characters: [] });
  });
  it('flushes queued project metadata before deletion and clears matching POV metadata only', async () => {
    const projectStore = useProjectStore();
    projectStore.setProjectData(
      'project-1',
      '/project-1',
      [
        { id: 'matching', name: 'Matching', pov_character_id: 'CHARACTER-1' },
        { id: 'unrelated', name: 'Unrelated', pov_character_id: 'Narrator' },
      ],
      { daily_target: 0, word_target: 0 }
    );
    const order: string[] = [];
    mocks.flush.mockImplementationOnce(async (projectId: string) => {
      order.push(`flush:${projectId}`);
    });
    mocks.delete.mockImplementationOnce(async (_projectId: string, characterId: string) => {
      order.push('delete');
      return {
        characters: projectStore.characters.filter((character) => character.id !== characterId),
      };
    });

    const { deleteCharacter } = useCharacters();
    await deleteCharacter('project-1', 'character-1');

    expect(order).toEqual(['flush:project-1', 'delete']);
    expect(projectStore.chapterById('matching')?.pov_character_id).toBeUndefined();
    expect(projectStore.chapterById('unrelated')?.pov_character_id).toBe('Narrator');
  });

  it('moves a selected alias to its newly associated character', async () => {
    const projectStore = useProjectStore();
    const { addAliasToCharacter } = useCharacters();

    await addAliasToCharacter('project-1', 'character-2', ' Al ');

    expect(
      projectStore.characters.find((character) => character.id === 'character-1')?.aliases
    ).toEqual([]);
    expect(
      projectStore.characters.find((character) => character.id === 'character-2')?.aliases
    ).toEqual(['Al']);
    expect(mocks.save).toHaveBeenCalledTimes(2);
    expect(mocks.analyze).toHaveBeenCalledTimes(1);
  });

  it('does not duplicate case-insensitive aliases on the same character', async () => {
    const projectStore = useProjectStore();
    projectStore.setCharacters([
      makeCharacter('character-1', 'Alice', ['Al']),
      makeCharacter('character-2', 'Bob', ['Alias']),
    ]);
    const { addAliasToCharacter } = useCharacters();

    await addAliasToCharacter('project-1', 'character-2', ' alias ');

    expect(mocks.save).not.toHaveBeenCalled();
    expect(
      projectStore.characters.find((character) => character.id === 'character-2')?.aliases
    ).toEqual(['Alias']);
  });

  it('does not let a completed save for another project replace the active project characters', async () => {
    mocks.save.mockImplementationOnce(async () => {
      await Promise.resolve();
      return { characters: [makeCharacter('character-1', 'Edited Alice')] };
    });

    const projectStore = useProjectStore();
    const { saveCharacter } = useCharacters();
    const save = saveCharacter('project-1', makeCharacter('character-1', 'Edited Alice'));

    projectStore.setProjectData('project-2', '/project-2', [], { daily_target: 0, word_target: 0 });
    const activeCharacters = [makeCharacter('project-2-character', 'Current project character')];
    projectStore.setCharacters(activeCharacters);
    await save;

    expect(projectStore.characters).toEqual(activeCharacters);
  });
  it('does not reassign alias ownership against a new project after the target save resolves', async () => {
    const projectStore = useProjectStore();
    const originalCharacters = [...projectStore.characters];
    mocks.save.mockImplementationOnce(async (_projectId: string, updatedCharacter: Character) => {
      await Promise.resolve();
      return {
        characters: originalCharacters.map((character) =>
          character.id === updatedCharacter.id ? updatedCharacter : character
        ),
      };
    });

    const { addAliasToCharacter } = useCharacters();
    const addAlias = addAliasToCharacter('project-1', 'character-2', 'Al');
    projectStore.setProjectData('project-2', '/project-2', [], { daily_target: 0, word_target: 0 });
    const activeCharacters = [
      makeCharacter('project-2-owner', 'Current owner', ['Al']),
      makeCharacter('project-2-target', 'Current target'),
    ];
    projectStore.setCharacters(activeCharacters);
    await addAlias;

    expect(mocks.save).toHaveBeenCalledOnce();
    expect(projectStore.characters).toEqual(activeCharacters);
  });
});
