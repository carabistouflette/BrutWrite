import { watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useSettingsStore } from '../../stores/settings';
import { APP_CONSTANTS } from '../../config/constants';

export function useTheme() {
  const settingsStore = useSettingsStore();
  const { settings } = storeToRefs(settingsStore);

  const initTheme = () => {
    // Theme Watcher
    watch(
      () => settings.value.interface.theme,
      (newTheme) => {
        document.documentElement.classList.remove('light', 'dark');
        if (newTheme === 'system') {
          const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
          document.documentElement.classList.add(isDark ? 'dark' : 'light');
        } else {
          document.documentElement.classList.add(newTheme);
        }
      },
      { immediate: true }
    );

    // Cyber-glass Intensity Watcher
    watch(
      () => settings.value.interface.cyberGlassIntensity,
      (intensity) => {
        const blur = (intensity / 100) * APP_CONSTANTS.UI.CYBER_GLASS.MAX_BLUR; // max 40px
        const opacity =
          APP_CONSTANTS.UI.CYBER_GLASS.MIN_OPACITY +
          (intensity / 100) * APP_CONSTANTS.UI.CYBER_GLASS.VAR_OPACITY; // range 0.1 - 0.6
        document.documentElement.style.setProperty('--cyber-glass-blur', `${blur}px`);
        document.documentElement.style.setProperty('--cyber-glass-opacity', `${opacity}`);
      },
      { immediate: true }
    );

    // UI Scaling Watcher
    watch(
      () => settings.value.interface.uiScaling,
      (scaling) => {
        const scale = scaling / 100;
        document.documentElement.style.setProperty('--ui-scale', `${scale}`);
      },
      { immediate: true }
    );
  };

  return {
    initTheme,
  };
}
