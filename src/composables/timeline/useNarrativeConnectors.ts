import { ref, watch, onUnmounted } from 'vue';
import type { Ref } from 'vue';

export interface NarrativeConnector {
    from: string;
    to: string;
}

export interface ConnectorPath {
    d: string;
    color: string;
    isFlashback: boolean;
}

export function useNarrativeConnectors(
    containerRef: Ref<HTMLElement | null>,
    connectors: Ref<NarrativeConnector[]>,
    isMounted: Ref<boolean>
) {
    const connectorPaths = ref<ConnectorPath[]>([]);
    const showNarrativeConnectors = ref(false);
    
    // Optimization: Throttle connector updates to avoid layout thrashing
    let connectorAnimationFrame: number | null = null;
    let lastConnectorUpdate = 0;
    const CONNECTOR_UPDATE_THROTTLE = 16; // ms

    function updateConnectorPositions() {
        if (!isMounted.value || !showNarrativeConnectors.value) {
            connectorPaths.value = [];
            return;
        }

        const now = performance.now();
        if (now - lastConnectorUpdate < CONNECTOR_UPDATE_THROTTLE) {
            if (connectorAnimationFrame) cancelAnimationFrame(connectorAnimationFrame);
            connectorAnimationFrame = requestAnimationFrame(updateConnectorPositions);
            return;
        }
        lastConnectorUpdate = now;

        const paths: ConnectorPath[] = [];
        const canvasRef = containerRef.value;
        if (!canvasRef) return;

        const containerRect = canvasRef.getBoundingClientRect();
        
        if (connectors.value.length === 0) {
            connectorPaths.value = [];
            return;
        }

        // Get unique connected IDs
        const connectedIds = new Set<string>();
        connectors.value.forEach(conn => {
            connectedIds.add(conn.from);
            connectedIds.add(conn.to);
        });

        // Create a lookup for item positions
        const itemRects = new Map<string, DOMRect>();
        connectedIds.forEach(id => {
            const el = canvasRef.querySelector(`[data-id="${id}"]`);
            if (el) itemRects.set(id, el.getBoundingClientRect());
        });

        connectors.value.forEach(conn => {
            const fromRect = itemRects.get(conn.from);
            const toRect = itemRects.get(conn.to);

            if (fromRect && toRect) {
                const x1 = fromRect.right - containerRect.left;
                const y1 = fromRect.top + fromRect.height / 2 - containerRect.top;
                const x2 = toRect.left - containerRect.left;
                const y2 = toRect.top + toRect.height / 2 - containerRect.top;

                // Simple curve calculation
                const dist = Math.abs(x2 - x1);
                const cp1x = x1 + dist * 0.4; // Slightly tighter curves
                const cp1y = y1;
                const cp2x = x2 - dist * 0.4;
                const cp2y = y2;
                
                const isFlashback = x2 < x1;
                const pathD = `M ${x1} ${y1} C ${cp1x} ${cp1y}, ${cp2x} ${cp2y}, ${x2} ${y2}`;

                paths.push({
                    d: pathD,
                    color: isFlashback ? 'var(--color-warning)' : 'var(--color-primary)',
                    isFlashback
                });
            }
        });

        connectorPaths.value = paths;
    }

    function toggleConnectors() {
        showNarrativeConnectors.value = !showNarrativeConnectors.value;
    }

    watch(showNarrativeConnectors, (val) => {
        if (val) requestAnimationFrame(updateConnectorPositions);
    });

    onUnmounted(() => {
        if (connectorAnimationFrame) cancelAnimationFrame(connectorAnimationFrame);
    });

    return {
        connectorPaths,
        showNarrativeConnectors,
        toggleConnectors,
        updateConnectorPositions
    };
}
