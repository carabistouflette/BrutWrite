<script setup lang="ts">
import { watch, onMounted } from 'vue';
import AppHeader from './components/layout/AppHeader.vue';
import MainLayout from './components/layout/MainLayout.vue';
import EditorMain from './components/EditorMain.vue';
import WelcomeScreen from './components/WelcomeScreen.vue';
import AppNotifications from './components/base/AppNotifications.vue';
import { useProjectData } from './composables/logic/useProjectData';
import { useSettings } from './composables/logic/useSettings';

const { activeId, projectId } = useProjectData();
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

watch(() => settings.value.interface.uiScaling, (scaling) => {
    const scale = scaling / 100;
    document.documentElement.style.setProperty('--ui-scale', `${scale}`);
    // Zoom property is buggy with drag-and-drop libraries coordinates.
    // We'll apply scaling to a wrapper instead of documentElement zoom.
}, { immediate: true });

// Load settings on startup
onMounted(async () => {
    await loadSettings();
    
    // Auto-load last project
    const lastPath = localStorage.getItem('last_opened_project_path');
    if (lastPath) {
        // We use the composable's loadProject directly
        const { loadProject } = useProjectData();
        console.log('Auto-loading project from:', lastPath);
        await loadProject(lastPath);
    }
});
</script>

<template>
  <div 
    id="app-scale-root"
    class="bg-paper text-ink flex flex-col font-sans overflow-hidden relative"
    :style="{ 
      transform: `scale(${settings.interface.uiScaling / 100})`,
      transformOrigin: 'top left',
      width: `${100 / (settings.interface.uiScaling / 100)}vw`,
      height: `${100 / (settings.interface.uiScaling / 100)}vh`
    }"
  >
    <!-- Cyber-Glass App Header -->
    <AppHeader data-tauri-drag-region />

    <!-- Welcome Screen / Project Loader -->
    <WelcomeScreen v-if="!projectId" />

    <!-- Main Content (Sidebar + Editor) -->
    <MainLayout v-else>
        <EditorMain v-if="activeId" />
        <div v-else class="h-full flex flex-col justify-center items-center text-ink/60 select-none">
            <h2 class="text-5xl font-serif font-bold italic tracking-tight mb-2">Start Writing</h2>
            <p class="text-xs font-sans tracking-[0.2em] uppercase text-ink/40 border-t border-accent/30 pt-4 mt-2">Select a chapter from the sidebar</p>
        </div>
    </MainLayout>

    <AppNotifications />
  </div>
</template>