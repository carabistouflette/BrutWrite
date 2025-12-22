import { useCalendar } from '../useCalendar';
import { type VisTimelineItem } from './useVisTimelineData';


export function useVisTimelineOptions(
    onMove: (item: VisTimelineItem, callback: (item: VisTimelineItem | null) => void) => void
) {
    const { formatDate, getYear } = useCalendar();

    const options = {
        orientation: { axis: 'top', item: 'top' },
        stack: true, 
        showCurrentTime: false,
        zoomable: true,
        moveable: true,
        editable: {
            updateTime: true, // Allow resizing/dragging
            updateGroup: true, // Allow moving between swimlanes
            remove: false, // Use the editor to remove
            add: false,
        },
        groupOrder: 'id',
        margin: { item: { horizontal: 5, vertical: 20 } },
        snap: null,
        onMove: onMove,
        format: {
            minorLabels: (date: Date, _scale: string, _step: number) => {
                    return formatDate(date);
            },
            majorLabels: (date: Date, _scale: string, _step: number) => {
                    // For major labels we often want just the Year
                    return getYear(date);
            }
        }
    };

    return {
        options
    };
}
