import { ref } from 'vue';
import { useEditor } from '@tiptap/vue-3';
import { useTiptapConfig } from './useTiptapConfig';
import { useEditorScroll } from './useEditorScroll';
import { useEditorWordCount } from './useEditorWordCount';
import { useEditorPersistence } from './useEditorPersistence';

export function useTiptapEditor(onContentChange?: (count: number) => void) {
  const containerRef = ref<HTMLElement | null>(null);
  const isDirty = ref(false); // Track if content has changed

  const { debouncedWordCountUpdate, resetWordCountState } = useEditorWordCount(onContentChange);

  const editor = useEditor(
    useTiptapConfig(
      ({ transaction }) => {
        isDirty.value = true;
        handleScroll();
        debouncedWordCountUpdate(transaction.doc);
      },
      () => {
        handleScroll();
      }
    )
  );

  const { handleScroll } = useEditorScroll(editor, containerRef);

  const { loadChapter, saveChapter } = useEditorPersistence(editor, isDirty, () =>
    resetWordCountState(editor.value)
  );

  return {
    editor,
    containerRef,
    loadChapter,
    saveChapter,
    resetWordCountState: () => resetWordCountState(editor.value),
  };
}
