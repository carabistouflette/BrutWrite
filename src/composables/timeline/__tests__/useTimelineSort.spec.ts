import { describe, expect, it } from 'vitest';
import { sortNodesChronologically } from '../useTimelineSort';
import type { FileNode } from '../../../types';

describe('sortNodesChronologically', () => {
  it('orders abstract timeframes by story time rather than lexicographic labels', () => {
    const sorted = sortNodesChronologically([
      { id: 'day-10', name: 'Day ten', abstract_timeframe: 'Day 10' },
      { id: 'day-2', name: 'Day two', abstract_timeframe: 'Day 2' },
      { id: 'week-2', name: 'Week two', abstract_timeframe: 'Week 2' },
      { id: 'day-8', name: 'Day eight', abstract_timeframe: 'Day 8' },
    ]);

    expect(sorted.map((node) => node.id)).toEqual(['day-2', 'day-8', 'week-2', 'day-10']);
  });

  it('preserves every nested node and its metadata while sorting each sibling group', () => {
    const project: FileNode[] = [
      {
        id: 'section',
        name: 'Section',
        filename: 'section.md',
        abstract_timeframe: 'Year 2',
        children: [
          { id: 'nested-unassigned', name: 'Untimed nested', filename: 'nested.md' },
          {
            id: 'nested-day-10',
            name: 'Nested day ten',
            filename: 'day-10.md',
            abstract_timeframe: 'Day 10',
            duration: '2 hours',
          },
          {
            id: 'nested-day-2',
            name: 'Nested day two',
            filename: 'day-2.md',
            abstract_timeframe: 'Day 2',
          },
        ],
      },
      { id: 'root-unassigned', name: 'Untimed root', filename: 'untimed.md' },
      { id: 'root-day-10', name: 'Day ten', filename: 'root-10.md', abstract_timeframe: 'Day 10' },
      { id: 'root-day-2', name: 'Day two', filename: 'root-2.md', abstract_timeframe: 'Day 2' },
    ];

    const sorted = sortNodesChronologically(project);
    const ids = (nodes: FileNode[]): string[] =>
      nodes.flatMap((node) => [node.id, ...(node.children ? ids(node.children) : [])]);

    expect(sorted.map((node) => node.id)).toEqual([
      'root-day-2',
      'root-day-10',
      'section',
      'root-unassigned',
    ]);
    expect(sorted[2].children?.map((node) => node.id)).toEqual([
      'nested-day-2',
      'nested-day-10',
      'nested-unassigned',
    ]);
    expect(sorted[2].children?.[1]).toMatchObject({
      filename: 'day-10.md',
      duration: '2 hours',
    });
    expect(ids(sorted).sort()).toEqual(ids(project).sort());
    expect(project[0].children?.[0].id).toBe('nested-unassigned');
  });
});
