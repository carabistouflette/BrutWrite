type SessionFlusher = () => Promise<void>;

type ProjectSessions = Map<string, Set<SessionFlusher>>;

const sessionFlushers = new WeakMap<object, ProjectSessions>();

export function registerResearchNoteSession(
  projectStore: object,
  projectId: string,
  projectRoot: string,
  flush: SessionFlusher
): () => void {
  let projectSessions = sessionFlushers.get(projectStore);
  if (!projectSessions) {
    projectSessions = new Map();
    sessionFlushers.set(projectStore, projectSessions);
  }

  const identityKey = `${projectId}\0${projectRoot}`;
  let flushers = projectSessions.get(identityKey);
  if (!flushers) {
    flushers = new Set();
    projectSessions.set(identityKey, flushers);
  }

  flushers.add(flush);
  return () => {
    flushers?.delete(flush);
    if (flushers?.size === 0) projectSessions?.delete(identityKey);
    if (projectSessions?.size === 0) sessionFlushers.delete(projectStore);
  };
}

export async function flushResearchNoteSessions(
  projectStore: object,
  projectId: string,
  projectRoot: string
): Promise<void> {
  const identityKey = `${projectId}\0${projectRoot}`;
  await Promise.all(
    Array.from(sessionFlushers.get(projectStore)?.get(identityKey) ?? [], (flush) => flush())
  );
}
