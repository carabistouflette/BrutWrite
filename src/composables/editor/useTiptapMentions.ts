import { VueRenderer } from '@tiptap/vue-3';
import Mention from '@tiptap/extension-mention';
import tippy from 'tippy.js';
import 'tippy.js/dist/tippy.css';
import MentionList from '../../components/base/MentionList.vue';
import { useCharacters } from '../useCharacters';

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
                let popup: any;

                return {
                    onStart: (props: any) => {
                        component = new VueRenderer(MentionList, {
                            props: props,
                            editor: props.editor,
                        });

                        if (!props.clientRect) {
                            return;
                        }

                        popup = tippy(document.body, {
                            getReferenceClientRect: props.clientRect,
                            appendTo: () => document.body,
                            content: component.element as Element,
                            showOnCreate: true,
                            interactive: true,
                            trigger: 'manual',
                            placement: 'bottom-start',
                        });
                    },
                    onUpdate(props: any) {
                        component.updateProps(props);

                        if (!props.clientRect) {
                            return;
                        }

                        popup[0].setProps({
                            getReferenceClientRect: props.clientRect,
                        });
                    },
                    onKeyDown(props: any) {
                        if (props.event.key === 'Escape') {
                            popup[0].hide();
                            return true;
                        }
                        return component.ref?.onKeyDown(props);
                    },
                    onExit() {
                        popup[0].destroy();
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
