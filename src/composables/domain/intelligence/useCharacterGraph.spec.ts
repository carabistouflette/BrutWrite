import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { nextTick } from 'vue';
import { useProjectStore } from '../../../stores/project';
import type { CharacterGraphPayload } from '../../../types/intelligence';
import { useCharacterGraph } from './useCharacterGraph';

const mocks = vi.hoisted(() => ({ getCharacterGraph: vi.fn() }));

vi.mock('../../../api/intelligence', () => ({
  intelligenceApi: { getCharacterGraph: mocks.getCharacterGraph },
}));

function graphPayload(label: string): CharacterGraphPayload {
  return {
    nodes: [
      {
        id: label,
        label,
        valence: 1,
        mentionCount: 1,
        isMapped: true,
        firstMention: null,
      },
    ],
    edges: [],
    metrics: {
      networkDensity: 0,
      connectedComponents: 1,
      largestComponentSize: 1,
      isolationRatio: 0,
    },
  };
}

function loadProject(id: string) {
  const projectStore = useProjectStore();
  projectStore.setProjectData(id, `/${id}`, [], { daily_target: 0, word_target: 0 });
}

describe('character graph analysis', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    loadProject('project-1');
  });

  it('runs successive filtered analyses and keeps the newest result', async () => {
    const latestPayload = graphPayload('latest');
    mocks.getCharacterGraph
      .mockImplementationOnce(async () => {
        await Promise.resolve();
        await Promise.resolve();
        return graphPayload('stale');
      })
      .mockResolvedValueOnce(latestPayload);

    const graph = useCharacterGraph();
    const firstAnalysis = graph.analyze(['chapter-1']);
    const secondAnalysis = graph.analyze(['chapter-2']);

    expect(mocks.getCharacterGraph).toHaveBeenCalledTimes(2);
    expect(mocks.getCharacterGraph).toHaveBeenNthCalledWith(
      1,
      'project-1',
      expect.objectContaining({ chapterIds: ['chapter-1'] })
    );
    expect(mocks.getCharacterGraph).toHaveBeenNthCalledWith(
      2,
      'project-1',
      expect.objectContaining({ chapterIds: ['chapter-2'] })
    );

    await Promise.all([firstAnalysis, secondAnalysis]);

    expect(graph.payload).toEqual(latestPayload);
    expect(graph.isLoading).toBe(false);
  });

  it('clears project data and ignores an analysis that completes after a project switch', async () => {
    mocks.getCharacterGraph.mockResolvedValueOnce(graphPayload('old-project'));

    const graph = useCharacterGraph();
    const analysis = graph.analyze();
    loadProject('project-2');
    await nextTick();

    expect(graph.payload).toBeNull();
    expect(graph.isLoading).toBe(false);

    await analysis;
    expect(graph.payload).toBeNull();
  });
});
