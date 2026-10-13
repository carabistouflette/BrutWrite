import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Editor } from '@tiptap/core';
import type { Node as ProsemirrorNode } from '@tiptap/pm/model';
import { useEditorWordCount } from '../useEditorWordCount';

function makeDoc(text: string): ProsemirrorNode {
  return {
    content: { size: 1 },
    textBetween: vi.fn(() => text),
  } as unknown as ProsemirrorNode;
}

describe('useEditorWordCount', () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it('counts words across block boundaries', () => {
    const onContentChange = vi.fn();
    const counter = useEditorWordCount(onContentChange);
    const doc = makeDoc('First paragraph Second paragraph');

    counter.debouncedWordCountUpdate(doc);
    vi.advanceTimersByTime(500);

    expect(doc.textBetween).toHaveBeenCalledWith(0, 1, ' ');
    expect(onContentChange).toHaveBeenCalledWith(4);
  });

  it('cancels a pending delta when loaded content resets the counter', () => {
    const onContentChange = vi.fn();
    const counter = useEditorWordCount(onContentChange);
    counter.debouncedWordCountUpdate(makeDoc('Old draft extra'));
    counter.resetWordCountState({ state: { doc: makeDoc('Restored version') } } as Editor);

    vi.advanceTimersByTime(500);

    expect(counter.lastWordCount.value).toBe(2);
    expect(onContentChange).not.toHaveBeenCalled();
  });
});
