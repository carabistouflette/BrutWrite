<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useProjectData } from '../../composables/useProjectData';
import { useCharacters } from '../../composables/useCharacters';
import { CharacterRole, type Character } from '../../types';
import ConfirmationModal from '../base/ConfirmationModal.vue';
import { useAppStatus } from '../../composables/useAppStatus';
import CharacterList from './CharacterList.vue';
import CharacterDetail from './CharacterDetail.vue';

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits(['close']);

const { projectId } = useProjectData();
const { characters, saveCharacter, deleteCharacter } = useCharacters();
const { notifyError, notifySuccess } = useAppStatus();

const selectedId = ref<string | null>(null);

// Local edit state to avoid auto-saving on every keystroke/lag
const localCharacter = ref<Character | null>(null);
const hasChanges = ref(false);

const selectedCharacter = computed(() => {
    return characters.value.find(c => c.id === selectedId.value);
});

// Watch selection to reset local state
watch(selectedId, (newId) => {
    if (newId && selectedCharacter.value) {
        // Deep clone to avoid mutating store directly
        localCharacter.value = JSON.parse(JSON.stringify(selectedCharacter.value));
        hasChanges.value = false;
    } else {
        localCharacter.value = null;
    }
});

const close = () => {
    if (hasChanges.value) {
        if (!confirm('You have unsaved changes. Close anyway?')) return;
    }
    emit('close');
};

const generateUUID = () => {
    if (typeof crypto !== 'undefined' && crypto.randomUUID) {
        return crypto.randomUUID();
    }
    // Simple fallback
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
        var r = Math.random() * 16 | 0, v = c == 'x' ? r : (r & 0x3 | 0x8);
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
            engine: {
                desire: '',
                fear: '',
                wound: '',
                secret: ''
            },
            physical_features: '',
            traits: [],
            arc: '',
            notes: ''
        };
        
        await saveCharacter(projectId.value!, newChar);
        notifySuccess(`Character ${newChar.name} created`);
        
        // Force selection next tick to ensure list is updated
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
        } catch (e) {
            notifyError('Failed to save character', e);
        }
    }
};

const showDeleteConfirm = ref(false);

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
</script>

<template>
  <Teleport to="#app-scale-root">
    <Transition
      enter-active-class="transition duration-300 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition duration-200 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div 
        v-if="show"
        class="fixed inset-0 z-50 flex items-center justify-center p-4 lg:p-8"
      >
        <!-- Backdrop -->
        <div 
            class="absolute inset-0 bg-black/60 backdrop-blur-sm"
            @click="close"
        ></div>

        <!-- Window Container -->
        <div 
            class="relative w-full max-w-6xl h-[85%] flex bg-paper/95 backdrop-blur-2xl border border-white/20 shadow-2xl rounded-2xl overflow-hidden text-ink modal-container"
            style="box-shadow: 0 24px 60px -12px rgba(0, 0, 0, 0.4)"
        >
            <!-- Sidebar (List) -->
            <CharacterList
                :characters="characters"
                :selected-id="selectedId"
                @select="(id) => selectedId = id"
                @create="createCharacter"
            />

            <!-- Content Area (Detail) -->
            <CharacterDetail
                v-if="localCharacter"
                v-model="localCharacter"
                :has-changes="hasChanges"
                @change="handleChange"
                @save="saveCurrent"
                @delete="requestDelete"
                @close="close"
            />
            
            <div v-else class="flex-1 flex flex-col items-center justify-center text-ink/30">
                <div class="w-16 h-16 rounded-full bg-stone mb-4 flex items-center justify-center">
                    <svg class="w-8 h-8 text-ink/20" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                    </svg>
                </div>
                <p>Select a character to view their soul.</p>
            </div>
        </div>
      </div>
    </Transition>
  </Teleport>

  <ConfirmationModal
    :show="showDeleteConfirm"
    title="Delete Character"
    :message="`Are you sure you want to delete ${localCharacter?.name}? This action cannot be undone.`"
    confirm-label="Delete"
    :is-destructive="true"
    @close="showDeleteConfirm = false"
    @confirm="confirmDelete"
  />
</template>

<style scoped>
.modal-container {
    animation: modal-pop 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes modal-pop {
    0% { transform: scale(0.95); opacity: 0; }
    100% { transform: scale(1); opacity: 1; }
}
</style>
