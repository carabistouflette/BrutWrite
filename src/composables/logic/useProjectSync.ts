import { projectApi } from '../../api/project';
import { useAppStatus } from '../useAppStatus';
import { projectToManifest } from '../../utils/tree';
import { projectData, projectId, pendingMetadataUpdates } from '../state/projectState';

let syncManifestTimeout: ReturnType<typeof setTimeout> | null = null;
let metadataTimeout: ReturnType<typeof setTimeout> | null = null;

export function useProjectSync() {
    const { notifyError } = useAppStatus();

    const syncManifestDebounced = () => {
        if (syncManifestTimeout) clearTimeout(syncManifestTimeout);
        syncManifestTimeout = setTimeout(async () => {
            if (!projectId.value) return;

            const manifest = projectToManifest(projectData.value);

            try {
                await projectApi.updateManifest(projectId.value, manifest);
                console.debug('Manifest synced');
            } catch (e) {
                notifyError('Failed to sync manifest', e);
            }
        }, 1500); // 1.5s debounce for structural changes
    };

    const syncNodeMetadataDebounced = (nodeId: string, updates: any) => {
        // Merge updates for the same node
        const current = pendingMetadataUpdates.get(nodeId) || {};
        pendingMetadataUpdates.set(nodeId, { ...current, ...updates });

        if (metadataTimeout) clearTimeout(metadataTimeout);
        metadataTimeout = setTimeout(async () => {
            if (!projectId.value) return;
            const updatesToSync = Array.from(pendingMetadataUpdates.entries());
            pendingMetadataUpdates.clear();
            metadataTimeout = null;

            for (const [id, up] of updatesToSync) {
                try {
                    await projectApi.updateNodeMetadata(projectId.value, id, up);
                } catch (e) {
                    notifyError(`Failed to sync metadata for node ${id}`, e);
                }
            }
        }, 1000);
    };

    return {
        syncManifestDebounced,
        syncNodeMetadataDebounced
    };
}
