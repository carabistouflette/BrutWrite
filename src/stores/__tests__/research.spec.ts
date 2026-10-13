import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { useResearchStore } from '../research';
import type { ResearchArtifact } from '../../api/research';

const mocks = vi.hoisted(() => ({
  fetchArtifacts: vi.fn(),
  notifyError: vi.fn(),
}));

vi.mock('../../api/research', () => ({
  researchApi: { fetchArtifacts: mocks.fetchArtifacts },
}));

vi.mock('../../composables/ui/useAppStatus', () => ({
  useAppStatus: () => ({ notifyError: mocks.notifyError }),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((resolvePromise) => {
    resolve = resolvePromise;
  });
  return { promise, resolve };
}

describe('research store project isolation', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mocks.fetchArtifacts.mockReset();
    mocks.notifyError.mockReset();
  });

  it('ignores an older project fetch after the current project has loaded', async () => {
    const oldProjectFetch = deferred<ResearchArtifact[]>();
    const currentProjectArtifact: ResearchArtifact = {
      id: 'current',
      path: '/current/research/source.md',
      name: 'source.md',
      file_type: 'text',
      tags: [],
    };
    const previousProjectArtifact: ResearchArtifact = {
      id: 'previous',
      path: '/previous/research/source.md',
      name: 'source.md',
      file_type: 'text',
      tags: [],
    };
    mocks.fetchArtifacts
      .mockReturnValueOnce(oldProjectFetch.promise)
      .mockResolvedValueOnce([currentProjectArtifact]);

    const store = useResearchStore();
    const oldRequest = store.fetchArtifacts();
    store.setActiveArtifact(previousProjectArtifact);
    await store.fetchArtifacts();

    oldProjectFetch.resolve([previousProjectArtifact]);
    await oldRequest;

    expect(store.artifacts).toEqual([currentProjectArtifact]);
    expect(store.activeArtifact).toBeNull();
  });
});
