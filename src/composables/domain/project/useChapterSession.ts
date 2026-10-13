import { ref, watch, type Ref } from 'vue';
import { chaptersApi } from '../../../api/chapters';
import { useAppStatus } from '../../ui/useAppStatus';

type SessionFlusher = () => Promise<void>;

const sessionFlushers = new WeakMap<object, Set<SessionFlusher>>();

export function registerChapterSession(projectStore: object, flush: SessionFlusher) {
  let flushers = sessionFlushers.get(projectStore);
  if (!flushers) {
    flushers = new Set();
    sessionFlushers.set(projectStore, flushers);
  }

  flushers.add(flush);
  return () => {
    flushers?.delete(flush);
    if (flushers?.size === 0) sessionFlushers.delete(projectStore);
  };
}

export async function flushChapterSessions(projectStore: object) {
  await Promise.all(Array.from(sessionFlushers.get(projectStore) ?? [], (flush) => flush()));
}

interface ChapterSessionLifecycle {
  beforeLoad?: (projectId: string, chapterId: string) => Promise<boolean> | boolean;
  onLoadError?: (projectId: string, chapterId: string) => void;
}

export function useChapterSession(
  projectId: Ref<string | undefined>,
  chapterId: Ref<string | undefined>,
  lifecycle: ChapterSessionLifecycle = {}
) {
  const content = ref('');
  const loadedProjectId = ref<string>();
  const loadedChapterId = ref<string>();
  const isLoading = ref(false);
  const { notifyError } = useAppStatus();
  let loadSequence = 0;

  const loadContent = async () => {
    const pid = projectId.value;
    const cid = chapterId.value;
    const sequence = ++loadSequence;

    if (!pid || !cid) {
      content.value = '';
      loadedProjectId.value = undefined;
      loadedChapterId.value = undefined;
      isLoading.value = false;
      return;
    }

    if (pid === loadedProjectId.value && cid === loadedChapterId.value) {
      isLoading.value = false;
      return;
    }

    isLoading.value = true;
    try {
      const mayLoad = await lifecycle.beforeLoad?.(pid, cid);
      if (
        sequence !== loadSequence ||
        projectId.value !== pid ||
        chapterId.value !== cid ||
        mayLoad === false
      ) {
        return;
      }

      const loaded = await chaptersApi.loadContent(pid, cid);
      if (sequence === loadSequence && projectId.value === pid && chapterId.value === cid) {
        content.value = loaded;
        loadedProjectId.value = pid;
        loadedChapterId.value = cid;
      }
    } catch (error) {
      if (sequence === loadSequence && projectId.value === pid && chapterId.value === cid) {
        notifyError('Failed to load chapter content', error);
        lifecycle.onLoadError?.(pid, cid);
      }
    } finally {
      if (sequence === loadSequence) isLoading.value = false;
    }
  };

  watch([projectId, chapterId], () => void loadContent(), { immediate: true });

  return {
    content,
    loadedProjectId,
    loadedChapterId,
    isLoading,
    loadContent,
  };
}
