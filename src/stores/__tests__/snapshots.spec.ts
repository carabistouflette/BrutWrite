import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { useProjectStore } from '../project';
import { useSnapshotStore } from '../snapshots';
import type { ProjectMetadata } from '../../types';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const invokeMock = vi.mocked(invoke);

function makeBranchMetadata(): ProjectMetadata {
  return {
    id: 'project-1',
    title: 'Project',
    author: 'Author',
    created_at: '',
    updated_at: '',
    settings: { daily_target: 500, word_target: 50000 },
    manifest: {
      chapters: [
        { id: 'chapter-1', title: 'Original', filename: 'chapter-1.md', word_count: 3, order: 0 },
        {
          id: 'chapter-2',
          title: 'Branch from snapshot',
          filename: 'chapter-2.md',
          word_count: 2,
          order: 1,
        },
      ],
    },
    characters: [],
    plotlines: [],
  };
}

describe('useSnapshotStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
  });

  it('applies returned branch metadata and selects the created chapter', async () => {
    const projectStore = useProjectStore();
    projectStore.setProjectData(
      'project-1',
      '/project',
      [{ id: 'chapter-1', name: 'Original', filename: 'chapter-1.md', word_count: 3 }],
      { daily_target: 500, word_target: 50000 }
    );
    projectStore.setActiveId('chapter-1');
    const metadata = makeBranchMetadata();
    invokeMock.mockResolvedValue(metadata);

    await useSnapshotStore().branchSnapshot('chapter-1', 'snapshot.md');

    expect(projectStore.nodes.map((node) => node.name)).toEqual([
      'Original',
      'Branch from snapshot',
    ]);
    expect(projectStore.activeId).toBe('chapter-2');
  });

  it('returns empty snapshot content as a valid restore result', async () => {
    const projectStore = useProjectStore();
    projectStore.setProjectData(
      'project-1',
      '/project',
      [{ id: 'chapter-1', name: 'Original', filename: 'chapter-1.md', word_count: 3 }],
      { daily_target: 500, word_target: 50000 }
    );
    invokeMock.mockResolvedValueOnce('').mockResolvedValueOnce([]);

    const restored = await useSnapshotStore().restoreSnapshot('chapter-1', 'empty.md');

    expect(restored).toBe('');
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'restore_snapshot', {
      projectId: 'project-1',
      chapterId: 'chapter-1',
      snapshotFilename: 'empty.md',
    });
  });
});
