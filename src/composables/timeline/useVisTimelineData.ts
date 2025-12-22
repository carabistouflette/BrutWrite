import { type Ref, watch } from 'vue';
import { DataSet } from 'vis-data';
import { useTimeline } from './useTimeline';
import { useTimeHelpers } from '../logic/useTimeHelpers';
import { Timeline, type TimelineItemType } from 'vis-timeline';
import type { Plotline } from '../../types';

export interface VisTimelineItem {
  id: string | number;
  group?: string | number;
  content: string | HTMLElement;
  start: Date | number | string;
  end?: Date | number | string;
  type?: TimelineItemType;
  className?: string;
  title?: string;
}

export interface VisTimelineGroup {
  id: string | number;
  content: string;
  style?: string;
}

export function useVisTimelineData(
  items: DataSet<VisTimelineItem>,
  groups: DataSet<VisTimelineGroup>,
  isMounted: Ref<boolean>,
  timeline: Ref<Timeline | null>
) {
  const { plotlines, assignedScenes, paradoxWarnings } = useTimeline();
  const { parseDurationToMillis, parseAbstractTimeframe } = useTimeHelpers();

  function syncData() {
    if (!isMounted.value || !timeline.value) return;
    // Sync groups
    const groupsData: VisTimelineGroup[] = plotlines.value.map((pl: Plotline) => ({
      id: pl.id,
      content: pl.name,
      style: `border-left: 4px solid ${pl.color}; background: linear-gradient(90deg, ${pl.color}10 0%, transparent 100%);`,
    }));

    const existingGroupIds = groups.getIds();
    const newGroupIds = groupsData.map((g: VisTimelineGroup) => g.id);
    const toRemoveGroups = existingGroupIds.filter(
      (id: string | number) => !newGroupIds.includes(id)
    );

    groups.remove(toRemoveGroups);
    groups.update(groupsData);

    if (groups.length === 0) {
      groups.add({ id: 'main', content: 'Main Plot' });
    }

    // Sync items
    const itemsData: VisTimelineItem[] = assignedScenes.value.map((scene) => {
      const warnings = paradoxWarnings.value.filter((w) => w.sceneIds.includes(scene.id));
      const hasWarning = warnings.length > 0;
      const start = scene.chronological_date
        ? new Date(scene.chronological_date)
        : parseAbstractTimeframe(scene.abstract_timeframe);

      let durationMillis = parseDurationToMillis(scene.duration);
      // Default to 2 hours if no duration, so it appears as a resizable range
      const isProjectedDuration = durationMillis <= 0;
      if (isProjectedDuration) {
        durationMillis = 1000 * 60 * 60 * 2; // 2 Hours default visualization
      }

      const end = new Date(start.getTime() + durationMillis);

      const group =
        scene.plotline_tag && groups.get(scene.plotline_tag)
          ? scene.plotline_tag
          : groups.getIds()[0];

      return {
        id: scene.id,
        group: group,
        content: `<span class="scene-item-content">${scene.title}</span>`,
        start,
        end,
        type: 'range', // Always range to allow resizing
        className: hasWarning
          ? 'timeline-item--warning'
          : isProjectedDuration
            ? 'timeline-item--projected'
            : 'timeline-item--normal',
        title: warnings.map((w) => w.message).join('\n') || scene.title,
      };
    });

    const existingItemIds = items.getIds();
    const newItemIds = itemsData.map((i) => i.id);
    const toRemoveItems = existingItemIds.filter((id: string | number) => !newItemIds.includes(id));

    items.remove(toRemoveItems);
    items.update(itemsData);
  }

  watch([plotlines, assignedScenes], syncData, { deep: true });

  return {
    syncData,
  };
}
