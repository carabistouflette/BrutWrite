import { computed } from 'vue';
import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useProjectStore } from '../project';

describe('project chapter indexes', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it.each(['load', 'replace structure'])(
    'keeps cached editor and timeline views reactive after %s',
    (operation) => {
      const store = useProjectStore();
      const nodes = [
        {
          id: 'parent',
          name: 'Parent',
          filename: 'parent.md',
          word_count: 2,
          children: [{ id: 'child', name: 'Scene', filename: 'child.md', word_count: 0 }],
        },
      ];
      if (operation === 'load') {
        store.setProjectData('project', '/project', nodes, {
          daily_target: 500,
          word_target: 50000,
        });
      } else {
        store.updateStructure(nodes);
      }
      store.setActiveId('child');

      const title = computed(() => store.activeChapter?.name);
      const sceneDate = computed(
        () => store.flatNodes.find((node) => node.id === 'child')?.chronological_date
      );
      const indexedWords = computed(() =>
        store.flatNodes.reduce((total, node) => total + (node.word_count || 0), 0)
      );
      expect(title.value).toBe('Scene');
      expect(sceneDate.value).toBeUndefined();
      expect(indexedWords.value).toBe(2);

      store.renameNodeAction('child', 'Revised scene');
      store.updateNodeMetadataAction('child', { chronological_date: '2026-01-02T00:00:00Z' });
      store.updateNodeStatsAction('child', 3);

      expect(title.value).toBe('Revised scene');
      expect(sceneDate.value).toBe('2026-01-02T00:00:00Z');
      expect(indexedWords.value).toBe(5);
      expect(store.totalWordCount).toBe(5);
    }
  );
});
