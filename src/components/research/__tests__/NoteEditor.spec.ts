import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { nextTick } from 'vue';
import NoteEditor from '../NoteEditor.vue';
import { useProjectStore } from '../../../stores/project';
import { flushResearchNoteSessions } from '../../../composables/domain/research/useResearchNoteSession';

const mocks = vi.hoisted(() => ({
  readFile: vi.fn(),
  saveNoteContent: vi.fn(),
  notifyError: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-fs', () => ({
  readFile: mocks.readFile,
}));

vi.mock('../../../stores/research', () => ({
  useResearchStore: () => ({
    activeArtifact: { name: 'Research note' },
    saveNoteContent: mocks.saveNoteContent,
  }),
}));

vi.mock('../../../composables/ui/useAppStatus', () => ({
  useAppStatus: () => ({ notifyError: mocks.notifyError }),
}));

describe('NoteEditor', () => {
  let wrapper: VueWrapper | undefined;

  beforeEach(() => {
    setActivePinia(createPinia());
    const projectStore = useProjectStore();
    projectStore.projectId = 'project-1';
    projectStore.path = '/project-1';
    vi.useFakeTimers();
    mocks.readFile.mockImplementation(async (path: string) =>
      new TextEncoder().encode(path === '/second.md' ? 'Second note' : 'First note')
    );
    mocks.saveNoteContent.mockResolvedValue(undefined);
  });

  afterEach(() => {
    wrapper?.unmount();
    wrapper = undefined;
    vi.useRealTimers();
    vi.clearAllMocks();
  });

  it('flushes a pending edit to the original note before loading another artifact', async () => {
    wrapper = mount(NoteEditor, { props: { id: 'first', path: '/first.md' } });
    await flushPromises();
    expect(wrapper.get('textarea').element.value).toBe('First note');

    await wrapper.get('textarea').setValue('Edited first note');
    await wrapper.setProps({ id: 'second', path: '/second.md' });
    await flushPromises();

    expect(mocks.saveNoteContent).toHaveBeenCalledExactlyOnceWith(
      'first',
      '/first.md',
      'Edited first note'
    );
    expect(wrapper.get('textarea').element.value).toBe('Second note');
    await vi.advanceTimersByTimeAsync(1000);
    expect(mocks.saveNoteContent).toHaveBeenCalledTimes(1);
  });

  it('blocks editing after a failed read and recovers when a readable note is selected', async () => {
    mocks.readFile.mockRejectedValueOnce(new Error('access denied'));
    wrapper = mount(NoteEditor, { props: { id: 'first', path: '/first.md' } });
    await flushPromises();

    expect(wrapper.get('textarea').element.disabled).toBe(true);
    expect(wrapper.find('[role="alert"]').exists()).toBe(true);

    await wrapper.setProps({ id: 'second', path: '/second.md' });
    await flushPromises();

    expect(wrapper.get('textarea').element.disabled).toBe(false);
    expect(wrapper.get('textarea').element.value).toBe('Second note');
    expect(wrapper.find('[role="alert"]').exists()).toBe(false);
  });
  it('keeps a failed pending edit available to an awaited project flush', async () => {
    mocks.saveNoteContent.mockRejectedValueOnce(new Error('disk full'));
    wrapper = mount(NoteEditor, { props: { id: 'first', path: '/first.md' } });
    await flushPromises();

    await wrapper.get('textarea').setValue('Unsaved edit');
    await vi.advanceTimersByTimeAsync(1000);
    await flushPromises();

    await flushResearchNoteSessions(useProjectStore(), 'project-1', '/project-1');

    expect(mocks.saveNoteContent).toHaveBeenCalledTimes(2);
    expect(mocks.saveNoteContent).toHaveBeenLastCalledWith('first', '/first.md', 'Unsaved edit');
  });

  it('keeps an unmounted failed session registered for a later transition retry', async () => {
    mocks.saveNoteContent
      .mockRejectedValueOnce(new Error('disk full'))
      .mockRejectedValueOnce(new Error('disk full'));
    wrapper = mount(NoteEditor, { props: { id: 'first', path: '/first.md' } });
    await flushPromises();

    await wrapper.get('textarea').setValue('Unsaved edit');
    await vi.advanceTimersByTimeAsync(1000);
    await flushPromises();
    wrapper.unmount();
    wrapper = undefined;
    await flushPromises();

    await flushResearchNoteSessions(useProjectStore(), 'project-1', '/project-1');

    expect(mocks.saveNoteContent).toHaveBeenCalledTimes(3);
    expect(mocks.saveNoteContent).toHaveBeenLastCalledWith('first', '/first.md', 'Unsaved edit');
  });

  it('re-registers a reused editor for the new project identity', async () => {
    wrapper = mount(NoteEditor, {
      props: { id: 'shared', path: '/project-1/note.md' },
    });
    await flushPromises();

    await wrapper.get('textarea').setValue('Project one edit');
    await vi.advanceTimersByTimeAsync(1000);
    await flushPromises();

    const projectStore = useProjectStore();
    projectStore.projectId = 'project-2';
    projectStore.path = '/project-2';
    await nextTick();
    await flushPromises();
    await wrapper.setProps({ id: 'shared', path: '/project-2/note.md' });
    await flushPromises();

    await wrapper.get('textarea').setValue('Project two edit');
    await flushResearchNoteSessions(projectStore, 'project-1', '/project-1');
    expect(mocks.saveNoteContent).toHaveBeenCalledTimes(1);

    await flushResearchNoteSessions(projectStore, 'project-2', '/project-2');
    expect(mocks.saveNoteContent).toHaveBeenCalledTimes(2);
    expect(mocks.saveNoteContent).toHaveBeenLastCalledWith(
      'shared',
      '/project-2/note.md',
      'Project two edit'
    );
  });
});
