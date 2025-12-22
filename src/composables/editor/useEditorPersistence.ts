import { type Editor } from '@tiptap/vue-3';
import { projectApi } from '../../api/project';
import { useAppStatus } from '../../composables/ui/useAppStatus';
import { useProjectData } from '../../composables/logic/useProjectData';
import type { Chapter } from '../../types';
import type { Ref } from 'vue';

export function useEditorPersistence(
  editor: Ref<Editor | undefined>,
  isDirty: Ref<boolean>,
  resetWordCountState: () => void
) {
  const { notifyError } = useAppStatus();
  const { updateNodeStats, activeId } = useProjectData();

  const loadChapter = async (projectId: string, chapterId: string) => {
    if (!editor.value) return;
    try {
      const content = await projectApi.loadChapter(projectId, chapterId);
      editor.value.commands.setContent(content, { emitUpdate: false });
      resetWordCountState();
      isDirty.value = false;
    } catch (e) {
      notifyError('Failed to load chapter', e);
      editor.value.commands.setContent(`<h1>Error</h1><p>Could not load chapter.</p>`);
    }
  };

  const saveChapter = async (projectId: string, chapterId: string) => {
    if (!editor.value) return;

    // Optimization: Don't save if nothing changed
    if (!isDirty.value) {
      return;
    }

    try {
      const content = editor.value.getHTML();
      const metadata = await projectApi.saveChapter(projectId, chapterId, content);

      // Sync frontend state from the metadata returned by backend
      if (activeId.value) {
        const chapter = metadata.manifest.chapters.find((c: Chapter) => c.id === chapterId);
        if (chapter) {
          updateNodeStats(chapterId, chapter.word_count, false);
        }
      }

      isDirty.value = false; // Reset dirty flag on successful save
      console.debug(`Auto-saved chapter ${chapterId}`);
    } catch (e) {
      notifyError(`Failed to save chapter ${chapterId}`, e);
    }
  };

  return {
    loadChapter,
    saveChapter,
  };
}
