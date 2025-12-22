import { ref } from 'vue';
import type { Editor } from '@tiptap/core';
import type { Node as ProsemirrorNode } from '@tiptap/pm/model';

export function useEditorWordCount(
    onContentChange?: (count: number) => void
) {
    const lastWordCount = ref(0);
    let wordCountTimeout: ReturnType<typeof setTimeout>;

    const calculateWordCount = (text: string): number => {
        return text.trim().length === 0
            ? 0
            : text.split(/\s+/).filter(w => w.length > 0).length;
    };

    const debouncedWordCountUpdate = (doc: ProsemirrorNode) => {
        clearTimeout(wordCountTimeout);
        wordCountTimeout = setTimeout(() => {
            const text = doc.textContent;
            const newCount = calculateWordCount(text);

            const delta = newCount - lastWordCount.value;
            if (delta !== 0) {
                if (onContentChange) onContentChange(delta);
                lastWordCount.value = newCount;
            }
        }, 500); // 500ms debounce
    };

    const resetWordCountState = (editor: Editor | null | undefined) => {
        if (!editor) return;
        lastWordCount.value = calculateWordCount(editor.state.doc.textContent);
    };

    return {
        calculateWordCount,
        debouncedWordCountUpdate,
        resetWordCountState,
        lastWordCount
    };
}
