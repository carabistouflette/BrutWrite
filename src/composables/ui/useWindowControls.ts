import { getCurrentWindow } from '@tauri-apps/api/window';

export function useWindowControls() {
  // Check if running in Tauri environment
  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

  const minimize = () => {
    if (isTauri) {
      getCurrentWindow().minimize();
    }
    // Non-Tauri environments: no-op (e.g., browser dev mode)
  };

  const toggleMaximize = () => {
    if (isTauri) {
      getCurrentWindow().toggleMaximize();
    }
    // Non-Tauri environments: no-op
  };

  const close = () => {
    if (isTauri) {
      getCurrentWindow().close();
    }
    // Non-Tauri environments: no-op
  };

  return {
    minimize,
    toggleMaximize,
    close,
    isTauri,
  };
}
