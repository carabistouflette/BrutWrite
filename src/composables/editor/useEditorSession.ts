import { ref, watch, computed, onBeforeUnmount, type ComputedRef, type Ref } from 'vue';
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
import { registerChapterSession, useChapterSession } from '../domain/project/useChapterSession';
import { useProjectSync } from '../domain/project/useProjectSync';
import { researchApi } from '../../api/research';
import { chaptersApi } from '../../api/chapters';
import { countWords } from '../../utils/stats';
import { APP_CONSTANTS } from '../../config/constants';
import type { Chapter, FileNode } from '../../types';

export interface EditorSession {
  activeChapter: ComputedRef<FileNode | undefined>;
  currentHtml: Ref<string>;
  isLoading: Ref<boolean>;
  isDirty: Ref<boolean>;
  handleRename: (newName: string) => Promise<void>;
  handleContentChange: (delta: number) => void;
  handleSave: (content: string) => Promise<void>;
  restoreSnapshot: (filename: string) => Promise<string | undefined>;
  branchSnapshot: (filename: string) => Promise<void>;
  openResearchArtifact: (id: string) => Promise<void>;
}

export function useEditorSession(projectId: Ref<string>, chapterId: Ref<string>): EditorSession {
  const projectStore = useProjectStore();
  const settingsStore = useSettingsStore();
  const researchStore = useResearchStore();
  const snapshotStore = useSnapshotStore();
  const { nodeMap } = storeToRefs(projectStore);
  const { settings } = storeToRefs(settingsStore);

  const { updateNodeStats, renameNode } = useProjectNodeOperations();
  const { addWords } = useGamification();
  const { notifyError } = useAppStatus();
  const { analyze: analyzeCharacterGraph } = useCharacterGraph();
  const { flush: flushProjectSync } = useProjectSync();

  const activeChapter = computed(() => nodeMap.value.get(chapterId.value));
  const currentHtml = ref('');
  const isDirty = ref(false);
  const snapshotOperationBusy = ref(false);
  let loadedIdentity: { projectId: string; chapterId: string } | null = null;
  let saveQueue = Promise.resolve();

  const saveActiveChapter = (
    content: string,
    targetProjectId = projectId.value,
    targetChapterId = chapterId.value
  ): Promise<void> => {
    if (!targetProjectId || !targetChapterId) return Promise.resolve();

    const save = saveQueue
      .catch(() => {})
      .then(async () => {
        try {
          if (projectStore.projectId === targetProjectId) await flushProjectSync();
          const metadata = await chaptersApi.saveContent(targetProjectId, targetChapterId, content);

          if (projectStore.projectId === targetProjectId && nodeMap.value.has(targetChapterId)) {
            const chapter = metadata.manifest.chapters.find(
              (item: Chapter) => item.id === targetChapterId
            );
            if (chapter) updateNodeStats(targetChapterId, chapter.word_count, false);
          }

          if (
            projectStore.projectId === targetProjectId &&
            settings.value.intelligence.autoAnalyzeOnSave
          ) {
            analyzeCharacterGraph();
          }

          if (
            projectId.value === targetProjectId &&
            chapterId.value === targetChapterId &&
            currentHtml.value === content
          ) {
            isDirty.value = false;
          }
        } catch (error) {
          notifyError(`Failed to save chapter ${targetChapterId}`, error);
          throw error;
        }
      });

    saveQueue = save;
    return save;
  };
  let currentFlush: Promise<void> | undefined;
  const flushCurrentSession = (): Promise<void> => {
    if (currentFlush) return currentFlush;

    const flush = (async () => {
      while (isDirty.value) {
        const identity = loadedIdentity;
        if (!identity) return;
        const content = currentHtml.value;
        await saveActiveChapter(content, identity.projectId, identity.chapterId);
        if (
          loadedIdentity?.projectId === identity.projectId &&
          loadedIdentity.chapterId === identity.chapterId &&
          currentHtml.value === content
        ) {
          isDirty.value = false;
        }
      }
    })();
    currentFlush = flush.finally(() => {
      currentFlush = undefined;
    });
    return currentFlush;
  };

  const chapterSession = useChapterSession(projectId, chapterId, {
    beforeLoad: async (nextProjectId, nextChapterId) => {
      const previousIdentity = loadedIdentity;
      if (!isDirty.value || !previousIdentity) return true;

      try {
        await flushCurrentSession();
        return true;
      } catch {
        if (
          nextProjectId === previousIdentity.projectId &&
          projectStore.projectId === previousIdentity.projectId &&
          projectStore.activeId === nextChapterId
        ) {
          projectStore.setActiveId(previousIdentity.chapterId);
        }
        return false;
      }
    },
    onLoadError: (targetProjectId, targetChapterId) => {
      if (
        loadedIdentity &&
        projectStore.projectId === loadedIdentity.projectId &&
        targetProjectId === loadedIdentity.projectId &&
        projectStore.activeId === targetChapterId
      ) {
        projectStore.setActiveId(loadedIdentity.chapterId);
      }
    },
  });
  const {
    content: loadedContent,
    loadedProjectId,
    loadedChapterId,
    isLoading: chapterLoading,
  } = chapterSession;
  const isLoading = computed(() => chapterLoading.value || snapshotOperationBusy.value);

  watch(
    [loadedProjectId, loadedChapterId],
    ([loadedPid, loadedCid]) => {
      loadedIdentity =
        loadedPid && loadedCid ? { projectId: loadedPid, chapterId: loadedCid } : null;
    },
    { flush: 'sync' }
  );

  watch([loadedContent, loadedProjectId, loadedChapterId], ([newContent, loadedPid, loadedCid]) => {
    if (loadedPid !== projectId.value || loadedCid !== chapterId.value) return;
    isDirty.value = false;
    currentHtml.value = newContent;
  });

  watch(
    currentHtml,
    (content) => {
      if (
        loadedIdentity &&
        projectStore.projectId === loadedIdentity.projectId &&
        nodeMap.value.has(loadedIdentity.chapterId)
      ) {
        updateNodeStats(loadedIdentity.chapterId, countWords(content), isDirty.value);
      }
    },
    { flush: 'sync' }
  );

  const handleRename = async (newName: string) => {
    if (chapterId.value && newName !== activeChapter.value?.name) {
      await renameNode(chapterId.value, newName);
    }
  };

  const handleContentChange = (delta: number) => {
    if (delta !== 0) addWords(delta);
  };

  const autoSaveInterval = computed(
    () => settings.value.general.autoSaveInterval || APP_CONSTANTS.EDITOR.AUTO_SAVE_INTERVAL
  );

  useAutoSave(flushCurrentSession, autoSaveInterval);
  const restoreSnapshot = async (filename: string) => {
    const targetProjectId = projectId.value;
    const targetChapterId = chapterId.value;
    if (!targetProjectId || !targetChapterId) return;

    snapshotOperationBusy.value = true;
    try {
      await flushCurrentSession();
      await flushProjectSync();
      if (
        projectId.value !== targetProjectId ||
        chapterId.value !== targetChapterId ||
        !loadedIdentity ||
        loadedIdentity.projectId !== targetProjectId ||
        loadedIdentity.chapterId !== targetChapterId
      ) {
        return;
      }
      const newContent = await snapshotStore.restoreSnapshot(targetChapterId, filename);
      if (
        projectId.value !== targetProjectId ||
        chapterId.value !== targetChapterId ||
        !loadedIdentity ||
        loadedIdentity.projectId !== targetProjectId ||
        loadedIdentity.chapterId !== targetChapterId
      ) {
        return;
      }
      currentHtml.value = newContent;
      isDirty.value = false;
      updateNodeStats(targetChapterId, countWords(newContent), false);
      return newContent;
    } catch (error) {
      notifyError('Failed to restore snapshot', error);
      throw error;
    } finally {
      snapshotOperationBusy.value = false;
    }
  };

  const branchSnapshot = async (filename: string) => {
    const targetProjectId = projectId.value;
    const targetChapterId = chapterId.value;
    if (!targetProjectId || !targetChapterId) return;

    snapshotOperationBusy.value = true;
    try {
      await flushCurrentSession();
      await flushProjectSync();
      if (
        projectId.value !== targetProjectId ||
        chapterId.value !== targetChapterId ||
        !loadedIdentity ||
        loadedIdentity.projectId !== targetProjectId ||
        loadedIdentity.chapterId !== targetChapterId
      ) {
        return;
      }
      await snapshotStore.branchSnapshot(targetChapterId, filename);
    } catch (error) {
      notifyError('Failed to branch snapshot', error);
      throw error;
    } finally {
      snapshotOperationBusy.value = false;
    }
  };

  const handleSave = async (content: string) => {
    const identity = loadedIdentity;
    if (
      !identity ||
      identity.projectId !== projectId.value ||
      identity.chapterId !== chapterId.value
    ) {
      return;
    }
    if (currentHtml.value !== content) {
      isDirty.value = true;
      currentHtml.value = content;
    }
    await flushCurrentSession();
  };

  const openResearchArtifact = async (id: string) => {
    const identity = { projectId: projectId.value, projectPath: projectStore.path };
    if (!identity.projectPath) return;

    try {
      const artifacts = await researchApi.fetchArtifacts();
      if (projectId.value !== identity.projectId || projectStore.path !== identity.projectPath)
        return;

      const researchRoot = `${identity.projectPath.replace(/[\\/]+$/, '')}/research/`.replace(
        /\\/g,
        '/'
      );
      const currentProjectArtifacts = artifacts.filter((artifact) =>
        artifact.path.replace(/\\/g, '/').startsWith(researchRoot)
      );
      researchStore.artifacts = currentProjectArtifacts;
      const currentActiveId = researchStore.activeArtifact?.id;
      researchStore.setActiveArtifact(
        currentActiveId
          ? (currentProjectArtifacts.find((artifact) => artifact.id === currentActiveId) ?? null)
          : null
      );

      const artifact = currentProjectArtifacts.find((item) => item.id === id);
      if (artifact) {
        researchStore.setActiveArtifact(artifact);
      }
    } catch (error) {
      if (projectId.value === identity.projectId && projectStore.path === identity.projectPath) {
        notifyError('Failed to fetch research artifacts', error);
      }
    }
  };

  const unregisterSession = registerChapterSession(projectStore, flushCurrentSession);
  onBeforeUnmount(() => {
    unregisterSession();
    void flushCurrentSession().catch((error) => {
      notifyError('Failed to save chapter before closing', error);
    });
  });

  return {
    activeChapter,
    currentHtml,
    isLoading,
    isDirty,
    handleRename,
    handleContentChange,
    handleSave,
    restoreSnapshot,
    branchSnapshot,
    openResearchArtifact,
  };
}
