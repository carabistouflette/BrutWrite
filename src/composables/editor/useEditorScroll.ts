import { type Ref } from 'vue';
import { type Editor } from '@tiptap/vue-3';

export function useEditorScroll(
    editor: Ref<Editor | undefined>,
    containerRef: Ref<HTMLElement | null>
) {
    let lastScrollTime = 0;
    let lastFrom = -1;
    let cachedContainerRect: DOMRect | null = null;

    const handleScroll = () => {
        const now = Date.now();
        if (now - lastScrollTime < 30) return; // Throttle slightly more
        
        if (!editor.value || !containerRef.value) return;
        const { from } = editor.value.state.selection;
        if (from === lastFrom) return;
        lastFrom = from;
        
        lastScrollTime = now;

        requestAnimationFrame(() => {
            if (!editor.value || !containerRef.value) return;

            const view = editor.value.view;
            const coords = view.coordsAtPos(from);
            
            if (!cachedContainerRect) {
                cachedContainerRect = containerRef.value.getBoundingClientRect();
                // Reset cache on next resize
                window.addEventListener('resize', () => { cachedContainerRect = null; }, { once: true });
            }
            
            const containerRect = cachedContainerRect;
            const containerCenter = containerRect.top + containerRect.height / 2;
            const cursorY = coords.top;
            const diff = cursorY - containerCenter;

            if (Math.abs(diff) > 50) {
                containerRef.value.scrollBy({ top: diff, behavior: 'smooth' });
            }
        });
    };

    return {
        handleScroll
    };
}
