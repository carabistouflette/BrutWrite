import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useProjectStore } from '../project';

describe('useProjectStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('initializes with empty state', () => {
    const store = useProjectStore();
    expect(store.nodes).toEqual([]);
    expect(store.activeId).toBeUndefined();
    expect(store.projectId).toBeUndefined();
  });

  it('sets project data correctly', () => {
    const store = useProjectStore();
    const mockNodes = [{ id: '1', name: 'Chapter 1', type: 'file' as const }];
    const mockSettings = {
      project: { name: 'Test Project', created: '', last_modified: '' },
      editor: {
        fontFamily: 'serif' as const,
        fontSize: 16,
        lineHeight: 1.6,
        maxWidth: 800,
        focusMode: false,
        typewriterMode: false,
      },
      general: {
        theme: 'system' as const,
        accentColor: 'default',
        reducedMotion: false,
        autoSaveInterval: 30,
      },
      daily_target: 500,
      word_target: 50000,
    };

    store.setProjectData('proj-123', mockNodes, mockSettings);

    expect(store.projectId).toBe('proj-123');
    expect(store.nodes).toHaveLength(1);
    expect(store.settings).toEqual(mockSettings);
  });
});
