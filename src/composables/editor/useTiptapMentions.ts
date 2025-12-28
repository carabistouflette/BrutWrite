import { VueRenderer } from '@tiptap/vue-3';
import Mention from '@tiptap/extension-mention';
import tippy, { type Instance } from 'tippy.js';
import 'tippy.js/dist/tippy.css';
import MentionList from '../../components/base/MentionList.vue';
import { useCharacters } from '../../composables/domain/characters/useCharacters';
import { useResearchStore } from '../../stores/research';
import type { Editor } from '@tiptap/core';
import type { EditorView } from '@tiptap/pm/view';

interface BaseSuggestionProps {
  clientRect?: (() => DOMRect | null) | null;
}

interface MentionItem {
  id: string;
  name: string;
  role?: string;
}

interface SuggestionProps extends BaseSuggestionProps {
  editor: Editor;
  query: string;
  items: MentionItem[];
  command: (props: { id: string; label: string }) => void;
  range: { from: number; to: number };
}

interface SuggestionKeyDownProps extends BaseSuggestionProps {
  event: KeyboardEvent;
  view: EditorView;
  range: { from: number; to: number };
}

export function useTiptapMentions() {
  const { characters } = useCharacters();
  const researchStore = useResearchStore();

  const characterMention = Mention.extend({ name: 'characterMention' }).configure({
    HTMLAttributes: {
      class: 'mention',
    },
    renderLabel({ node }) {
      return `${node.attrs.label ?? node.attrs.id}`;
    },
    suggestion: {
      items: ({ query }: { query: string }) => {
        return characters.value
          .filter((item) => item.name.toLowerCase().startsWith(query.toLowerCase()))
          .slice(0, 5)
          .map((c) => ({ id: c.id, name: c.name, role: c.role }));
      },
      render: () => {
        let component: VueRenderer;
        let popup: Instance | null = null;

        return {
          onStart: (props: SuggestionProps) => {
            component = new VueRenderer(MentionList, {
              props: props,
              editor: props.editor,
            });

            if (!props.clientRect) {
              return;
            }

            const instances = tippy(document.body, {
              getReferenceClientRect: props.clientRect as () => DOMRect,
              appendTo: () => document.body,
              content: component.element as Element,
              showOnCreate: true,
              interactive: true,
              trigger: 'manual',
              placement: 'bottom-start',
            });
            popup = Array.isArray(instances) ? instances[0] : instances;
          },
          onUpdate(props: SuggestionProps) {
            component.updateProps(props);

            if (!props.clientRect || !popup) {
              return;
            }

            popup.setProps({
              getReferenceClientRect: props.clientRect as () => DOMRect,
            });
          },
          onKeyDown(props: SuggestionKeyDownProps) {
            if (props.event?.key === 'Escape' && popup) {
              popup.hide();
              return true;
            }
            return component.ref?.onKeyDown(props);
          },
          onExit() {
            popup?.destroy();
            component.destroy();
          },
        };
      },
    },
  });

  const researchMention = Mention.extend({ name: 'researchMention' }).configure({
    HTMLAttributes: {
      class: 'research-mention',
    },
    suggestion: {
      char: '[', // Trigger on [
      items: async ({ query }: { query: string }) => {
        if (researchStore.artifacts.length === 0) {
          await researchStore.fetchArtifacts();
        }
        return researchStore.artifacts
          .filter((item) => item.name.toLowerCase().includes(query.toLowerCase()))
          .slice(0, 10)
          .map((a) => ({ id: a.id, name: a.name, role: a.file_type })); // Mapping file_type to role for UI
      },
      render: () => {
        let component: VueRenderer;
        let popup: Instance | null = null;

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
            })[0];
          },
          onUpdate(props: SuggestionProps) {
            component.updateProps(props);
            if (!props.clientRect || !popup) return;
            popup.setProps({
              getReferenceClientRect: props.clientRect as () => DOMRect,
            });
          },
          onKeyDown(props: SuggestionKeyDownProps) {
            if (props.event.key === 'Escape' && popup) {
              popup.hide();
              return true;
            }
            return component.ref?.onKeyDown(props);
          },
          onExit() {
            popup?.destroy();
            component.destroy();
          },
        };
      },
      command: ({
        editor,
        range,
        props,
      }: {
        editor: Editor;
        range: { from: number; to: number };
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        props: any;
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
  });

  return {
    mentionExtension: [characterMention, researchMention],
  };
}
