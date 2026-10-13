import { storeToRefs } from 'pinia';
import { chaptersApi } from '../../../api/chapters';
import { projectApi } from '../../../api/project';
import { useAppStatus } from '../../ui/useAppStatus';
import { projectToManifest } from '../../../utils/tree';
import { useProjectStore } from '../../../stores/project';
import type {
  Chapter,
  ChapterMetadataUpdate,
  Manifest,
  NullableChapterMetadataField,
} from '../../../types';

interface PendingProjectSync {
  manifestTimer: ReturnType<typeof setTimeout> | null;
  metadataTimer: ReturnType<typeof setTimeout> | null;
  manifest: Manifest | null;
  metadata: Map<string, ChapterMetadataUpdate>;
  flushing: Promise<void> | null;
}

const pendingSyncByProject = new Map<string, PendingProjectSync>();
const nullableMetadataFields: Record<NullableChapterMetadataField, true> = {
  chronological_date: true,
  abstract_timeframe: true,
  duration: true,
  plotline_tag: true,
  depends_on: true,
  pov_character_id: true,
};

const getPendingSync = (projectId: string): PendingProjectSync => {
  let pending = pendingSyncByProject.get(projectId);
  if (!pending) {
    pending = {
      manifestTimer: null,
      metadataTimer: null,
      manifest: null,
      metadata: new Map(),
      flushing: null,
    };
    pendingSyncByProject.set(projectId, pending);
  }
  return pending;
};

export function useProjectSync() {
  const { notifyError } = useAppStatus();
  const projectStore = useProjectStore();
  const { nodes: projectData, projectId } = storeToRefs(projectStore);

  const flushProject = (id: string, pending: PendingProjectSync): Promise<void> => {
    if (pending.flushing) return pending.flushing;

    const flushing = Promise.resolve()
      .then(async () => {
        while (pending.manifest || pending.metadata.size > 0) {
          clearTimeout(pending.manifestTimer ?? undefined);
          pending.manifestTimer = null;
          clearTimeout(pending.metadataTimer ?? undefined);
          pending.metadataTimer = null;

          const manifest = pending.manifest;
          pending.manifest = null;
          const metadata = [...pending.metadata.entries()];
          pending.metadata.clear();
          const failures: unknown[] = [];

          if (manifest) {
            try {
              await projectApi.updateManifest(id, manifest);
            } catch (error) {
              pending.manifest ??= manifest;
              notifyError('Failed to sync manifest', error);
              failures.push(error);
            }
          }

          for (const [nodeId, updates] of metadata) {
            try {
              await chaptersApi.updateNodeMetadata(id, nodeId, updates);
            } catch (error) {
              const newerUpdates = pending.metadata.get(nodeId);
              pending.metadata.set(nodeId, { ...updates, ...newerUpdates });
              notifyError(`Failed to sync metadata for node ${nodeId}`, error);
              failures.push(error);
            }
          }

          if (failures.length > 0) throw failures[0];
        }
      })
      .finally(() => {
        pending.flushing = null;
        if (
          pendingSyncByProject.get(id) === pending &&
          !pending.manifest &&
          pending.metadata.size === 0 &&
          !pending.manifestTimer &&
          !pending.metadataTimer
        ) {
          pendingSyncByProject.delete(id);
        }
      });

    pending.flushing = flushing;
    return flushing;
  };

  const syncManifestDebounced = () => {
    const id = projectId.value;
    if (!id) return;

    const pending = getPendingSync(id);
    pending.manifest = projectToManifest(projectData.value);
    clearTimeout(pending.manifestTimer ?? undefined);
    pending.manifestTimer = setTimeout(() => {
      pending.manifestTimer = null;
      void flushProject(id, pending).catch(() => undefined);
    }, 1500);
  };

  const syncNodeMetadataDebounced = (nodeId: string, updates: Partial<Chapter>) => {
    const id = projectId.value;
    if (!id) return;

    const pending = getPendingSync(id);
    const normalizedUpdates = Object.fromEntries(
      Object.entries(updates).map(([key, value]) => [
        key,
        value === undefined && nullableMetadataFields[key as NullableChapterMetadataField]
          ? null
          : value,
      ])
    ) as ChapterMetadataUpdate;
    const current = pending.metadata.get(nodeId) || {};
    pending.metadata.set(nodeId, { ...current, ...normalizedUpdates });

    clearTimeout(pending.metadataTimer ?? undefined);
    pending.metadataTimer = setTimeout(() => {
      pending.metadataTimer = null;
      void flushProject(id, pending).catch(() => undefined);
    }, 1000);
  };

  const flush = (id = projectId.value): Promise<void> => {
    if (!id) return Promise.resolve();
    const pending = pendingSyncByProject.get(id);
    return pending ? flushProject(id, pending) : Promise.resolve();
  };

  const cancel = (id = projectId.value) => {
    if (!id) return;
    const pending = pendingSyncByProject.get(id);
    if (!pending || pending.flushing) return;

    clearTimeout(pending.manifestTimer ?? undefined);
    clearTimeout(pending.metadataTimer ?? undefined);
    pending.manifestTimer = null;
    pending.metadataTimer = null;
    pendingSyncByProject.delete(id);
  };

  return {
    syncManifestDebounced,
    syncNodeMetadataDebounced,
    flush,
    cancel,
  };
}
