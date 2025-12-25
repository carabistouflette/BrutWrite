<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';
import BaseIcon from '../base/BaseIcon.vue';

const props = defineProps<{
  show: boolean;
  initialTags: string[];
  artifactName: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save', tags: string[]): void;
}>();

const localTags = ref<string[]>([]);
const newTagInput = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

watch(
  () => props.show,
  (newVal) => {
    if (newVal) {
      localTags.value = [...props.initialTags];
      newTagInput.value = '';
      nextTick(() => {
        inputRef.value?.focus();
      });
    }
  }
);

const addTag = () => {
  const tag = newTagInput.value.trim();
  if (tag && !localTags.value.includes(tag)) {
    localTags.value.push(tag);
  }
  newTagInput.value = '';
};

const removeTag = (tagToRemove: string) => {
  localTags.value = localTags.value.filter((t) => t !== tagToRemove);
};

const handleSave = () => {
  emit('save', localTags.value);
  emit('close');
};
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition duration-150 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div v-if="show" class="fixed inset-0 z-100 flex items-center justify-center p-4">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-paper/80 backdrop-blur-sm" @click="$emit('close')"></div>

        <!-- Modal -->
        <div
          class="bg-paper border border-ink/10 shadow-2xl rounded-xl w-full max-w-md relative z-10 overflow-hidden flex flex-col max-h-[80vh]"
          @click.stop
        >
          <!-- Header -->
          <div
            class="px-6 py-4 border-b border-ink/5 bg-stone/20 flex justify-between items-center"
          >
            <div>
              <h3 class="font-serif text-lg font-bold text-ink">Manage Tags</h3>
              <p class="text-xs text-ink/40 truncate max-w-[250px]">{{ artifactName }}</p>
            </div>
            <!-- Close Button -->
            <button class="text-ink/30 hover:text-ink transition-colors" @click="$emit('close')">
              <BaseIcon name="x" size="20" />
            </button>
          </div>

          <!-- Content -->
          <div class="p-6 overflow-y-auto custom-scrollbar">
            <!-- Input Area -->
            <div class="mb-6">
              <label class="block text-[10px] font-black uppercase tracking-widest text-ink/30 mb-2"
                >Add New Tag</label
              >
              <div class="flex gap-2">
                <input
                  ref="inputRef"
                  v-model="newTagInput"
                  type="text"
                  placeholder="Type and press Enter..."
                  class="flex-1 bg-stone/30 border border-ink/5 rounded-lg px-3 py-2 text-sm text-ink focus:outline-none focus:border-accent/50 focus:bg-stone/50 transition-all placeholder:text-ink/20"
                  @keydown.enter.prevent="addTag"
                />
                <button
                  class="px-4 py-2 bg-ink text-paper text-xs font-bold uppercase tracking-wider rounded-lg hover:bg-ink/80 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  :disabled="!newTagInput.trim()"
                  @click="addTag"
                >
                  Add
                </button>
              </div>
            </div>

            <!-- Tags List -->
            <div>
              <label class="block text-[10px] font-black uppercase tracking-widest text-ink/30 mb-3"
                >Current Tags</label
              >
              <div v-if="localTags.length > 0" class="flex flex-wrap gap-2">
                <div
                  v-for="tag in localTags"
                  :key="tag"
                  class="group flex items-center gap-1.5 pl-3 pr-1.5 py-1 bg-stone/40 hover:bg-stone border border-ink/5 rounded-full transition-all duration-200"
                >
                  <span class="text-xs font-medium text-ink/70 group-hover:text-ink"
                    >#{{ tag }}</span
                  >
                  <button
                    class="p-0.5 rounded-full hover:bg-red-500/10 text-ink/20 hover:text-red-500 transition-colors"
                    @click="removeTag(tag)"
                  >
                    <BaseIcon name="x" size="14" />
                  </button>
                </div>
              </div>
              <p v-else class="text-sm text-ink/20 italic text-center py-4">
                No tags assigned yet.
              </p>
            </div>
          </div>

          <!-- Footer -->
          <div class="px-6 py-4 border-t border-ink/5 bg-stone/10 flex justify-end gap-3">
            <button
              class="px-4 py-2 text-xs font-bold uppercase tracking-wider text-ink/40 hover:text-ink transition-colors"
              @click="$emit('close')"
            >
              Cancel
            </button>
            <button
              class="px-6 py-2 bg-accent text-white text-xs font-bold uppercase tracking-wider rounded-lg shadow-lg shadow-accent/20 hover:bg-accent/90 hover:shadow-accent/30 hover:-translate-y-0.5 transition-all duration-200"
              @click="handleSave"
            >
              Save Changes
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: var(--stone);
  border-radius: 99px;
}
.custom-scrollbar:hover::-webkit-scrollbar-thumb {
  background: var(--ink-rgb);
  opacity: 0.2;
}
</style>
