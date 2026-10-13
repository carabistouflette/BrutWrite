import { defineComponent, h, ref, type Ref } from 'vue';
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useProjectStore } from '../../../stores/project';
import { useEditorSession, type EditorSession } from '../useEditorSession';
import { flushChapterSessions } from '../../domain/project/useChapterSession';
import type { ProjectMetadata } from '../../../types';

const sessionMocks = vi.hoisted(() => ({
  loadContent: vi.fn(),
  saveContent: vi.fn(),
  restoreSnapshot: vi.fn(),
  branchSnapshot: vi.fn(),
  flushProjectSync: vi.fn(),
  updateNodeStats: vi.fn(),
  renameNode: vi.fn(),
  addWords: vi.fn(),
  analyze: vi.fn(),
  notifyError: vi.fn(),
}));

vi.mock('../../../api/chapters', () => ({
  chaptersApi: {
    loadContent: sessionMocks.loadContent,
    saveContent: sessionMocks.saveContent,
  },
}));
vi.mock('../../../stores/snapshots', () => ({
  useSnapshotStore: () => ({
    restoreSnapshot: sessionMocks.restoreSnapshot,
    branchSnapshot: sessionMocks.branchSnapshot,
  }),
}));
vi.mock('../../domain/project/useProjectNodeOperations', () => ({
  useProjectNodeOperations: () => ({
    updateNodeStats: sessionMocks.updateNodeStats,
    renameNode: sessionMocks.renameNode,
  }),
}));
vi.mock('../../domain/project/useProjectSync', () => ({
  useProjectSync: () => ({ flush: sessionMocks.flushProjectSync }),
}));
vi.mock('../../domain/gamification/useGamification', () => ({
  useGamification: () => ({ addWords: sessionMocks.addWords }),
}));
vi.mock('../../domain/intelligence/useCharacterGraph', () => ({
  useCharacterGraph: () => ({ analyze: sessionMocks.analyze }),
}));
vi.mock('../../ui/useAppStatus', () => ({
  useAppStatus: () => ({ notifyError: sessionMocks.notifyError }),
}));
vi.mock('../useAutoSave', () => ({ useAutoSave: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

describe('useEditorSession', () => {
  let wrapper: VueWrapper | undefined;
  let session: EditorSession | undefined;

  function mountEditorSession(projectId: Ref<string>, chapterId: Ref<string>): VueWrapper {
    return mount(
      defineComponent({
        setup() {
          session = useEditorSession(projectId, chapterId);
          return () => h('div');
        },
      })
    );
  }

  beforeEach(() => {
    setActivePinia(createPinia());
    sessionMocks.loadContent
      .mockReset()
      .mockImplementation(async (_projectId, chapterId) =>
        chapterId === 'chapter-1' ? '<p>Original</p>' : '<p>Second chapter</p>'
      );
    sessionMocks.saveContent.mockReset().mockResolvedValue({
      manifest: {
        chapters: [
          { id: 'chapter-1', word_count: 2 },
          { id: 'chapter-2', word_count: 2 },
        ],
      },
    } as ProjectMetadata);
    sessionMocks.restoreSnapshot.mockReset().mockResolvedValue('');
    sessionMocks.branchSnapshot.mockReset().mockResolvedValue(undefined);
    sessionMocks.flushProjectSync.mockReset().mockResolvedValue(undefined);
    sessionMocks.updateNodeStats.mockReset();
    sessionMocks.renameNode.mockReset();
    sessionMocks.addWords.mockReset();
    sessionMocks.analyze.mockReset();
    sessionMocks.notifyError.mockReset();
    wrapper = undefined;
    session = undefined;
  });

  afterEach(() => {
    wrapper?.unmount();
    wrapper = undefined;
  });

  it('saves the previous dirty chapter before loading the newly selected chapter', async () => {
    const projectStore = useProjectStore();
    projectStore.setProjectData(
      'project-1',
      '/project',
      [
        { id: 'chapter-1', name: 'Original', filename: 'chapter-1.md', word_count: 1 },
        { id: 'chapter-2', name: 'Second', filename: 'chapter-2.md', word_count: 2 },
      ],
      { daily_target: 500, word_target: 50000 }
    );
    projectStore.setActiveId('chapter-1');
    const projectId = ref('project-1');
    const chapterId = ref('chapter-1');

    wrapper = mountEditorSession(projectId, chapterId);
    await flushPromises();

    session!.currentHtml.value = '<p>Immediate switch must preserve this text</p>';
    session!.isDirty.value = true;
    projectStore.setActiveId('chapter-2');
    chapterId.value = 'chapter-2';
    await flushPromises();

    expect(sessionMocks.saveContent).toHaveBeenCalledWith(
      'project-1',
      'chapter-1',
      '<p>Immediate switch must preserve this text</p>'
    );
    expect(session!.currentHtml.value).toBe('<p>Second chapter</p>');
    expect(session!.isDirty.value).toBe(false);
  });

  it('awaits the active dirty chapter save when the project session is flushed', async () => {
    const projectStore = useProjectStore();
    projectStore.setProjectData(
      'project-1',
      '/project',
      [{ id: 'chapter-1', name: 'Original', filename: 'chapter-1.md', word_count: 1 }],
      { daily_target: 500, word_target: 50000 }
    );
    projectStore.setActiveId('chapter-1');
    const projectId = ref('project-1');
    const chapterId = ref('chapter-1');

    wrapper = mountEditorSession(projectId, chapterId);
    await flushPromises();

    session!.currentHtml.value = '<p>Save before close</p>';
    session!.isDirty.value = true;
    await flushChapterSessions(projectStore);

    expect(sessionMocks.saveContent).toHaveBeenCalledWith(
      'project-1',
      'chapter-1',
      '<p>Save before close</p>'
    );
    expect(session!.isDirty.value).toBe(false);
  });

  it('saves edits made during an in-flight save before reporting the session flushed', async () => {
    const projectStore = useProjectStore();
    projectStore.setProjectData(
      'project-1',
      '/project',
      [{ id: 'chapter-1', name: 'Original', filename: 'chapter-1.md', word_count: 1 }],
      { daily_target: 500, word_target: 50000 }
    );
    projectStore.setActiveId('chapter-1');
    const projectId = ref('project-1');
    const chapterId = ref('chapter-1');

    wrapper = mountEditorSession(projectId, chapterId);
    await flushPromises();

    let resolveFirstSave!: (metadata: ProjectMetadata) => void;
    sessionMocks.saveContent.mockImplementationOnce(
      () =>
        new Promise<ProjectMetadata>((resolve) => {
          resolveFirstSave = resolve;
        })
    );
    session!.currentHtml.value = '<p>First version</p>';
    session!.isDirty.value = true;
    const flushing = flushChapterSessions(projectStore);
    await flushPromises();
    expect(sessionMocks.saveContent).toHaveBeenCalledTimes(1);

    session!.currentHtml.value = '<p>Latest version typed during save</p>';
    session!.isDirty.value = true;
    resolveFirstSave({
      manifest: {
        chapters: [{ id: 'chapter-1', word_count: 2 }],
      },
    } as ProjectMetadata);
    await flushing;

    expect(sessionMocks.saveContent).toHaveBeenNthCalledWith(
      1,
      'project-1',
      'chapter-1',
      '<p>First version</p>'
    );
    expect(sessionMocks.saveContent).toHaveBeenNthCalledWith(
      2,
      'project-1',
      'chapter-1',
      '<p>Latest version typed during save</p>'
    );
    expect(session!.isDirty.value).toBe(false);
  });
  it('applies an empty restored snapshot and resets the local word count', async () => {
    const projectStore = useProjectStore();
    projectStore.setProjectData(
      'project-1',
      '/project',
      [{ id: 'chapter-1', name: 'Original', filename: 'chapter-1.md', word_count: 1 }],
      { daily_target: 500, word_target: 50000 }
    );
    projectStore.setActiveId('chapter-1');
    const projectId = ref('project-1');
    const chapterId = ref('chapter-1');

    wrapper = mountEditorSession(projectId, chapterId);
    await flushPromises();

    session!.currentHtml.value = '<p>Unsaved draft</p>';
    session!.isDirty.value = true;
    const restored = await session!.restoreSnapshot('empty.md');

    expect(sessionMocks.restoreSnapshot).toHaveBeenCalledWith('chapter-1', 'empty.md');
    expect(restored).toBe('');
    expect(session!.currentHtml.value).toBe('');
    expect(session!.isDirty.value).toBe(false);
    expect(sessionMocks.updateNodeStats).toHaveBeenLastCalledWith('chapter-1', 0, false);
  });

  it('does not apply a late restore result to a chapter selected while it was pending', async () => {
    const projectStore = useProjectStore();
    projectStore.setProjectData(
      'project-1',
      '/project',
      [
        { id: 'chapter-1', name: 'Original', filename: 'chapter-1.md', word_count: 1 },
        { id: 'chapter-2', name: 'Second', filename: 'chapter-2.md', word_count: 2 },
      ],
      { daily_target: 500, word_target: 50000 }
    );
    projectStore.setActiveId('chapter-1');
    const projectId = ref('project-1');
    const chapterId = ref('chapter-1');

    wrapper = mountEditorSession(projectId, chapterId);
    await flushPromises();

    let resolveRestore!: (content: string) => void;
    sessionMocks.restoreSnapshot.mockImplementationOnce(
      () =>
        new Promise<string>((resolve) => {
          resolveRestore = resolve;
        })
    );
    const restoring = session!.restoreSnapshot('late.md');
    await flushPromises();

    projectStore.setActiveId('chapter-2');
    chapterId.value = 'chapter-2';
    await flushPromises();
    resolveRestore('<p>Stale restored content</p>');

    expect(await restoring).toBeUndefined();
    await flushPromises();
    expect(session!.currentHtml.value).toBe('<p>Second chapter</p>');
  });

  it('does not start a branch for a chapter selected while flushing is pending', async () => {
    const projectStore = useProjectStore();
    projectStore.setProjectData(
      'project-1',
      '/project',
      [
        { id: 'chapter-1', name: 'Original', filename: 'chapter-1.md', word_count: 1 },
        { id: 'chapter-2', name: 'Second', filename: 'chapter-2.md', word_count: 2 },
      ],
      { daily_target: 500, word_target: 50000 }
    );
    projectStore.setActiveId('chapter-1');
    const projectId = ref('project-1');
    const chapterId = ref('chapter-1');

    wrapper = mountEditorSession(projectId, chapterId);
    await flushPromises();

    let resolveSync!: () => void;
    sessionMocks.flushProjectSync.mockImplementationOnce(
      () =>
        new Promise<void>((resolve) => {
          resolveSync = resolve;
        })
    );
    const branching = session!.branchSnapshot('branch.md');
    await flushPromises();

    projectStore.setActiveId('chapter-2');
    chapterId.value = 'chapter-2';
    resolveSync();
    await branching;
    await flushPromises();

    expect(sessionMocks.branchSnapshot).not.toHaveBeenCalled();
  });
});
