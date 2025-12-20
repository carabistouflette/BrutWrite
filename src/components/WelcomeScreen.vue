<template>
  <div class="h-screen w-screen flex flex-col items-center justify-center bg-paper text-ink selection:bg-accent/20">
    <div class="max-w-md w-full px-6 py-12 flex flex-col items-center space-y-12">
        
        <!-- Logo -->
        <div class="text-center space-y-4">
            <h1 class="text-6xl font-serif font-bold italic tracking-tight">BrutWrite</h1>
            <p class="text-xs uppercase tracking-[0.3em] opacity-40">Distraction Free Writing Environment</p>
        </div>

        <!-- Actions -->
        <div class="w-full space-y-4">
            <button 
                @click="handleNewProject"
                class="w-full group relative overflow-hidden bg-ink text-paper py-4 px-6 rounded-xl font-medium transition-all hover:shadow-2xl hover:shadow-ink/20 hover:-translate-y-1"
            >
                <div class="relative z-10 flex items-center justify-center gap-3">
                    <svg class="w-5 h-5 opacity-60" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                    </svg>
                    <span>Create New Project</span>
                </div>
                <div class="absolute inset-0 bg-accent/20 translate-y-full group-hover:translate-y-0 transition-transform duration-300"></div>
            </button>

            <button 
                @click="handleOpenProject"
                class="w-full group relative overflow-hidden bg-transparent border border-ink/10 hover:border-ink/30 text-ink py-4 px-6 rounded-xl font-medium transition-all hover:bg-white/50"
            >
                <div class="flex items-center justify-center gap-3">
                    <svg class="w-5 h-5 opacity-40" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z" />
                    </svg>
                    <span>Open Existing Project</span>
                </div>
            </button>
        </div>

        <!-- Check for recent projects could go here later -->
    </div>
  </div>
</template>

<script setup lang="ts">
import { open, save } from '@tauri-apps/plugin-dialog';
import { useProjectData } from '../composables/useProjectData';

const { loadProject, createProject } = useProjectData();

const handleOpenProject = async () => {
    try {
        const selected = await open({
            title: 'Open Project',
            filters: [
                { name: 'BrutWrite Project', extensions: ['json'] }
            ]
        });

        if (selected && typeof selected === 'string') {
            await loadProject(selected);
        }
    } catch (e) {
        console.error('Failed to open project dialog:', e);
    }
};

const handleNewProject = async () => {
    try {
        // Asking for a directory to create the project in
        const selected = await save({
            title: 'Create New Project',
             filters: [
                { name: 'BrutWrite Project', extensions: ['json'] }
            ],
            defaultPath: 'MyStory.json' 
        });

        if (selected && typeof selected === 'string') {
            // Default project name from filename
            const name = selected.split(/[\\/]/).pop()?.replace('.json', '') || 'Untitled Project';
            await createProject(selected, name, 'Unknown Author');
        }
    } catch (e) {
         console.error('Failed to create project dialog:', e);
    }
};
</script>
