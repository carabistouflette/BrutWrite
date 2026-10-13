import { defineComponent, h } from 'vue';
import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import SnapshotManager from '../SnapshotManager.vue';

const snapshotStoreMock = vi.hoisted(() => ({
  snapshots: ['2026-10-13T120000_00000000_e3b0c442.md'],
  loading: false,
  fetchSnapshots: vi.fn(),
  loadSnapshotContent: vi.fn(),
}));

vi.mock('../../../stores/snapshots', () => ({
  useSnapshotStore: () => snapshotStoreMock,
}));

const ConfirmationModalStub = defineComponent({
  props: { show: Boolean },
  emits: ['confirm', 'cancel', 'close'],
  setup(props, { emit, slots }) {
    return () =>
      props.show
        ? h('div', { class: 'confirmation-stub' }, [
            h('button', { onClick: () => emit('confirm') }, 'Confirm'),
            slots.default?.(),
          ])
        : null;
  },
});

describe('SnapshotManager', () => {
  let wrapper: ReturnType<typeof mount> | undefined;
  let target: HTMLDivElement;

  beforeEach(() => {
    snapshotStoreMock.snapshots = ['2026-10-13T120000_00000000_e3b0c442.md'];
    snapshotStoreMock.fetchSnapshots.mockReset();
    snapshotStoreMock.loadSnapshotContent.mockReset().mockResolvedValue('');
    target = document.createElement('div');
    target.id = 'app-scale-root';
    document.body.append(target);
  });

  afterEach(() => {
    wrapper?.unmount();
    target.remove();
  });

  it('allows restoring an empty snapshot from the confirmation dialog', async () => {
    const filename = snapshotStoreMock.snapshots[0];
    wrapper = mount(SnapshotManager, {
      props: { chapterId: 'chapter-1', currentContent: '<p>Draft</p>' },
      global: { stubs: { ConfirmationModal: ConfirmationModalStub } },
    });
    await flushPromises();

    const snapshotButton = Array.from(target.querySelectorAll('button')).find((button) =>
      button.textContent?.includes('e3b0c442')
    );
    expect(snapshotButton).toBeDefined();
    snapshotButton?.click();
    await flushPromises();

    const restoreButton = Array.from(target.querySelectorAll('button')).find((button) =>
      button.textContent?.includes('Restore This Version')
    );
    expect(restoreButton?.disabled).toBe(false);
    restoreButton?.click();
    await flushPromises();

    target.querySelector<HTMLButtonElement>('.confirmation-stub button')?.click();
    await flushPromises();

    expect(wrapper.emitted('restore')).toEqual([['', filename]]);
  });
});
