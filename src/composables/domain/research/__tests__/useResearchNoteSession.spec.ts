import { describe, expect, it, vi } from 'vitest';
import { flushResearchNoteSessions, registerResearchNoteSession } from '../useResearchNoteSession';

describe('research note session flush registry', () => {
  it('flushes only sessions captured for the requested project identity and root', async () => {
    const projectStore = {};
    const projectA = vi.fn(async () => {});
    const otherRoot = vi.fn(async () => {});
    const otherProject = vi.fn(async () => {});

    registerResearchNoteSession(projectStore, 'project-a', '/projects/a', projectA);
    registerResearchNoteSession(projectStore, 'project-a', '/projects/copy', otherRoot);
    registerResearchNoteSession(projectStore, 'project-b', '/projects/a', otherProject);

    await flushResearchNoteSessions(projectStore, 'project-a', '/projects/a');

    expect(projectA).toHaveBeenCalledTimes(1);
    expect(otherRoot).not.toHaveBeenCalled();
    expect(otherProject).not.toHaveBeenCalled();
  });
});
