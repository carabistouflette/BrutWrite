<script setup lang="ts">
import { CharacterRole, type Character } from '../../types';

const props = defineProps<{
  modelValue: Character;
  hasChanges: boolean;
}>();

const emit = defineEmits(['update:modelValue', 'change', 'save', 'delete', 'close']);

// Roles for select
const roles = Object.values(CharacterRole);

const updateField = (field: keyof Character, value: string) => {
  emit('update:modelValue', { ...props.modelValue, [field]: value });
  emit('change');
};

const updateEngineField = (field: string, value: string) => {
  emit('update:modelValue', {
    ...props.modelValue,
    engine: { ...props.modelValue.engine, [field]: value },
  });
  emit('change');
};
</script>

<template>
  <div class="flex-1 flex flex-col h-full bg-transparent overflow-hidden relative">
    <!-- Toolbar/Header -->
    <div class="px-8 py-4 border-b border-ink/5 flex justify-between items-center bg-paper/50">
      <div class="flex items-center gap-4">
        <input
          :value="modelValue.name"
          class="text-2xl font-serif font-bold bg-transparent border-none focus:ring-0 p-0 text-ink placeholder-ink/20 w-full max-w-md focus:outline-none"
          placeholder="Character Name"
          @input="(e) => updateField('name', (e.target as HTMLInputElement).value)"
        />
        <span
          v-if="hasChanges"
          class="text-xs text-accent font-medium bg-accent/10 px-2 py-0.5 rounded-full animate-pulse"
          >Unsaved</span
        >
      </div>

      <button
        class="w-8 h-8 flex items-center justify-center rounded-full hover:bg-black/5 text-ink/40 hover:text-ink transition-colors"
        @click="$emit('close')"
      >
        <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M6 18L18 6M6 6l12 12"
          />
        </svg>
      </button>
    </div>

    <!-- Main Scrollable Content -->
    <div class="flex-1 overflow-y-auto px-8 py-8 custom-scrollbar">
      <div class="max-w-4xl mx-auto space-y-12">
        <!-- Core Identity -->
        <section class="grid grid-cols-1 md:grid-cols-2 gap-8">
          <div class="space-y-4">
            <label class="block text-xs uppercase tracking-widest text-ink/40 font-bold"
              >Role in Story</label
            >
            <div class="flex flex-wrap gap-2">
              <button
                v-for="role in roles"
                :key="role"
                class="px-4 py-2 rounded-lg text-sm transition-all border font-bold"
                :class="
                  modelValue.role === role
                    ? 'bg-ink text-paper! border-ink shadow-md'
                    : 'bg-transparent border-ink/10 text-ink/60 hover:border-ink/30'
                "
                @click="updateField('role', role)"
              >
                {{ role.charAt(0).toUpperCase() + role.slice(1) }}
              </button>
            </div>
          </div>

          <div class="space-y-4">
            <label class="block text-xs uppercase tracking-widest text-ink/40 font-bold"
              >Archetype</label
            >
            <input
              :value="modelValue.archetype"
              class="w-full bg-ink/5 border border-ink/10 rounded-xl px-4 py-3 focus:outline-none focus:bg-ink/10 focus:ring-2 focus:ring-accent/20 transition-all font-medium text-ink placeholder-ink/20"
              placeholder="e.g. The Reluctant Hero, The Mentor"
              @input="(e) => updateField('archetype', (e.target as HTMLInputElement).value)"
            />
          </div>
        </section>

        <!-- The Engine -->
        <section>
          <div class="flex items-center gap-3 mb-6">
            <h3 class="text-lg font-bold font-serif italic text-ink">The Engine</h3>
            <div class="h-px flex-1 bg-ink/10"></div>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <!-- Desire -->
            <div
              class="p-5 bg-linear-to-br from-green-500/5 to-transparent rounded-2xl border border-green-500/10 hover:border-green-500/20 transition-colors group"
            >
              <label
                class="block text-xs uppercase tracking-widest text-green-700/60 font-bold mb-2 group-hover:text-green-700 transition-colors"
                >Goal / Desire</label
              >
              <textarea
                :value="modelValue.engine?.desire"
                rows="3"
                class="w-full bg-transparent resize-none focus:outline-none text-ink placeholder-ink/10 leading-relaxed font-medium"
                placeholder="What do they want more than anything?"
                @input="(e) => updateEngineField('desire', (e.target as HTMLTextAreaElement).value)"
              ></textarea>
            </div>

            <!-- Fear -->
            <div
              class="p-5 bg-linear-to-br from-red-500/5 to-transparent rounded-2xl border border-red-500/10 hover:border-red-500/20 transition-colors group"
            >
              <label
                class="block text-xs uppercase tracking-widest text-red-700/60 font-bold mb-2 group-hover:text-red-700 transition-colors"
                >Fear / Ghost</label
              >
              <textarea
                :value="modelValue.engine?.fear"
                rows="3"
                class="w-full bg-transparent resize-none focus:outline-none text-ink/90 placeholder-ink/20 leading-relaxed"
                placeholder="What are they running from?"
                @input="(e) => updateEngineField('fear', (e.target as HTMLTextAreaElement).value)"
              ></textarea>
            </div>

            <!-- Wound -->
            <div
              class="p-5 bg-linear-to-br from-purple-500/5 to-transparent rounded-2xl border border-purple-500/10 hover:border-purple-500/20 transition-colors group"
            >
              <label
                class="block text-xs uppercase tracking-widest text-purple-700/60 font-bold mb-2 group-hover:text-purple-700 transition-colors"
                >The Wound</label
              >
              <textarea
                :value="modelValue.engine?.wound"
                rows="3"
                class="w-full bg-transparent resize-none focus:outline-none text-ink/90 placeholder-ink/20 leading-relaxed"
                placeholder="The past trauma defining them..."
                @input="(e) => updateEngineField('wound', (e.target as HTMLTextAreaElement).value)"
              ></textarea>
            </div>

            <!-- Secret -->
            <div
              class="p-5 bg-linear-to-br from-amber-500/5 to-transparent rounded-2xl border border-amber-500/10 hover:border-amber-500/20 transition-colors group"
            >
              <label
                class="block text-xs uppercase tracking-widest text-amber-700/60 font-bold mb-2 group-hover:text-amber-700 transition-colors"
                >The Secret</label
              >
              <textarea
                :value="modelValue.engine?.secret"
                rows="3"
                class="w-full bg-transparent resize-none focus:outline-none text-ink/90 placeholder-ink/20 leading-relaxed"
                placeholder="What creates tension?"
                @input="(e) => updateEngineField('secret', (e.target as HTMLTextAreaElement).value)"
              ></textarea>
            </div>
          </div>
        </section>

        <!-- Physical & Notes -->
        <section class="grid grid-cols-1 gap-8">
          <div class="space-y-3">
            <h3 class="text-lg font-bold font-serif italic text-ink">Physicality</h3>
            <div
              class="bg-ink/5 rounded-xl p-4 border border-ink/10 focus-within:ring-2 focus-within:ring-accent/10 transition-all"
            >
              <textarea
                :value="modelValue.physical_features"
                rows="4"
                class="w-full bg-transparent resize-none focus:outline-none text-ink placeholder-ink/10 leading-relaxed font-medium"
                placeholder="Distinguishing features, mannerisms, style..."
                @input="
                  (e) => updateField('physical_features', (e.target as HTMLTextAreaElement).value)
                "
              ></textarea>
            </div>
          </div>

          <div class="space-y-3">
            <h3 class="text-lg font-bold font-serif italic text-ink">Notes & Arc</h3>
            <div
              class="bg-ink/5 rounded-xl p-4 border border-ink/10 focus-within:ring-2 focus-within:ring-accent/10 transition-all"
            >
              <textarea
                :value="modelValue.notes"
                rows="8"
                class="w-full bg-transparent resize-none focus:outline-none text-ink placeholder-ink/10 leading-relaxed font-medium"
                placeholder="General notes, ideas, character arc progression..."
                @input="(e) => updateField('notes', (e.target as HTMLTextAreaElement).value)"
              ></textarea>
            </div>
          </div>
        </section>

        <!-- Actions -->
        <div class="flex justify-end gap-4 pt-8 border-t border-ink/5">
          <button
            class="px-5 py-2.5 rounded-lg text-sm font-medium text-red-600 hover:bg-red-50 transition-colors"
            @click="$emit('delete')"
          >
            Delete Character
          </button>
          <button
            :disabled="!hasChanges"
            class="px-8 py-2.5 rounded-lg text-sm font-medium transition-all shadow-lg shadow-accent/20"
            :class="
              hasChanges
                ? 'bg-accent text-white hover:bg-accent-dark hover:shadow-accent/40'
                : 'bg-stone text-ink/40 cursor-not-allowed'
            "
            @click="$emit('save')"
          >
            Save Changes
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: rgba(0, 0, 0, 0.1);
  border-radius: 3px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background-color: rgba(0, 0, 0, 0.2);
}
</style>
