// @vitest-environment happy-dom
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { nextTick } from 'vue';
import { useProjectStore } from '../../../stores/project';
import { useProjectSession } from '../useProjectSession';

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
    vi.advanceTimersByTime(1000);
    expect(setItemSpy).not.toHaveBeenCalled();

    // Complete debounce
    vi.advanceTimersByTime(1001);

    expect(setItemSpy).toHaveBeenCalledWith(
      'project_session_/path/to/p1',
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

    vi.advanceTimersByTime(3000);

    expect(setItemSpy).not.toHaveBeenCalled();
  });

  it('updates cache when nodes change', async () => {
    const projectStore = useProjectStore();
    const { setupAutoSave } = useProjectSession();

    projectStore.setProjectData('p1', '/path/to/p1', [], { daily_target: 0, word_target: 0 });
    setupAutoSave();

    // Add a node
    projectStore.nodes.push({ id: 'n1', name: 'New Node' });

    await nextTick();

    vi.advanceTimersByTime(2500);

    expect(setItemSpy).toHaveBeenCalledWith(
      'project_session_/path/to/p1',
      expect.stringContaining('New Node')
    );
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

    vi.advanceTimersByTime(2500);

    expect(setItemSpy).toHaveBeenCalledWith(
      'project_session_/path/to/p1',
      expect.stringContaining('"activeId":"n1"')
    );
  });
});
