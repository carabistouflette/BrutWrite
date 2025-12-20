<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useProjectData } from '../../composables/useProjectData';
import { CharacterRole, type Character } from '../../types';
import ConfirmationModal from '../base/ConfirmationModal.vue';

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits(['close']);

const { characters, saveCharacter, deleteCharacter, projectId } = useProjectData();

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
    console.log('Attempting to create character...');
    if (!projectId.value) {
        console.error('Cannot create character: No active project ID');
        // TODO: Replace with a nice toast notification
        alert('Error: No active project found. Please open a project first.');
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
        
        console.log('Sending character to backend:', newChar);
        await saveCharacter(newChar);
        console.log('Character saved successfully.');
        
        // Force selection next tick to ensure list is updated
        setTimeout(() => {
            selectedId.value = newChar.id;
        }, 50);
        
    } catch (e) {
        console.error('Failed to create character:', e);
        alert(`Failed to create character: ${e}`);
    }
};

const saveCurrent = async () => {
    if (localCharacter.value) {
        await saveCharacter(localCharacter.value);
        hasChanges.value = false;
    }
};

const showDeleteConfirm = ref(false);

const requestDelete = () => {
    if (!localCharacter.value) return;
    showDeleteConfirm.value = true;
};

const confirmDelete = async () => {
    if (localCharacter.value) {
        await deleteCharacter(localCharacter.value.id);
        selectedId.value = null;
        showDeleteConfirm.value = false;
    }
};

const handleChange = () => {
    hasChanges.value = true;
};

// Roles for select
const roles = Object.values(CharacterRole);

</script>

<template>
  <Teleport to="body">
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
            class="relative w-full max-w-6xl h-[85vh] flex bg-paper/95 backdrop-blur-2xl border border-white/20 shadow-2xl rounded-2xl overflow-hidden text-ink modal-container"
            style="box-shadow: 0 24px 60px -12px rgba(0, 0, 0, 0.4)"
        >
            <!-- Sidebar (List) -->
            <div class="w-72 border-r border-ink/5 flex flex-col bg-stone/30">
                <div class="p-6 flex justify-between items-center">
                    <h2 class="font-serif text-xl font-bold italic tracking-tight">Dramatis Personae</h2>
                    <button @click="createCharacter" class="p-2 hover:bg-accent/10 text-accent rounded-full transition-colors" title="Add Character">
                        <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                        </svg>
                    </button>
                </div>
                
                <div class="flex-1 overflow-y-auto px-3 space-y-1 custom-scrollbar">
                    <button 
                        v-for="char in characters" 
                        :key="char.id"
                        @click="selectedId = char.id"
                        class="w-full text-left px-4 py-3 rounded-xl transition-all duration-200 group relative border border-transparent"
                        :class="selectedId === char.id ? 'bg-white shadow-sm border-black/5' : 'hover:bg-white/50'"
                    >
                        <div class="font-medium text-ink group-hover:text-black transition-colors">{{ char.name }}</div>
                        <div class="text-xs text-ink/50 uppercase tracking-wider mt-0.5 flex justify-between">
                            <span>{{ char.role }}</span>
                            <span v-if="char.archetype" class="opacity-60 truncate ml-2 max-w-[80px]">{{ char.archetype }}</span>
                        </div>
                    </button>
                    
                    <div v-if="characters.length === 0" class="text-center py-8 text-ink/30 italic text-sm">
                        No characters yet.<br>Create one to start.
                    </div>
                </div>
            </div>

            <!-- Content Area (Detail) -->
            <div class="flex-1 flex flex-col h-full bg-transparent overflow-hidden relative">
                <!-- Toolbar/Header -->
                <div class="px-8 py-4 border-b border-ink/5 flex justify-between items-center bg-paper/50">
                   <div v-if="localCharacter" class="flex items-center gap-4">
                        <input 
                            v-model="localCharacter.name" 
                            @input="handleChange"
                            class="text-2xl font-serif font-bold bg-transparent border-none focus:ring-0 p-0 text-ink placeholder-ink/30 w-full max-w-md focus:outline-none"
                            placeholder="Character Name"
                        />
                        <span v-if="hasChanges" class="text-xs text-accent font-medium bg-accent/10 px-2 py-0.5 rounded-full animate-pulse">Unsaved</span>
                   </div>
                   <div v-else class="text-ink/30 italic">Select a character</div>

                   <button 
                         @click="close"
                         class="w-8 h-8 flex items-center justify-center rounded-full hover:bg-black/5 text-ink/40 hover:text-ink transition-colors"
                     >
                        <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                     </button>
                </div>

                <!-- Main Scrollable Content -->
                <div v-if="localCharacter" class="flex-1 overflow-y-auto px-8 py-8 custom-scrollbar">
                    
                    <div class="max-w-4xl mx-auto space-y-12">
                        
                        <!-- Core Identity -->
                        <section class="grid grid-cols-1 md:grid-cols-2 gap-8">
                            <div class="space-y-4">
                                <label class="block text-xs uppercase tracking-widest text-ink/40 font-bold">Role in Story</label>
                                <div class="flex flex-wrap gap-2">
                                    <button 
                                        v-for="role in roles" 
                                        :key="role"
                                        @click="() => { if (localCharacter) localCharacter.role = role; handleChange(); }"
                                        class="px-4 py-2 rounded-lg text-sm transition-all border"
                                        :class="localCharacter.role === role ? 'bg-ink text-paper border-ink shadow-md' : 'bg-transparent border-ink/10 text-ink/60 hover:border-ink/30'"
                                    >
                                        {{ role.charAt(0).toUpperCase() + role.slice(1) }}
                                    </button>
                                </div>
                            </div>

                            <div class="space-y-4">
                                <label class="block text-xs uppercase tracking-widest text-ink/40 font-bold">Archetype</label>
                                <input 
                                    v-model="localCharacter.archetype"
                                    @input="handleChange"
                                    class="w-full bg-stone/30 border border-ink/5 rounded-xl px-4 py-3 focus:outline-none focus:ring-2 focus:ring-accent/20 transition-all font-medium"
                                    placeholder="e.g. The Reluctant Hero, The Mentor"
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
                                <div class="p-5 bg-linear-to-br from-green-500/5 to-transparent rounded-2xl border border-green-500/10 hover:border-green-500/20 transition-colors group">
                                    <label class="block text-xs uppercase tracking-widest text-green-700/60 font-bold mb-2 group-hover:text-green-700 transition-colors">Goal / Desire</label>
                                    <textarea 
                                        v-model="localCharacter.engine!.desire"
                                        @input="handleChange"
                                        rows="3"
                                        class="w-full bg-transparent resize-none focus:outline-none text-ink/90 placeholder-ink/20 leading-relaxed"
                                        placeholder="What do they want more than anything?"
                                    ></textarea>
                                </div>

                                <!-- Fear -->
                                <div class="p-5 bg-linear-to-br from-red-500/5 to-transparent rounded-2xl border border-red-500/10 hover:border-red-500/20 transition-colors group">
                                    <label class="block text-xs uppercase tracking-widest text-red-700/60 font-bold mb-2 group-hover:text-red-700 transition-colors">Fear / Ghost</label>
                                    <textarea 
                                        v-model="localCharacter.engine!.fear"
                                        @input="handleChange"
                                        rows="3"
                                        class="w-full bg-transparent resize-none focus:outline-none text-ink/90 placeholder-ink/20 leading-relaxed"
                                        placeholder="What are they running from?"
                                    ></textarea>
                                </div>

                                <!-- Wound -->
                                <div class="p-5 bg-linear-to-br from-purple-500/5 to-transparent rounded-2xl border border-purple-500/10 hover:border-purple-500/20 transition-colors group">
                                    <label class="block text-xs uppercase tracking-widest text-purple-700/60 font-bold mb-2 group-hover:text-purple-700 transition-colors">The Wound</label>
                                    <textarea 
                                        v-model="localCharacter.engine!.wound"
                                        @input="handleChange"
                                        rows="3"
                                        class="w-full bg-transparent resize-none focus:outline-none text-ink/90 placeholder-ink/20 leading-relaxed"
                                        placeholder="The past trauma defining them..."
                                    ></textarea>
                                </div>

                                <!-- Secret -->
                                <div class="p-5 bg-linear-to-br from-amber-500/5 to-transparent rounded-2xl border border-amber-500/10 hover:border-amber-500/20 transition-colors group">
                                    <label class="block text-xs uppercase tracking-widest text-amber-700/60 font-bold mb-2 group-hover:text-amber-700 transition-colors">The Secret</label>
                                    <textarea 
                                        v-model="localCharacter.engine!.secret"
                                        @input="handleChange"
                                        rows="3"
                                        class="w-full bg-transparent resize-none focus:outline-none text-ink/90 placeholder-ink/20 leading-relaxed"
                                        placeholder="What creates tension?"
                                    ></textarea>
                                </div>
                            </div>
                        </section>

                        <!-- Physical & Notes -->
                        <section class="grid grid-cols-1 gap-8">
                            <div class="space-y-3">
                                <h3 class="text-lg font-bold font-serif italic text-ink">Physicality</h3>
                                <div class="bg-white/40 rounded-xl p-4 border border-ink/5 focus-within:ring-2 focus-within:ring-accent/10 transition-all">
                                    <textarea 
                                        v-model="localCharacter.physical_features"
                                        @input="handleChange"
                                        rows="4"
                                        class="w-full bg-transparent resize-none focus:outline-none text-ink/90 placeholder-ink/20 leading-relaxed"
                                        placeholder="Distinguishing features, mannerisms, style..."
                                    ></textarea>
                                </div>
                            </div>
                             
                            <div class="space-y-3">
                                <h3 class="text-lg font-bold font-serif italic text-ink">Notes & Arc</h3>
                                <div class="bg-white/40 rounded-xl p-4 border border-ink/5 focus-within:ring-2 focus-within:ring-accent/10 transition-all">
                                    <textarea 
                                        v-model="localCharacter.notes"
                                        @input="handleChange"
                                        rows="8"
                                        class="w-full bg-transparent resize-none focus:outline-none text-ink/90 placeholder-ink/20 leading-relaxed"
                                        placeholder="General notes, ideas, character arc progression..."
                                    ></textarea>
                                </div>
                            </div>
                        </section>

                        <!-- Actions -->
                        <div class="flex justify-end gap-4 pt-8 border-t border-ink/5">
                            <button 
                                @click="requestDelete" 
                                class="px-5 py-2.5 rounded-lg text-sm font-medium text-red-600 hover:bg-red-50 transition-colors"
                            >
                                Delete Character
                            </button>
                            <button 
                                @click="saveCurrent" 
                                :disabled="!hasChanges"
                                class="px-8 py-2.5 rounded-lg text-sm font-medium transition-all shadow-lg shadow-accent/20"
                                :class="hasChanges ? 'bg-accent text-white hover:bg-accent-dark hover:shadow-accent/40' : 'bg-stone text-ink/40 cursor-not-allowed'"
                            >
                                Save Changes
                            </button>
                        </div>

                    </div>
                </div>
                
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
.custom-scrollbar::-webkit-scrollbar {
    width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
    background-color: rgba(0,0,0,0.1);
    border-radius: 3px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background-color: rgba(0,0,0,0.2);
}

.modal-container {
    animation: modal-pop 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes modal-pop {
    0% { transform: scale(0.95); opacity: 0; }
    100% { transform: scale(1); opacity: 1; }
}
</style>
