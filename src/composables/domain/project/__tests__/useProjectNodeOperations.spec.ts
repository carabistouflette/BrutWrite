import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import type { Chapter, FileNode, ProjectMetadata } from '../../../../types';
import { chaptersApi } from '../../../../api/chapters';
import { projectApi } from '../../../../api/project';
import { useProjectNodeOperations } from '../useProjectNodeOperations';
import { useProjectSync } from '../useProjectSync';
import { useProjectStore } from '../../../../stores/project';

vi.mock('../../../../api/chapters', () => ({
  chaptersApi: {
    createNode: vi.fn(),
    deleteNode: vi.fn(),
    updateNodeMetadata: vi.fn(),
  },
}));

vi.mock('../../../../api/project', () => ({
  projectApi: {
    updateManifest: vi.fn(),
  },
}));

const chapter = (id: string, title: string, order: number, parent_id?: string): Chapter => ({
  id,
  parent_id,
  title,
  filename: `${id}.md`,
  word_count: 0,
  order,
});

const metadata = (chapters: Chapter[]): ProjectMetadata =>
  ({
    id: 'p1',
    title: 'Project',
    author: 'Author',
    created_at: '',
    updated_at: '',
    settings: { daily_target: 0, word_target: 0 },
    manifest: { chapters },
    characters: [],
    plotlines: [],
  }) as ProjectMetadata;

const fileNode = (id: string, name: string, children: FileNode[] = []): FileNode => ({
  id,
  name,
  children,
});

const settings = { daily_target: 0, word_target: 0 };

describe('useProjectNodeOperations', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.useFakeTimers();
    vi.mocked(projectApi.updateManifest).mockResolvedValue({} as ProjectMetadata);
    vi.mocked(chaptersApi.createNode).mockResolvedValue(metadata([]));
    vi.mocked(chaptersApi.deleteNode).mockResolvedValue(metadata([]));
    vi.mocked(chaptersApi.updateNodeMetadata).mockResolvedValue(metadata([]));
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.clearAllMocks();
  });

  it('persists a reordered and nested structure in visible preorder', async () => {
    const store = useProjectStore();
    store.setProjectData(
      'p1',
      '/project.json',
      [fileNode('chapter-a', 'Chapter A'), fileNode('chapter-b', 'Chapter B')],
      settings
    );

    const { updateStructure } = useProjectNodeOperations();
    const { flush } = useProjectSync();
    await updateStructure([
      fileNode('chapter-b', 'Chapter B', [fileNode('section-a', 'Section A')]),
      fileNode('chapter-a', 'Chapter A'),
    ]);
    await flush();

    const [, manifest] = vi.mocked(projectApi.updateManifest).mock.calls[0];
    expect(manifest.chapters.map(({ id, parent_id, order }) => [id, parent_id, order])).toEqual([
      ['chapter-b', undefined, 0],
      ['section-a', 'chapter-b', 1],
      ['chapter-a', undefined, 2],
    ]);
  });

  it('adds a chapter at the root without replacing the existing tree', async () => {
    const store = useProjectStore();
    store.setProjectData(
      'p1',
      '/project.json',
      [fileNode('first', 'First'), fileNode('second', 'Second')],
      settings
    );
    vi.mocked(chaptersApi.createNode).mockResolvedValue(
      metadata([
        chapter('first', 'First', 0),
        chapter('second', 'Second', 1),
        chapter('new', 'New Chapter', 2),
      ])
    );

    const { addChapter } = useProjectNodeOperations();
    await expect(addChapter()).resolves.toBe('new');

    expect(store.nodes.map(({ id }) => id)).toEqual(['first', 'second', 'new']);
    expect(store.activeId).toBe('new');
    await useProjectSync().flush();
  });

  it('adds sections into the live tree and deletes their full subtree with a valid selection', async () => {
    const store = useProjectStore();
    store.setProjectData(
      'p1',
      '/project.json',
      [fileNode('root', 'Root', [fileNode('child', 'Child')]), fileNode('other', 'Other')],
      settings
    );
    let createStarted = false;
    vi.mocked(chaptersApi.createNode).mockImplementation(() => {
      createStarted = true;
      return Promise.resolve(
        metadata([
          chapter('root', 'Root', 0),
          chapter('child', 'Child', 1, 'root'),
          chapter('other', 'Other', 2),
          chapter('section', 'New Section', 3, 'root'),
        ])
      );
    });

    const { addSection, deleteNode, updateStructure } = useProjectNodeOperations();
    const addingSection = addSection('root');
    await Promise.resolve();
    expect(createStarted).toBe(true);
    await updateStructure([
      fileNode('other', 'Other'),
      fileNode('root', 'Locally Edited Root', [fileNode('child', 'Locally Edited Child')]),
    ]);
    await expect(addingSection).resolves.toBe('section');
    expect(store.nodes.map(({ id }) => id)).toEqual(['other', 'root']);
    expect(store.nodes[1].name).toBe('Locally Edited Root');
    expect(store.nodes[1].children?.map(({ id }) => id)).toEqual(['child', 'section']);
    expect(store.nodes[1].children?.[0].name).toBe('Locally Edited Child');
    expect(store.activeId).toBe('section');
    store.updateNodeMetadataAction('other', { depends_on: 'child' });

    await deleteNode('root');
    await useProjectSync().flush();

    expect(store.nodes.map(({ id }) => id)).toEqual(['other']);
    expect(store.activeId).toBe('other');
    expect(store.nodeMap.get('other')?.depends_on).toBeUndefined();
    expect(chaptersApi.deleteNode).toHaveBeenCalledWith('p1', 'root');
  });

  it('flushes queued metadata to its original project and sends explicit null for clears', async () => {
    const store = useProjectStore();
    store.setProjectData(
      'p1',
      '/p1.json',
      [
        {
          id: 'chapter',
          name: 'Original name',
          chronological_date: '2025-01-01',
          children: [],
        },
      ],
      settings
    );

    const { renameNode, updateNodeTemporal } = useProjectNodeOperations();
    await renameNode('chapter', 'Renamed');
    updateNodeTemporal('chapter', { chronological_date: undefined });

    store.setProjectData('p2', '/p2.json', [fileNode('other', 'Other')], settings);
    const { flush } = useProjectSync();
    await flush('p2');
    expect(chaptersApi.updateNodeMetadata).not.toHaveBeenCalled();

    await flush('p1');

    expect(chaptersApi.updateNodeMetadata).toHaveBeenCalledWith('p1', 'chapter', {
      title: 'Renamed',
      chronological_date: null,
    });
    expect(store.projectId).toBe('p2');
  });
});
