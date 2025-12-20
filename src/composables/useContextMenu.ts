import { ref, onMounted, onUnmounted } from 'vue';
import type { ContextMenuPosition } from '../types';

export function useContextMenu() {
    const showMenu = ref(false);
    const menuPos = ref<ContextMenuPosition>({ x: 0, y: 0 });
    const targetNodeId = ref<string | null>(null);

    const openMenu = (e: MouseEvent, id: string) => {
        e.preventDefault();
        targetNodeId.value = id;
        menuPos.value = { x: e.clientX, y: e.clientY };
        showMenu.value = true;
    };

    const closeMenu = () => {
        showMenu.value = false;
    };

    onMounted(() => {
        document.addEventListener('click', closeMenu);
    });

    onUnmounted(() => {
        document.removeEventListener('click', closeMenu);
    });

    return {
        showMenu,
        menuPos,
        targetNodeId,
        openMenu,
        closeMenu
    };
}
