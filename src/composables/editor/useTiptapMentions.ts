import { VueRenderer } from '@tiptap/vue-3';
import Mention from '@tiptap/extension-mention';
import tippy, { type Instance } from 'tippy.js';
import 'tippy.js/dist/tippy.css';
import MentionList from '../../components/base/MentionList.vue';
import { useCharacters } from '../useCharacters';
import type { Editor } from '@tiptap/core';

interface BaseSuggestionProps {
    clientRect?: (() => DOMRect | null) | null;
}

interface SuggestionProps extends BaseSuggestionProps {
    editor: Editor;
    query: string;
    items: any[];
    command: (props: any) => void;
    range: { from: number; to: number };
}

interface SuggestionKeyDownProps extends BaseSuggestionProps {
    event: KeyboardEvent;
    view: any;
    range: { from: number; to: number };
}


export function useTiptapMentions() {
    const { characters } = useCharacters();

    const mentionExtension = Mention.configure({
        HTMLAttributes: {
            class: 'mention',
        },
        suggestion: {
            items: ({ query }: { query: string }) => {
                return characters.value
                    .filter(item => item.name.toLowerCase().startsWith(query.toLowerCase()))
                    .slice(0, 5)
                    .map(c => ({ id: c.id, name: c.name, role: c.role }));
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
                            getReferenceClientRect: props.clientRect as any,
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
                            getReferenceClientRect: props.clientRect as any,
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

    return {
        mentionExtension
    };
}
