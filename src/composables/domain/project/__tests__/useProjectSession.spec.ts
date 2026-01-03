// @vitest-environment happy-dom
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { nextTick } from 'vue';
import { useProjectStore } from '../../../../stores/project';
import { useProjectSession } from '../useProjectSession';
import { APP_CONSTANTS } from '../../../../config/constants';

describe('useProjectSession Auto-Save', () => {
  let setItemSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    setActivePinia(createPinia());
    vi.useFakeTimers();

    // Mock localStorage completely using stubGlobal
    const store: Record<string, string> = {};

    const mockLocalStorage = {
      getItem: vi.fn((key: string) => store[key] || null),
      setItem: vi.fn((key: string, value: string) => {
        store[key] = value + '';
      }),
      removeItem: vi.fn((key: string) => {
        delete store[key];
      }),
      clear: vi.fn(() => {
        Object.keys(store).forEach((key) => delete store[key]);
      }),
      key: vi.fn(),
      length: 0,
    };

    vi.stubGlobal('localStorage', mockLocalStorage);
    setItemSpy = mockLocalStorage.setItem;
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('auto-saves after debounce when state changes', async () => {
    const projectStore = useProjectStore();
    const { setupAutoSave } = useProjectSession();

    // Setup initial state with a valid path
    projectStore.setProjectData('p1', '/path/to/p1', [], { daily_target: 0, word_target: 0 });

    // Initialize watcher
    setupAutoSave();

    // Trigger change
    if (projectStore.settings) {
      projectStore.settings.daily_target = 100;
    }

    await nextTick();

    // Should NOT have saved yet (debounce)
    expect(setItemSpy).not.toHaveBeenCalled();

    // Fast forward part way
    vi.advanceTimersByTime(APP_CONSTANTS.CACHE.DEBOUNCE_MS / 2);
    expect(setItemSpy).not.toHaveBeenCalled();

    // Complete debounce
    vi.advanceTimersByTime(APP_CONSTANTS.CACHE.DEBOUNCE_MS / 2 + 10);

    const expectedKey = `${APP_CONSTANTS.CACHE.KEY_PREFIX}/path/to/p1`;
    expect(setItemSpy).toHaveBeenCalledWith(
      expectedKey,
      expect.stringContaining('"daily_target":100')
    );
  });

  it('does not save if path is missing', async () => {
    const projectStore = useProjectStore();
    const { setupAutoSave } = useProjectSession();

    // Setup state WITHOUT path
    projectStore.setProjectData('p1', '/path/to/p1', [], { daily_target: 0, word_target: 0 });
    projectStore.path = undefined;

    setupAutoSave();

    if (projectStore.settings) {
      projectStore.settings.daily_target = 999;
    }

    await nextTick();

    vi.advanceTimersByTime(APP_CONSTANTS.CACHE.DEBOUNCE_MS + 100);

    expect(setItemSpy).not.toHaveBeenCalled();
  });

  it('updates cache when nodes change', async () => {
    const projectStore = useProjectStore();
    const { setupAutoSave } = useProjectSession();

    projectStore.setProjectData('p1', '/path/to/p1', [], { daily_target: 0, word_target: 0 });
    setupAutoSave();

    // Use updateStructure which triggers the action subscription
    projectStore.updateStructure([{ id: 'n1', name: 'New Node', children: [] }]);

    await nextTick();

    vi.advanceTimersByTime(APP_CONSTANTS.CACHE.DEBOUNCE_MS + 100);

    // CRITICAL: We should NOT save nodes to localStorage (per optimization)
    // This test verifies that even when nodes UPDATE, they are NOT saved
    expect(setItemSpy).toHaveBeenCalled(); // Still called because action triggers
    const savedData = JSON.parse(setItemSpy.mock.calls[0][1]);
    expect(savedData.nodes).toBeUndefined(); // Nodes should NOT be saved
  });

  it('updates cache when activeId changes', async () => {
    const projectStore = useProjectStore();
    const { setupAutoSave } = useProjectSession();

    projectStore.setProjectData('p1', '/path/to/p1', [{ id: 'n1', name: 'N1' }], {
      daily_target: 0,
      word_target: 0,
    });
    setupAutoSave();

    projectStore.setActiveId('n1');

    await nextTick();

    vi.advanceTimersByTime(APP_CONSTANTS.CACHE.DEBOUNCE_MS + 100);

    const expectedKey = `${APP_CONSTANTS.CACHE.KEY_PREFIX}/path/to/p1`;
    expect(setItemSpy).toHaveBeenCalledWith(
      expectedKey,
      expect.stringContaining('"activeId":"n1"')
    );
  });
});
