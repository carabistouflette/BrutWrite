import { ref, watch, type Ref } from 'vue';
import { chaptersApi } from '../../../api/chapters';
import { useAppStatus } from '../../ui/useAppStatus';

export function useChapterSession(
  projectId: Ref<string | undefined>,
  chapterId: Ref<string | undefined>
) {
  const content = ref('');
  const isLoading = ref(false);
  const { notifyError } = useAppStatus();

  const loadContent = async () => {
    const pid = projectId.value;
    const cid = chapterId.value;

    if (!pid || !cid) {
      content.value = '';
      return;
    }

    isLoading.value = true;
    try {
      content.value = await chaptersApi.loadContent(pid, cid);
    } catch (e) {
      notifyError('Failed to load chapter content', e);
      content.value = '<h1>Error</h1><p>Could not load content.</p>';
    } finally {
      isLoading.value = false;
    }
  };

  // Watch for ID changes to reload
  watch([projectId, chapterId], loadContent, { immediate: true });

  return {
    content,
    isLoading,
    loadContent,
  };
}
