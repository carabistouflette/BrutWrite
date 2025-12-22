import { ref } from 'vue';

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

    const debouncedWordCountUpdate = (doc: any) => {
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

    const resetWordCountState = (editor: any) => {
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
