import { ref, type Ref } from 'vue';
import { useEditor, type Editor } from '@tiptap/vue-3';
import { useTiptapConfig } from './useTiptapConfig';
import { useEditorScroll } from './useEditorScroll';
import { useEditorWordCount } from './useEditorWordCount';

export function useTiptapEditor(onContentChange?: (delta: number) => void) {
  const containerRef = ref<HTMLElement | null>(null);
  const isDirty = ref(false);

  const { debouncedWordCountUpdate, resetWordCountState } = useEditorWordCount(onContentChange);

  const editor = useEditor(
    useTiptapConfig(
      ({ transaction }) => {
        if (transaction.docChanged) {
          isDirty.value = true;
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
      editor.value.commands.setContent(content, { emitUpdate: false });
      isDirty.value = false;
      resetWordCountState(editor.value);
    }
  };

  const getHTML = () => {
    return editor.value?.getHTML() || '';
  };

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
