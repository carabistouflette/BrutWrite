import { ref } from 'vue';
import type { Editor } from '@tiptap/core';
import type { Node as ProsemirrorNode } from '@tiptap/pm/model';

export function useEditorWordCount(onContentChange?: (count: number) => void) {
  const lastWordCount = ref(0);
  let wordCountTimeout: ReturnType<typeof setTimeout> | undefined;

  const calculateWordCount = (text: string): number => {
    return text.trim().length === 0 ? 0 : text.split(/\s+/).filter((w) => w.length > 0).length;
  };

  const cancelWordCountUpdate = () => {
    if (wordCountTimeout !== undefined) clearTimeout(wordCountTimeout);
    wordCountTimeout = undefined;
  };

  const debouncedWordCountUpdate = (doc: ProsemirrorNode) => {
    cancelWordCountUpdate();
    wordCountTimeout = setTimeout(() => {
      const text = doc.textBetween(0, doc.content.size, ' ');
      const newCount = calculateWordCount(text);

      const delta = newCount - lastWordCount.value;
      if (delta !== 0) {
        if (onContentChange) onContentChange(delta);
        lastWordCount.value = newCount;
      }
    }, 500);
  };

  const resetWordCountState = (editor: Editor | null | undefined) => {
    cancelWordCountUpdate();
    if (!editor) return;
    lastWordCount.value = calculateWordCount(
      editor.state.doc.textBetween(0, editor.state.doc.content.size, ' ')
    );
  };

  return {
    calculateWordCount,
    debouncedWordCountUpdate,
    resetWordCountState,
    cancelWordCountUpdate,
    lastWordCount,
  };
}
