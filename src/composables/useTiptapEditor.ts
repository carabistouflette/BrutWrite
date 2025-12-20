import { ref } from 'vue';
import { useEditor, VueRenderer } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Focus from '@tiptap/extension-focus';
import Mention from '@tiptap/extension-mention';
import tippy from 'tippy.js';
import 'tippy.js/dist/tippy.css';

import { projectApi } from '../api/project';
import { useCharacters } from './useCharacters';
import MentionList from '../components/base/MentionList.vue';

export function useTiptapEditor(
    onContentChange?: (count: number) => void
) {
    const containerRef = ref<HTMLElement | null>(null);
    const lastWordCount = ref(0);
    const { characters } = useCharacters();

    const editor = useEditor({
        content: '',
        extensions: [
            StarterKit.configure({
                heading: { levels: [1, 2, 3] }
            }),
            Focus.configure({
                className: 'has-focus',
                mode: 'all',
            }),
            Mention.configure({
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
                                    // using vue 3 renderer
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
            }),
        ],
        editorProps: {
            attributes: {
                class: 'prose prose-invert prose-lg max-w-none focus:outline-none min-h-screen p-16',
                spellcheck: 'false',
            },
        },
        // ... (rest of configuration)
        onUpdate: ({ transaction }) => {
            handleScroll();
            debouncedWordCountUpdate(transaction.doc.textContent);
        },
        onSelectionUpdate: () => {
            handleScroll();
        }
    });

    let wordCountTimeout: ReturnType<typeof setTimeout>;
    const debouncedWordCountUpdate = (text: string) => {
        clearTimeout(wordCountTimeout);
        wordCountTimeout = setTimeout(() => {
            const newCount = text.trim().length === 0
                ? 0
                : text.split(/\s+/).filter(w => w.length > 0).length;

            const delta = newCount - lastWordCount.value;
            if (delta !== 0) {
                if (onContentChange) onContentChange(delta);
                lastWordCount.value = newCount;
            }
        }, 500); // 500ms debounce
    };

    let lastScrollTime = 0;
    const handleScroll = () => {
        const now = Date.now();
        if (now - lastScrollTime < 20) return; // Throttle to ~50fps
        lastScrollTime = now;

        requestAnimationFrame(() => {
            if (!editor.value || !containerRef.value) return;

            const { from } = editor.value.state.selection;
            const view = editor.value.view;
            const coords = view.coordsAtPos(from);
            const containerRect = containerRef.value.getBoundingClientRect();

            const containerCenter = containerRect.top + containerRect.height / 2;
            const cursorY = coords.top;
            const diff = cursorY - containerCenter;

            if (Math.abs(diff) > 50) {
                containerRef.value.scrollBy({ top: diff, behavior: 'smooth' });
            }
        });
    };

    const resetWordCountState = () => {
        if (!editor.value) return;
        const text = editor.value.state.doc.textContent;
        lastWordCount.value = text.trim().length === 0 ? 0 : text.split(/\s+/).filter(w => w.length > 0).length;
    }

    // --- Backend Actions ---

    const loadChapter = async (projectId: string, chapterId: string) => {
        if (!editor.value) return;
        try {
            const content = await projectApi.loadChapter(projectId, chapterId);
            editor.value.commands.setContent(content, { emitUpdate: false });
            resetWordCountState();
        } catch (e) {
            console.error('Failed to load chapter:', e);
            editor.value.commands.setContent(`<h1>Error</h1><p>Could not load chapter.</p>`);
        }
    };

    const saveChapter = async (projectId: string, chapterId: string) => {
        if (!editor.value) return;
        try {
            const content = editor.value.getHTML();
            await projectApi.saveChapter(projectId, chapterId, content);
        } catch (e) {
            console.error('Failed to save chapter:', e);
        }
    }

    return {
        editor,
        containerRef,
        loadChapter,
        saveChapter,
        resetWordCountState
    };
}
