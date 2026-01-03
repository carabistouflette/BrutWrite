import { ref, watch } from 'vue';
import { useResizable } from './useResizable';
import { useResearchStore } from '../../stores/research';

export function useLayoutController() {
  const researchStore = useResearchStore();

  // Sidebars Widths
  const {
    width: sidebarWidth,
    isResizing: isResizingSidebar, // renamed for clarity
    startResize: startResizeSidebar,
  } = useResizable({
    initialWidth: 256,
    minWidth: 200,
    maxWidth: 600,
  });

  const {
    width: researchWidth,
    isResizing: isResizingResearch,
    startResize: startResizeResearch,
  } = useResizable({
    initialWidth: 400,
    minWidth: 300,
    maxWidth: 800,
    edge: 'right',
  });

  // Toggle State
  const showSettings = ref(false);
  const showCharacters = ref(false);
  const showCharacterGraph = ref(false);
  const showTimeline = ref(false);
  const showResearch = ref(false);

  // Auto-open research sidebar when artifact is selected
  watch(
    () => researchStore.activeArtifact,
    (artifact) => {
      if (artifact && !showResearch.value) {
        showResearch.value = true;
      }
    }
  );

  return {
    // Resizing
    sidebarWidth,
    isResizingSidebar,
    startResizeSidebar,
    researchWidth,
    isResizingResearch,
    startResizeResearch,

    // Toggles
    showSettings,
    showCharacters,
    showCharacterGraph,
    showTimeline,
    showResearch,
  };
}
