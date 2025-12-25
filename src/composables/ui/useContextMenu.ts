import { ref, onMounted, onUnmounted, type Ref } from 'vue';
import type { ContextMenuPosition } from '../../types';

export function useContextMenu<T = unknown>() {
  const showMenu = ref(false);
  const menuPos = ref<ContextMenuPosition>({ x: 0, y: 0 });
  const contextData = ref<T | null>(null) as Ref<T | null>;

  const openMenu = (e: MouseEvent, data: T) => {
    e.preventDefault();
    contextData.value = data;

    // ContextMenu is teleported to body, so we need viewport coordinates (clientX/Y)
    // The menu handles its own scaling visually.
    menuPos.value = {
      x: e.clientX,
      y: e.clientY,
    };
    // Defer showing to avoid immediate closure by document listener
    setTimeout(() => {
      showMenu.value = true;
    }, 0);
  };

  const closeMenu = () => {
    showMenu.value = false;
    // Optional: clear data on close? Often better to keep it for animations or short checks after close
    // contextData.value = null;
  };

  const handleContextMenuClose = () => {
    if (showMenu.value) closeMenu();
  };

  onMounted(() => {
    document.addEventListener('click', closeMenu);
    document.addEventListener('contextmenu', handleContextMenuClose);
  });

  onUnmounted(() => {
    document.removeEventListener('click', closeMenu);
    document.removeEventListener('contextmenu', handleContextMenuClose);
  });

  return {
    showMenu,
    menuPos,
    contextData,
    openMenu,
    closeMenu,
  };
}
