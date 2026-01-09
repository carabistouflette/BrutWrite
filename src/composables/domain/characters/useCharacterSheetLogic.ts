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

  const selectedCharacter = computed(() => {
    return characters.value.find((c) => c.id === selectedId.value);
  });

  watch(selectedId, (newId) => {
    if (newId && selectedCharacter.value) {
      localCharacter.value = JSON.parse(JSON.stringify(selectedCharacter.value));
      hasChanges.value = false;
    } else {
      localCharacter.value = null;
    }
  });

  const showUnsavedConfirm = ref(false);

  const attemptClose = () => {
    if (hasChanges.value) {
      showUnsavedConfirm.value = true;
    } else {
      emit('close');
    }
  };

  const forceClose = () => {
    showUnsavedConfirm.value = false;
    emit('close');
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

  const createCharacter = async () => {
    if (!projectId.value) {
      notifyError('Cannot create character: No active project found.');
      return;
    }

    try {
      const newChar: Character = {
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

      await saveCharacter(projectId.value, newChar);
      notifySuccess(`Character ${newChar.name} created`);

      setTimeout(() => {
        selectedId.value = newChar.id;
      }, 50);
    } catch (e) {
      notifyError('Failed to create character', e);
    }
  };

  const saveCurrent = async () => {
    if (localCharacter.value && projectId.value) {
      try {
        await saveCharacter(projectId.value, localCharacter.value);
        hasChanges.value = false;
        notifySuccess('Character saved');

        // Trigger graph refresh to reflect changes (e.g. name, aliases)
        const graphStore = useCharacterGraph();
        graphStore.analyze();
      } catch (e) {
        notifyError('Failed to save character', e);
      }
    }
  };

  const requestDelete = () => {
    if (!localCharacter.value) return;
    showDeleteConfirm.value = true;
  };

  const confirmDelete = async () => {
    if (localCharacter.value && projectId.value) {
      try {
        const name = localCharacter.value.name;
        await deleteCharacter(projectId.value, localCharacter.value.id);
        selectedId.value = null;
        showDeleteConfirm.value = false;
        notifySuccess(`Character ${name} deleted`);
      } catch (e) {
        notifyError('Failed to delete character', e);
      }
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
    createCharacter,
    saveCurrent,
    requestDelete,
    confirmDelete,
    handleChange,
  };
}
