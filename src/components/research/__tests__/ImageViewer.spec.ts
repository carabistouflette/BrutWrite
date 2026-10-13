import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import ImageViewer from '../ImageViewer.vue';

const mocks = vi.hoisted(() => ({ readFile: vi.fn() }));
vi.mock('@tauri-apps/plugin-fs', () => ({ readFile: mocks.readFile }));

let wrapper: VueWrapper | undefined;
afterEach(() => {
  wrapper?.unmount();
  wrapper = undefined;
  vi.unstubAllGlobals();
  vi.clearAllMocks();
});

function installBlobUrls() {
  class PreviewURL extends URL {
    static createObjectURL = vi
      .fn()
      .mockReturnValueOnce('blob:current')
      .mockReturnValue('blob:stale');
    static revokeObjectURL = vi.fn();
  }
  vi.stubGlobal('URL', PreviewURL);
}

describe('ImageViewer', () => {
  it('keeps the selected image when an earlier file read finishes late', async () => {
    installBlobUrls();
    let finishEarlier!: (bytes: Uint8Array) => void;
    mocks.readFile
      .mockImplementationOnce(
        () =>
          new Promise<Uint8Array>((resolve) => {
            finishEarlier = resolve;
          })
      )
      .mockResolvedValueOnce(new Uint8Array([2]));
    wrapper = mount(ImageViewer, { props: { path: '/earlier.png' } });
    await wrapper.setProps({ path: '/selected.png' });
    await flushPromises();
    expect(wrapper.get('img').attributes('src')).toBe('blob:current');
    finishEarlier(new Uint8Array([1]));
    await flushPromises();
    expect(wrapper.get('img').attributes('src')).toBe('blob:current');
  });

  it('removes the previous preview on a denied file and recovers on another selection', async () => {
    installBlobUrls();
    mocks.readFile
      .mockResolvedValueOnce(new Uint8Array([1]))
      .mockRejectedValueOnce(new Error('Permission denied'))
      .mockResolvedValueOnce(new Uint8Array([3]));
    wrapper = mount(ImageViewer, { props: { path: '/readable.png' } });
    await flushPromises();
    expect(wrapper.get('img').attributes('src')).toBe('blob:current');
    await wrapper.setProps({ path: '/denied.png' });
    await flushPromises();
    expect(wrapper.find('img').exists()).toBe(false);
    expect(wrapper.get('[role="alert"]').text()).toContain('Permission denied');
    await wrapper.setProps({ path: '/another.png' });
    await flushPromises();
    expect(wrapper.find('[role="alert"]').exists()).toBe(false);
    expect(wrapper.get('img').attributes('src')).toBe('blob:stale');
  });
});
