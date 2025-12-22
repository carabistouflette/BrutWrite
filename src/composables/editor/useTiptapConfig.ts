import StarterKit from '@tiptap/starter-kit';
import Focus from '@tiptap/extension-focus';
import { useTiptapMentions } from './useTiptapMentions';
import type { Transaction } from '@tiptap/pm/state';

export function useTiptapConfig(
  onUpdate: (props: { transaction: Transaction }) => void,
  onSelectionUpdate: () => void
) {
  const { mentionExtension } = useTiptapMentions();

  const extensions = [
    StarterKit.configure({
      heading: { levels: [1, 2, 3] },
    }),
    Focus.configure({
      className: 'has-focus',
      mode: 'all',
    }),
    mentionExtension,
  ];

  const editorProps = {
    attributes: {
      class: 'prose prose-invert prose-lg max-w-none focus:outline-none min-h-screen p-16',
      spellcheck: 'false',
    },
  };

  return {
    extensions,
    editorProps,
    onUpdate,
    onSelectionUpdate,
  };
}
