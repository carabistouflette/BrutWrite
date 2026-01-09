import { ref, watch, computed, onBeforeUnmount, type Ref } from 'vue';
import { storeToRefs } from 'pinia';
import { useProjectStore } from '../../stores/project';
import { useSettingsStore } from '../../stores/settings';
import { useResearchStore } from '../../stores/research';
import { useSnapshotStore } from '../../stores/snapshots';
import { useProjectNodeOperations } from '../domain/project/useProjectNodeOperations';
import { useGamification } from '../domain/gamification/useGamification';
import { useCharacterGraph } from '../domain/intelligence/useCharacterGraph';
import { useAppStatus } from '../ui/useAppStatus';
import { useAutoSave } from '../../composables/editor/useAutoSave';
import { useChapterSession } from '../domain/project/useChapterSession';
import { chaptersApi } from '../../api/chapters';
import { countWords } from '../../utils/stats';
import { APP_CONSTANTS } from '../../config/constants';
import type { Chapter } from '../../types';

export function useEditorSession(projectId: Ref<string>, chapterId: Ref<string>) {
  // Stores
  const projectStore = useProjectStore();
  const settingsStore = useSettingsStore();
  const researchStore = useResearchStore();
  const snapshotStore = useSnapshotStore();

  const { activeId, nodeMap } = storeToRefs(projectStore);
  const { settings } = storeToRefs(settingsStore);

  // Composables
  const { updateNodeStats, renameNode } = useProjectNodeOperations();
  const { addWords } = useGamification();
  const { notifyError } = useAppStatus();
  const { analyze: analyzeCharacterGraph } = useCharacterGraph();

  // Session State
  const activeChapter = computed(() => nodeMap.value.get(chapterId.value));
  const currentHtml = ref('');
  const isDirty = ref(false);

  // 1. Session Loading
  const { content: loadedContent, isLoading } = useChapterSession(projectId, chapterId);

  // Sync loaded content to local state
  watch(loadedContent, (newVal) => {
    currentHtml.value = newVal;
    isDirty.value = false;
  });

  // 2. Core Actions
  const handleRename = async (newName: string) => {
    if (chapterId.value && newName !== activeChapter.value?.name) {
      await renameNode(chapterId.value, newName);
    }
  };

  const handleContentChange = (delta: number) => {
    addWords(delta);
    if (chapterId.value) {
      const currentWordCount = activeChapter.value?.word_count || 0;
      updateNodeStats(chapterId.value, currentWordCount + delta);
    }
  };

  const saveActiveChapter = async (content: string) => {
    if (!chapterId.value || !projectId.value) return;

    try {
      const metadata = await chaptersApi.saveContent(projectId.value, chapterId.value, content);

      // Sync word count from backend response to be sure
      if (activeId.value === chapterId.value) {
        const chapter = metadata.manifest.chapters.find((c: Chapter) => c.id === chapterId.value);
        if (chapter) {
          updateNodeStats(chapterId.value, chapter.word_count, false);
        }
      }

      // Intelligence Hook
      if (settings.value.intelligence.autoAnalyzeOnSave) {
        analyzeCharacterGraph();
      }

      // Chapter auto-saved
    } catch (e) {
      notifyError(`Failed to save chapter ${chapterId.value}`, e);
      throw e;
    }
  };

  // 3. Auto-Save Logic
  const autoSaveInterval = computed(
    () => settings.value.general.autoSaveInterval || APP_CONSTANTS.EDITOR.AUTO_SAVE_INTERVAL
  );

  useAutoSave(async () => {
    if (isDirty.value) {
      await saveActiveChapter(currentHtml.value);
      isDirty.value = false;
    }
  }, autoSaveInterval);

  // Manual Save
  const handleSave = async (content: string) => {
    await saveActiveChapter(content);
    isDirty.value = false;
  };

  // Snapshot Restoration
  const restoreSnapshot = async (filename: string) => {
    if (!chapterId.value) return;
    try {
      const newContent = await snapshotStore.restoreSnapshot(chapterId.value, filename);
      if (newContent !== undefined) {
        currentHtml.value = newContent;
        isDirty.value = false;

        const wc = countWords(newContent);
        updateNodeStats(chapterId.value, wc);
        return newContent;
      }
    } catch (e) {
      notifyError('Failed to restore snapshot', e);
      throw e;
    }
  };

  const branchSnapshot = async (filename: string) => {
    if (!projectId.value || !chapterId.value) return;
    try {
      await snapshotStore.branchSnapshot(chapterId.value, filename);
    } catch (e) {
      notifyError('Failed to branch snapshot', e);
      throw e;
    }
  };

  // Research Navigation
  const openResearchArtifact = (id: string) => {
    const artifact = researchStore.artifacts.find((a) => a.id === id);
    if (artifact) {
      researchStore.setActiveArtifact(artifact);
    } else {
      researchStore.fetchArtifacts().then(() => {
        const found = researchStore.artifacts.find((a) => a.id === id);
        if (found) researchStore.setActiveArtifact(found);
      });
    }
  };

  // Lifecycle
  onBeforeUnmount(async () => {
    if (isDirty.value) {
      await saveActiveChapter(currentHtml.value);
    }
  });

  return {
    // State
    activeChapter,
    currentHtml,
    isLoading,
    isDirty,

    // Actions
    handleRename,
    handleContentChange,
    handleSave,
    restoreSnapshot,
    branchSnapshot,
    openResearchArtifact,
  };
}
