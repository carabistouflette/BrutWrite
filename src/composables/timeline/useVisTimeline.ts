import { ref, onMounted, onUnmounted, type Ref } from 'vue';
import { DataSet } from 'vis-data';
import { Timeline } from 'vis-timeline';
import 'vis-timeline/styles/vis-timeline-graph2d.min.css';
import { useVisTimelineOptions } from './useVisTimelineOptions';
import { useVisTimelineEvents } from './useVisTimelineEvents';
import {
  useVisTimelineData,
  type VisTimelineItem,
  type VisTimelineGroup,
} from './useVisTimelineData';

export type { VisTimelineItem };

export function useVisTimeline(
  containerRef: Ref<HTMLElement | null>,
  onSelect: (sceneId: string | null) => void,
  onHover: (info: { id: string; x: number; y: number } | null) => void,
  onRangeChange: () => void
) {
  const timeline = ref<Timeline | null>(null);
  const items = new DataSet<VisTimelineItem>([]);
  const groups = new DataSet<VisTimelineGroup>([]);
  const isMounted = ref(false);

  const { handleItemMove, handleDrop } = useVisTimelineEvents(timeline, groups);
  const { options } = useVisTimelineOptions(handleItemMove);
  const { syncData } = useVisTimelineData(items, groups, isMounted, timeline);

  onMounted(() => {
    isMounted.value = true;
    if (!containerRef.value) return;

    timeline.value = new Timeline(containerRef.value, items as any, groups as any, options);

    // Selection handler
    timeline.value.on('select', (props: { items: string[] }) => {
      if (props.items.length > 0) {
        onSelect(props.items[0]);
      } else {
        onSelect(null);
      }
    });

    // Hover handler
    timeline.value.on('itemover', (props: { item: string; event: MouseEvent }) => {
      onHover({
        id: props.item,
        x: props.event.clientX,
        y: props.event.clientY,
      });
    });

    timeline.value.on('itemout', () => {
      onHover(null);
    });

    // Range change handler
    timeline.value.on('rangechange', () => {
      onRangeChange();
    });

    // Initial sync
    syncData();
  });

  onUnmounted(() => {
    isMounted.value = false;
    timeline.value?.destroy();
    timeline.value = null;
  });

  function fit() {
    timeline.value?.fit();
  }

  function zoomIn(percentage: number) {
    timeline.value?.zoomIn(percentage);
  }

  function zoomOut(percentage: number) {
    timeline.value?.zoomOut(percentage);
  }

  return {
    timeline,
    fit,
    zoomIn,
    zoomOut,
    handleDrop,
    isMounted,
  };
}
