import StarterKit from '@tiptap/starter-kit';
import Focus from '@tiptap/extension-focus';
import Link from '@tiptap/extension-link';
import { useTiptapMentions } from './useTiptapMentions';
import type { Transaction } from '@tiptap/pm/state';

export function useTiptapConfig(
  onUpdate: (props: { transaction: Transaction }) => void,
  onSelectionUpdate: () => void
) {
  const { mentionExtension } = useTiptapMentions();

  let extensions = [
    StarterKit.configure({
      heading: { levels: [1, 2, 3] },
      // In later versions of Tiptap, some kits may include link by default.
      // We disable it here to use our custom-configured Link extension below.
      link: false,
    }),
    Focus.configure({
      className: 'has-focus',
      mode: 'all',
    }),
    Link.configure({
      openOnClick: false,
      HTMLAttributes: {
        class: 'text-accent underline cursor-pointer',
      },
    }),
    ...mentionExtension,
  ];

  // Robustly deduplicate extensions by name to resolve any internal or package-level duplication
  const extensionNames = new Set<string>();
  extensions = extensions.filter((extension) => {
    if (!extension) return false;
    const name = (extension as { name?: string }).name;
    if (!name || extensionNames.has(name)) {
      return false;
    }
    extensionNames.add(name);
    return true;
  });

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
