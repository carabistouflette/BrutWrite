import { ref, onMounted, onUnmounted } from 'vue';
import type { ContextMenuPosition } from '../types';

export function useContextMenu() {
    const showMenu = ref(false);
    const menuPos = ref<ContextMenuPosition>({ x: 0, y: 0 });
    const targetNodeId = ref<string | null>(null);

    const openMenu = (e: MouseEvent, id: string) => {
        e.preventDefault();
        targetNodeId.value = id;

        // Adjust coordinates for UI Scaling
        // If the document is zoomed, clientX/Y are viewport relative, 
        // but absolute positioning inside the zoomed element needs to be scaled back.
        const scale = (parseFloat(document.documentElement.style.getPropertyValue('--ui-scale')) || 1);

        menuPos.value = {
            x: e.clientX / scale,
            y: e.clientY / scale
        };
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
