import { ref, computed, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useProjectStore } from '../../../stores/project';
import { useCharacters } from './useCharacters';
import { CharacterRole, type Character } from '../../../types';
import { useAppStatus } from '../../ui/useAppStatus';
import { useCharacterGraph } from '../../domain/intelligence/useCharacterGraph';

export function useCharacterSheetLogic(emit: (event: 'close') => void) {
  const projectStore = useProjectStore();
  const { projectId } = storeToRefs(projectStore);
  const { characters, saveCharacter, deleteCharacter } = useCharacters();
  const { notifyError, notifySuccess } = useAppStatus();

  const selectedId = ref<string | null>(null);
  const localCharacter = ref<Character | null>(null);
  const hasChanges = ref(false);
  const showDeleteConfirm = ref(false);
  const showUnsavedConfirm = ref(false);
  const pendingSelectedId = ref<string | null>(null);
  const createAfterDiscard = ref(false);

  const selectedCharacter = computed(() => {
    return characters.value.find((character) => character.id === selectedId.value);
  });

  watch(selectedId, (newId) => {
    if (newId && selectedCharacter.value) {
      localCharacter.value = JSON.parse(JSON.stringify(selectedCharacter.value));
      hasChanges.value = false;
    } else {
      localCharacter.value = null;
      hasChanges.value = false;
    }
  });

  watch(
    projectId,
    () => {
      selectedId.value = null;
      localCharacter.value = null;
      hasChanges.value = false;
      showDeleteConfirm.value = false;
      showUnsavedConfirm.value = false;
      pendingSelectedId.value = null;
      createAfterDiscard.value = false;
      emit('close');
    },
    { flush: 'sync' }
  );

  const attemptClose = () => {
    if (hasChanges.value) {
      pendingSelectedId.value = null;
      createAfterDiscard.value = false;
      showUnsavedConfirm.value = true;
    } else {
      emit('close');
    }
  };

  const cancelUnsavedChanges = () => {
    showUnsavedConfirm.value = false;
    pendingSelectedId.value = null;
    createAfterDiscard.value = false;
  };

  const requestSelectCharacter = (id: string) => {
    if (id === selectedId.value) return;
    if (hasChanges.value) {
      pendingSelectedId.value = id;
      createAfterDiscard.value = false;
      showUnsavedConfirm.value = true;
      return;
    }
    selectedId.value = id;
  };

  const generateUUID = () => {
    if (typeof crypto !== 'undefined' && crypto.randomUUID) {
      return crypto.randomUUID();
    }
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function (c) {
      const r = (Math.random() * 16) | 0,
        v = c == 'x' ? r : (r & 0x3) | 0x8;
      return v.toString(16);
    });
  };

  const performCreateCharacter = async () => {
    const currentProjectId = projectId.value;
    if (!currentProjectId) {
      notifyError('Cannot create character: No active project found.');
      return;
    }

    try {
      const newCharacter: Character = {
        id: generateUUID(),
        name: 'New Character',
        role: CharacterRole.Secondary,
        archetype: '',
        description: '',
        engine: { desire: '', fear: '', wound: '', secret: '' },
        physical_features: '',
        aliases: [],
        traits: [],
        arc: '',
        notes: '',
      };

      await saveCharacter(currentProjectId, newCharacter);
      if (projectId.value !== currentProjectId) return;

      notifySuccess(`Character ${newCharacter.name} created`);
      selectedId.value = newCharacter.id;
      useCharacterGraph().analyze();
    } catch (error) {
      notifyError('Failed to create character', error);
    }
  };

  const createCharacter = async () => {
    if (hasChanges.value) {
      pendingSelectedId.value = null;
      createAfterDiscard.value = true;
      showUnsavedConfirm.value = true;
      return;
    }
    await performCreateCharacter();
  };

  const forceClose = async () => {
    showUnsavedConfirm.value = false;
    const nextId = pendingSelectedId.value;
    const shouldCreate = createAfterDiscard.value;
    pendingSelectedId.value = null;
    createAfterDiscard.value = false;

    if (nextId) {
      selectedId.value = nextId;
    } else if (shouldCreate) {
      await performCreateCharacter();
    } else {
      emit('close');
    }
  };

  const saveCurrent = async () => {
    const currentProjectId = projectId.value;
    const character = localCharacter.value;
    if (!currentProjectId || !character) return;

    try {
      await saveCharacter(currentProjectId, character);
      if (projectId.value !== currentProjectId || localCharacter.value?.id !== character.id) return;

      hasChanges.value = false;
      notifySuccess('Character saved');

      const graphStore = useCharacterGraph();
      graphStore.analyze();
    } catch (error) {
      notifyError('Failed to save character', error);
    }
  };

  const requestDelete = () => {
    if (!localCharacter.value) return;
    showDeleteConfirm.value = true;
  };

  const confirmDelete = async () => {
    const currentProjectId = projectId.value;
    const character = localCharacter.value;
    if (!currentProjectId || !character) return;

    try {
      await deleteCharacter(currentProjectId, character.id);
      if (projectId.value !== currentProjectId) return;

      selectedId.value = null;
      localCharacter.value = null;
      hasChanges.value = false;
      showDeleteConfirm.value = false;
      notifySuccess(`Character ${character.name} deleted`);
      const graphStore = useCharacterGraph();
      graphStore.clear();
      graphStore.analyze();
    } catch (error) {
      notifyError('Failed to delete character', error);
    }
  };

  const handleChange = () => {
    hasChanges.value = true;
  };

  return {
    characters,
    selectedId,
    localCharacter,
    hasChanges,
    showDeleteConfirm,
    showUnsavedConfirm,
    attemptClose,
    forceClose,
    cancelUnsavedChanges,
    requestSelectCharacter,
    createCharacter,
    saveCurrent,
    requestDelete,
    confirmDelete,
    handleChange,
  };
}
