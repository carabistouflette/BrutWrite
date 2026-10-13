import { effectScope, nextTick, ref } from 'vue';
import { flushPromises } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { chaptersApi } from '../../../../api/chapters';
import {
  flushChapterSessions,
  registerChapterSession,
  useChapterSession,
} from '../useChapterSession';

vi.mock('../../../../api/chapters', () => ({
  chaptersApi: { loadContent: vi.fn() },
}));

const loadContentMock = vi.mocked(chaptersApi.loadContent);

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((resolvePromise) => {
    resolve = resolvePromise;
  });
  return { promise, resolve };
}

describe('useChapterSession', () => {
  beforeEach(() => loadContentMock.mockReset());

  it('ignores a late chapter load after the selection changes', async () => {
    const firstLoad = deferred<string>();
    const secondLoad = deferred<string>();
    loadContentMock.mockImplementation((_projectId, id) =>
      id === 'chapter-1' ? firstLoad.promise : secondLoad.promise
    );
    const projectId = ref('project-1');
    const chapterId = ref('chapter-1');
    const scope = effectScope();
    const session = scope.run(() => useChapterSession(projectId, chapterId))!;
    await flushPromises();

    chapterId.value = 'chapter-2';
    await nextTick();
    await flushPromises();
    firstLoad.resolve('<p>Stale chapter</p>');
    secondLoad.resolve('<p>Selected chapter</p>');
    await flushPromises();

    expect(session.content.value).toBe('<p>Selected chapter</p>');
    expect(session.loadedChapterId.value).toBe('chapter-2');
    scope.stop();
  });

  it('cancels loading when saving the previous session fails', async () => {
    const beforeLoad = vi.fn().mockResolvedValueOnce(true).mockResolvedValueOnce(false);
    loadContentMock.mockResolvedValue('<p>Current chapter</p>');
    const projectId = ref('project-1');
    const chapterId = ref('chapter-1');
    const scope = effectScope();
    const session = scope.run(() => useChapterSession(projectId, chapterId, { beforeLoad }))!;
    await flushPromises();

    chapterId.value = 'chapter-2';
    await nextTick();
    await flushPromises();

    expect(beforeLoad).toHaveBeenNthCalledWith(1, 'project-1', 'chapter-1');
    expect(beforeLoad).toHaveBeenNthCalledWith(2, 'project-1', 'chapter-2');
    expect(loadContentMock).toHaveBeenCalledOnce();
    expect(session.content.value).toBe('<p>Current chapter</p>');
    expect(session.loadedChapterId.value).toBe('chapter-1');
    expect(session.isLoading.value).toBe(false);
    scope.stop();
  });

  it('awaits registered editor sessions and propagates save failures', async () => {
    const projectStore = {};
    const flush = vi.fn().mockRejectedValue(new Error('save failed'));
    const unregister = registerChapterSession(projectStore, flush);

    await expect(flushChapterSessions(projectStore)).rejects.toThrow('save failed');
    expect(flush).toHaveBeenCalledOnce();
    unregister();
    await flushChapterSessions(projectStore);
    expect(flush).toHaveBeenCalledOnce();
  });
});
