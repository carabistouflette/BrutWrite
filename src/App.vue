<script setup lang="ts">
import { watch, onMounted } from 'vue';
import AppHeader from './components/AppHeader.vue';
import MainLayout from './components/MainLayout.vue';
import EditorMain from './components/EditorMain.vue';
import { useProjectData } from './composables/useProjectData';
import { useSettings } from './composables/useSettings';

const { activeId } = useProjectData();
const { settings, loadSettings } = useSettings();

// Apply settings to document
watch(() => settings.value.interface.theme, (newTheme) => {
    document.documentElement.classList.remove('light', 'dark');
    if (newTheme === 'system') {
        const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        document.documentElement.classList.add(isDark ? 'dark' : 'light');
    } else {
        document.documentElement.classList.add(newTheme);
    }
}, { immediate: true });

watch(() => settings.value.interface.cyberGlassIntensity, (intensity) => {
    const blur = (intensity / 100) * 40; // max 40px
    const opacity = 0.1 + (intensity / 100) * 0.5; // range 0.1 - 0.6
    document.documentElement.style.setProperty('--cyber-glass-blur', `${blur}px`);
    document.documentElement.style.setProperty('--cyber-glass-opacity', `${opacity}`);
}, { immediate: true });

// Load settings on startup
onMounted(async () => {
    await loadSettings();
});
</script>

<template>
  <div class="h-screen w-screen bg-paper text-ink flex flex-col font-sans overflow-hidden relative">
    <!-- Cyber-Glass App Header -->
    <AppHeader data-tauri-drag-region />

    <!-- Main Content (Sidebar + Editor) -->
    <MainLayout>
        <EditorMain v-if="activeId" />
        <div v-else class="h-full flex flex-col justify-center items-center text-ink/60 select-none">
            <h2 class="text-5xl font-serif font-bold italic tracking-tight mb-2">Start Writing</h2>
            <p class="text-xs font-sans tracking-[0.2em] uppercase text-ink/40 border-t border-accent/30 pt-4 mt-2">Select a chapter from the sidebar</p>
        </div>
    </MainLayout>
  </div>
</template>