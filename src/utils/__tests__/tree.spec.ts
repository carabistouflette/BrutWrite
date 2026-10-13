import { describe, expect, it } from 'vitest';
import type { FileNode } from '../../types';
import { projectToManifest, reconstructHierarchy } from '../tree';

describe('project tree manifest order', () => {
  it('serializes roots and descendants in their visible preorder', () => {
    const nodes: FileNode[] = [
      {
        id: 'chapter-a',
        name: 'Chapter A',
        children: [
          { id: 'section-a1', name: 'Section A1', children: [] },
          { id: 'section-a2', name: 'Section A2', children: [] },
        ],
      },
      { id: 'chapter-b', name: 'Chapter B', children: [] },
      { id: 'chapter-c', name: 'Chapter C', children: [] },
    ];

    const manifest = projectToManifest(nodes);

    expect(manifest.chapters.map(({ id, order, parent_id }) => [id, order, parent_id])).toEqual([
      ['chapter-a', 0, undefined],
      ['section-a1', 1, 'chapter-a'],
      ['section-a2', 2, 'chapter-a'],
      ['chapter-b', 3, undefined],
      ['chapter-c', 4, undefined],
    ]);

    const reconstructed = reconstructHierarchy(manifest.chapters);
    expect(reconstructed.map(({ id }) => id)).toEqual(['chapter-a', 'chapter-b', 'chapter-c']);
    expect(reconstructed[0].children?.map(({ id }) => id)).toEqual(['section-a1', 'section-a2']);
  });

  it('reconstructs sibling and root order from manifest order values', () => {
    const chapters = [
      {
        id: 'chapter-c',
        parent_id: undefined,
        title: 'C',
        filename: 'c.md',
        word_count: 0,
        order: 5,
      },
      {
        id: 'section-b2',
        parent_id: 'chapter-b',
        title: 'B2',
        filename: 'b2.md',
        word_count: 0,
        order: 4,
      },
      {
        id: 'section-a1',
        parent_id: 'chapter-a',
        title: 'A1',
        filename: 'a1.md',
        word_count: 0,
        order: 1,
      },
      {
        id: 'chapter-b',
        parent_id: undefined,
        title: 'B',
        filename: 'b.md',
        word_count: 0,
        order: 2,
      },
      {
        id: 'chapter-a',
        parent_id: undefined,
        title: 'A',
        filename: 'a.md',
        word_count: 0,
        order: 0,
      },
      {
        id: 'section-b1',
        parent_id: 'chapter-b',
        title: 'B1',
        filename: 'b1.md',
        word_count: 0,
        order: 3,
      },
    ];

    const tree = reconstructHierarchy(chapters);

    expect(tree.map(({ id }) => id)).toEqual(['chapter-a', 'chapter-b', 'chapter-c']);
    expect(tree[1].children?.map(({ id }) => id)).toEqual(['section-b1', 'section-b2']);
  });
});
