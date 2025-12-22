import { projectApi } from '../../api/project';
import { useAppStatus } from '../useAppStatus';
import { projectToManifest } from '../../utils/tree';
import { 
    projectData, 
    projectId, 
    pendingMetadataUpdates, 
    syncManifestTimeout, 
    metadataTimeout 
} from '../state/projectState';
import type { Chapter } from '../../types';

export function useProjectSync() {
    const { notifyError: originalNotifyError } = useAppStatus();

    const notifyError = (message: string, error?: unknown) => {
        originalNotifyError(message, error);
    };

    const syncManifestDebounced = () => {
        if (syncManifestTimeout.value) clearTimeout(syncManifestTimeout.value);
        syncManifestTimeout.value = setTimeout(async () => {
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

    const syncNodeMetadataDebounced = (nodeId: string, updates: Partial<Chapter>) => {
        // Merge updates for the same node
        const current = pendingMetadataUpdates.get(nodeId) || {};
        pendingMetadataUpdates.set(nodeId, { ...current, ...updates });

        if (metadataTimeout.value) clearTimeout(metadataTimeout.value);
        metadataTimeout.value = setTimeout(async () => {
            if (!projectId.value) return;
            const updatesToSync = Array.from(pendingMetadataUpdates.entries());
            pendingMetadataUpdates.clear();
            metadataTimeout.value = null;

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
