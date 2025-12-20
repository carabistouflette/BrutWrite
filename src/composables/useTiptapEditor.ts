import { ref } from 'vue';
import { useEditor } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Link from '@tiptap/extension-link';
import { projectApi } from '../api/project';

export function useTiptapEditor(
    onContentChange?: (count: number) => void
) {
    const containerRef = ref<HTMLElement | null>(null);
    const lastWordCount = ref(0);

    const editor = useEditor({
        content: '',
        extensions: [
            StarterKit.configure({
                heading: { levels: [1, 2, 3] }
            }),
            Link.configure({ openOnClick: false }),
        ],
        editorProps: {
            attributes: {
                class: 'prose prose-invert prose-lg max-w-none focus:outline-none min-h-screen p-16',
            },
        },
        onUpdate: ({ transaction }) => {
            handleScroll();

            const text = transaction.doc.textContent;
            const newCount = text.trim().length === 0
                ? 0
                : text.split(/\s+/).filter(w => w.length > 0).length;

            const delta = newCount - lastWordCount.value;
            if (delta !== 0) {
                if (onContentChange) onContentChange(delta);
                lastWordCount.value = newCount;
            }
        },
        onSelectionUpdate: () => {
            handleScroll();
        }
    });

    const handleScroll = () => {
        requestAnimationFrame(() => {
            if (!editor.value || !containerRef.value) return;

            const { from } = editor.value.state.selection;
            const view = editor.value.view;
            const coords = view.coordsAtPos(from);
            const containerRect = containerRef.value.getBoundingClientRect();

            const containerCenter = containerRect.top + containerRect.height / 2;
            const cursorY = coords.top;
            const diff = cursorY - containerCenter;

            if (Math.abs(diff) > 50) {
                containerRef.value.scrollBy({ top: diff, behavior: 'smooth' });
            }
        });
    };

    const resetWordCountState = () => {
        if (!editor.value) return;
        const text = editor.value.state.doc.textContent;
        lastWordCount.value = text.trim().length === 0 ? 0 : text.split(/\s+/).filter(w => w.length > 0).length;
    }

    // --- Backend Actions ---

    const loadChapter = async (projectId: string, chapterId: string) => {
        if (!editor.value) return;
        try {
            const content = await projectApi.loadChapter(projectId, chapterId);
            editor.value.commands.setContent(content, { emitUpdate: false });
            resetWordCountState();
        } catch (e) {
            console.error('Failed to load chapter:', e);
            editor.value.commands.setContent(`<h1>Error</h1><p>Could not load chapter.</p>`);
        }
    };

    const saveChapter = async (projectId: string, chapterId: string) => {
        if (!editor.value) return;
        try {
            const content = editor.value.getHTML();
            await projectApi.saveChapter(projectId, chapterId, content);
        } catch (e) {
            console.error('Failed to save chapter:', e);
        }
    }

    return {
        editor,
        containerRef,
        loadChapter,
        saveChapter,
        resetWordCountState
    };
}
