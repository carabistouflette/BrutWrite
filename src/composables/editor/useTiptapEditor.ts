import { ref, type Ref, onBeforeUnmount } from 'vue';
import { useEditor, type Editor } from '@tiptap/vue-3';
import { useTiptapConfig } from './useTiptapConfig';
import { useEditorScroll } from './useEditorScroll';
import { useEditorWordCount } from './useEditorWordCount';

export function useTiptapEditor(
  onUpdate?: (payload: { delta: number; html: string }) => void,
  onDocumentChange?: (html: string) => void
) {
  const containerRef = ref<HTMLElement | null>(null);
  const isDirty = ref(false);
  let isProgrammaticContentUpdate = false;

  const { debouncedWordCountUpdate, resetWordCountState, cancelWordCountUpdate } =
    useEditorWordCount((delta) => {
      if (onUpdate) {
        onUpdate({ delta, html: getHTML() });
      }
    });

  const editor = useEditor(
    useTiptapConfig(
      ({ transaction }) => {
        if (transaction.docChanged && !isProgrammaticContentUpdate) {
          isDirty.value = true;
          onDocumentChange?.(getHTML());
          debouncedWordCountUpdate(transaction.doc);
        }
        handleScroll();
      },
      () => {
        handleScroll();
      }
    )
  );

  const { handleScroll } = useEditorScroll(editor, containerRef);

  const setContent = (content: string) => {
    if (editor.value) {
      isProgrammaticContentUpdate = true;
      try {
        editor.value.commands.setContent(content, { emitUpdate: false });
      } finally {
        isProgrammaticContentUpdate = false;
      }
      isDirty.value = false;
      resetWordCountState(editor.value);
    }
  };

  const getHTML = () => {
    return editor.value?.getHTML() || '';
  };

  onBeforeUnmount(cancelWordCountUpdate);

  return {
    editor: editor as Ref<Editor | undefined>,
    containerRef,
    isDirty,
    setContent,
    getHTML,
    markAsClean: () => {
      isDirty.value = false;
    },
    focus: () => {
      editor.value?.commands.focus();
    },
  };
}
