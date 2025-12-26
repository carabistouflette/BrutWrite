<script setup lang="ts">
import { useCharacterSheetLogic } from '../../composables/domain/characters/useCharacterSheetLogic';
import ConfirmationModal from '../base/ConfirmationModal.vue';
import CharacterList from './CharacterList.vue';
import CharacterDetail from './CharacterDetail.vue';

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits(['close']);

const {
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
} = useCharacterSheetLogic(emit);
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
      <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center p-4 lg:p-8">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" @click="attemptClose"></div>

        <!-- Window Container -->
        <div
          class="relative w-full max-w-6xl h-[85%] flex bg-paper/95 backdrop-blur-2xl border border-white/20 shadow-2xl rounded-2xl overflow-hidden text-ink modal-container"
          style="box-shadow: 0 24px 60px -12px rgba(0, 0, 0, 0.4)"
        >
          <!-- Sidebar (List) -->
          <CharacterList
            :characters="characters"
            :selected-id="selectedId"
            @select="(id) => (selectedId = id)"
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
            @close="attemptClose"
          />

          <div v-else class="flex-1 flex flex-col items-center justify-center text-ink/30">
            <div class="w-16 h-16 rounded-full bg-stone mb-4 flex items-center justify-center">
              <svg
                class="w-8 h-8 text-ink/20"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                />
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

  <ConfirmationModal
    :show="showUnsavedConfirm"
    title="Unsaved Changes"
    message="You have unsaved changes. Are you sure you want to close without saving?"
    confirm-label="Discard Changes"
    :is-destructive="true"
    @close="showUnsavedConfirm = false"
    @confirm="forceClose"
  />
</template>

<style scoped>
.modal-container {
  animation: modal-pop 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes modal-pop {
  0% {
    transform: scale(0.95);
    opacity: 0;
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}
</style>
