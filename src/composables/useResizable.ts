import { ref, onUnmounted } from 'vue';

interface UseResizableOptions {
    initialWidth: number;
    minWidth: number;
    maxWidth: number;
}

export function useResizable({ initialWidth, minWidth, maxWidth }: UseResizableOptions) {
    const width = ref(initialWidth);
    const isResizing = ref(false);

    const handleResize = (e: MouseEvent) => {
        if (isResizing.value) {
            const scale = (parseFloat(document.documentElement.style.getPropertyValue('--ui-scale')) || 1);
            width.value = Math.max(minWidth, Math.min(e.clientX / scale, maxWidth));
        }
    };

    const stopResize = () => {
        isResizing.value = false;
        document.removeEventListener('mousemove', handleResize);
        document.removeEventListener('mouseup', stopResize);
        document.body.style.cursor = '';
        document.body.style.userSelect = '';
    };

    const startResize = () => {
        isResizing.value = true;
        document.addEventListener('mousemove', handleResize);
        document.addEventListener('mouseup', stopResize);
        document.body.style.cursor = 'col-resize';
        document.body.style.userSelect = 'none';
    };

    onUnmounted(() => {
        stopResize(); // Cleanup just in case
    });

    return {
        width,
        isResizing,
        startResize
    };
}
