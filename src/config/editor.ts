import StarterKit from '@tiptap/starter-kit';
import Focus from '@tiptap/extension-focus';
import Link from '@tiptap/extension-link';

export const getBaseExtensions = () => [
  StarterKit.configure({
    heading: { levels: [1, 2, 3] },
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
];

export const EDITOR_PROPS = {
  attributes: {
    class: 'prose prose-invert prose-lg max-w-none focus:outline-none min-h-screen p-16',
    spellcheck: 'false',
  },
};
