import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useUIStore = defineStore('ui', () => {
  const isSidebarOpen = ref(true);
  const activePanel = ref<'research' | 'timeline' | 'gamification' | null>(null);

  function toggleSidebar() {
    isSidebarOpen.value = !isSidebarOpen.value;
  }

  function setPanel(panel: 'research' | 'timeline' | 'gamification' | null) {
    if (activePanel.value === panel) {
      activePanel.value = null; // Toggle off if same
    } else {
      activePanel.value = panel;
    }
  }

  return {
    isSidebarOpen,
    activePanel,
    toggleSidebar,
    setPanel,
  };
});
