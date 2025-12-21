<script setup lang="ts">
import { ref } from 'vue';
import GeneralSettings from './settings/GeneralSettings.vue';
import EditorSettings from './settings/EditorSettings.vue';
import InterfaceSettings from './settings/InterfaceSettings.vue';

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits(['close']);

const activeTab = ref<'general' | 'editor' | 'interface'>('general');

const close = () => {
  emit('close');
};

const tabs = [
  { id: 'general', label: 'General', icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z' },
  { id: 'editor', label: 'Editor', icon: 'M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z' },
  { id: 'interface', label: 'Interface', icon: 'M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z' }
] as const;
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
        class="fixed inset-0 z-50 flex items-center justify-center p-4 lg:p-12"
      >
        <!-- Backdrop -->
        <div 
            class="absolute inset-0 bg-black/40 backdrop-blur-md"
            @click="close"
        ></div>

        <!-- Window Container -->
        <div 
            class="relative w-full max-w-4xl h-[85%] flex bg-paper/90 backdrop-blur-xl border border-white/40 dark:border-white/5 shadow-2xl rounded-2xl overflow-hidden text-ink modal-container"
            :class="{ 'modal-exit': !show }"
        >
            <!-- Sidebar -->
            <div class="w-64 cyber-glass border-r border-ink/5 flex flex-col">
                <div class="p-6">
                    <h2 class="font-serif text-2xl italic font-bold">Settings</h2>
                </div>
                
                <nav class="flex-1 px-3 space-y-1">
                    <button 
                        v-for="tab in tabs" 
                        :key="tab.id"
                        @click="activeTab = tab.id"
                        class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-200"
                        :class="activeTab === tab.id ? 'bg-stone shadow-sm text-accent' : 'text-ink/60 hover:bg-stone/50 hover:text-ink'"
                    >
                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" :d="tab.icon" />
                        </svg>
                        {{ tab.label }}
                    </button>
                </nav>

                <div class="p-4 border-t border-ink/5 text-center text-[10px] text-ink/30 uppercase tracking-widest">
                    BrutWrite v0.1.0
                </div>
            </div>

            <!-- Content Area -->
            <div class="flex-1 flex flex-col h-full bg-transparent relative">
                <!-- Header (Optional) -->
                <div class="px-8 py-6 flex justify-between items-center">
                    <h3 class="text-xl font-medium tracking-tight">{{ tabs.find(t => t.id === activeTab)?.label }}</h3>
                    <button 
                         @click="close"
                         class="w-8 h-8 flex items-center justify-center rounded-full hover:bg-black/5 text-ink/40 hover:text-ink transition-colors"
                     >
                        <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                     </button>
                </div>

                <!-- Scrollable Settings -->
                <div class="flex-1 overflow-y-auto px-8 pb-8 custom-scrollbar">
                    <Transition 
                        mode="out-in"
                        enter-active-class="transition duration-300 ease-out"
                        enter-from-class="opacity-0 translate-x-4"
                        enter-to-class="opacity-100 translate-x-0"
                        leave-active-class="transition duration-150 ease-in"
                        leave-from-class="opacity-100 translate-x-0"
                        leave-to-class="opacity-0 -translate-x-4"
                    >
                        <KeepAlive>
                            <component :is="
                                activeTab === 'general' ? GeneralSettings :
                                activeTab === 'editor' ? EditorSettings :
                                InterfaceSettings
                            " />
                        </KeepAlive>
                    </Transition>
                </div>
            </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
    width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
    background-color: var(--color-stone);
    border-radius: 3px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background-color: var(--color-ink-rgb);
    opacity: 0.2;
}

@keyframes modal-in {
    from { 
        opacity: 0; 
        transform: scale(0.95) translateY(20px); 
    }
    to { 
        opacity: 1; 
        transform: scale(1) translateY(0); 
    }
}

.modal-container {
    animation: modal-in 0.4s cubic-bezier(0.16, 1, 0.3, 1) forwards;
    box-shadow: 
        0 20px 50px -12px rgba(0, 0, 0, 0.2), 
        0 0 0 1px rgba(255, 255, 255, 0.4) inset;
}

:global(.dark) .modal-container {
    box-shadow: 
        0 20px 50px -12px rgba(0, 0, 0, 0.5);
}

.modal-exit {
    transition: all 0.2s ease-in;
    opacity: 0;
    transform: scale(0.98) translateY(10px);
}
</style>
