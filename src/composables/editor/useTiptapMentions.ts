import { Extension, Mark, mergeAttributes } from '@tiptap/core';
import { PluginKey } from '@tiptap/pm/state';
import Suggestion from '@tiptap/suggestion';
import { VueRenderer } from '@tiptap/vue-3';
import tippy, { type Instance } from 'tippy.js';
import { useCharacters } from '../../composables/domain/characters/useCharacters';
import { useResearchStore } from '../../stores/research';
import MentionList from '../../components/base/MentionList.vue';
import type { Editor, Range } from '@tiptap/core';

// --- Types ---
interface MentionItem {
  id: string;
  name: string;
  role?: string;
  type?: 'character' | 'research';
}

interface SuggestionProps {
  editor: Editor;
  query: string;
  items: MentionItem[];
  command: (props: MentionItem) => void;
  range: { from: number; to: number };
  clientRect?: (() => DOMRect | null) | null;
}

// --- Custom Mark for Character Mentions ---
// Explicitly defines the schema for mentions to prevent attribute stripping
const CharacterMark = Mark.create({
  name: 'characterMention',

  keepOnSplit: false,
  excludes: '_', // Disallow other marks (like bold/italic) inside the mention? flexible.

  addAttributes() {
    return {
      id: {
        default: null,
        parseHTML: (element) => element.getAttribute('data-id'),
        renderHTML: (attributes) => ({
          'data-id': attributes.id,
          'data-type': 'character-mention',
          class: 'mention text-accent font-medium bg-accent/10 rounded px-1 decoration-clone', // Styled classes
        }),
      },
    };
  },

  parseHTML() {
    return [
      {
        tag: 'span[data-type="character-mention"]',
      },
    ];
  },

  renderHTML({ HTMLAttributes }) {
    return ['span', mergeAttributes(HTMLAttributes), 0];
  },
});

export function useTiptapMentions() {
  const { characters } = useCharacters();
  const researchStore = useResearchStore();

  /*
   * 1. Character Suggestions
   * Inserts text with the CharacterMention mark.
   */
  const CharacterSuggestion = Extension.create({
    name: 'characterSuggestion',

    addOptions() {
      return {
        suggestion: {
          char: '@',
          pluginKey: new PluginKey('characterSuggestion'),
          command: ({
            editor,
            range,
            props,
          }: {
            editor: Editor;
            range: Range;
            props: MentionItem;
          }) => {
            // Replace the @query with the Name and apply the semantic mark
            editor
              .chain()
              .focus()
              .insertContentAt(range, [
                {
                  type: 'text',
                  text: props.name,
                  marks: [
                    {
                      type: 'characterMention',
                      attrs: {
                        id: props.id,
                      },
                    },
                  ],
                },
                {
                  type: 'text',
                  text: ' ', // Add space after for typing flow
                },
              ])
              .run();
          },
        },
      };
    },

    addProseMirrorPlugins() {
      return [
        Suggestion({
          editor: this.editor,
          ...this.options.suggestion,
        }),
      ];
    },
  });

  /*
   * Configuration for Suggestions
   */
  const suggestionOptions = {
    items: ({ query }: { query: string }) => {
      return characters.value
        .filter((item) => item.name.toLowerCase().startsWith(query.toLowerCase()))
        .slice(0, 5)
        .map((c) => ({ id: c.id, name: c.name, role: c.role, type: 'character' }));
    },
    render: () => {
      let component: VueRenderer;
      let popup: Instance[] | null = null;

      return {
        onStart: (props: SuggestionProps) => {
          component = new VueRenderer(MentionList, {
            props: props,
            editor: props.editor,
          });

          if (!props.clientRect) return;

          popup = tippy('body', {
            getReferenceClientRect: props.clientRect as () => DOMRect,
            appendTo: () => document.body,
            content: component.element as Element,
            showOnCreate: true,
            interactive: true,
            trigger: 'manual',
            placement: 'bottom-start',
          });
        },
        onUpdate(props: SuggestionProps) {
          component.updateProps(props);
          if (!props.clientRect || !popup) return;
          popup[0].setProps({
            getReferenceClientRect: props.clientRect as () => DOMRect,
          });
        },
        onKeyDown(props: { event: KeyboardEvent }) {
          if (props.event.key === 'Escape') {
            popup?.[0].hide();
            return true;
          }
          return component.ref?.onKeyDown(props);
        },
        onExit() {
          popup?.[0].destroy();
          component.destroy();
        },
      };
    },
  };

  /*
   * Research Suggestions (Keep as Link logic)
   */
  const ResearchSuggestion = Extension.create({
    name: 'researchSuggestion',

    addOptions() {
      return {
        suggestion: {
          char: '[',
          pluginKey: new PluginKey('researchSuggestion'),
          command: ({
            editor,
            range,
            props,
          }: {
            editor: Editor;
            range: Range;
            props: MentionItem;
          }) => {
            editor
              .chain()
              .focus()
              .insertContentAt(range, [
                {
                  type: 'text',
                  text: props.name,
                  marks: [
                    {
                      type: 'link',
                      attrs: { href: `research://${props.id}` },
                    },
                  ],
                },
              ])
              .run();
          },
        },
      };
    },

    addProseMirrorPlugins() {
      return [
        Suggestion({
          editor: this.editor,
          ...this.options.suggestion,
        }),
      ];
    },
  });

  const researchSuggestionOptions = {
    items: async ({ query }: { query: string }) => {
      if (researchStore.artifacts.length === 0) {
        await researchStore.fetchArtifacts();
      }
      return researchStore.artifacts
        .filter((item) => item.name.toLowerCase().includes(query.toLowerCase()))
        .slice(0, 10)
        .map((a) => ({ id: a.id, name: a.name, role: a.file_type }));
    },
    render: suggestionOptions.render, // Reuse renderer
  };

  return {
    mentionExtension: [
      CharacterMark,
      CharacterSuggestion.configure({ suggestion: suggestionOptions }),
      ResearchSuggestion.configure({ suggestion: researchSuggestionOptions }),
    ],
  };
}
