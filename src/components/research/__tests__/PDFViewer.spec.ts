import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import PDFViewer from '../PDFViewer.vue';

const mocks = vi.hoisted(() => ({
  getDocument: vi.fn(),
  readFile: vi.fn(),
}));

vi.mock('pdfjs-dist', () => ({
  version: 'test',
  GlobalWorkerOptions: { workerSrc: '' },
  getDocument: mocks.getDocument,
}));

vi.mock('@tauri-apps/plugin-fs', () => ({
  readFile: mocks.readFile,
}));

describe('PDFViewer', () => {
  let wrapper: VueWrapper | undefined;
  let pdf: {
    numPages: number;
    getPage: (pageNumber: number) => Promise<unknown>;
    loadingTask: { destroy: () => Promise<void> };
  };

  beforeEach(() => {
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(
      {} as CanvasRenderingContext2D
    );
    const pages = Array.from({ length: 2 }, () => ({
      getViewport: vi.fn(({ scale }: { scale: number }) => ({
        width: 600 * scale,
        height: 800 * scale,
      })),
      render: vi.fn(() => ({ promise: Promise.resolve(), cancel: vi.fn() })),
    }));
    pdf = {
      numPages: pages.length,
      getPage: vi.fn(async (pageNumber: number) => pages[pageNumber - 1]),
      loadingTask: { destroy: vi.fn().mockResolvedValue(undefined) },
    };
    mocks.readFile.mockResolvedValue(new Uint8Array([1]));
    mocks.getDocument.mockReturnValue({ promise: Promise.resolve(pdf) });
  });

  afterEach(() => {
    wrapper?.unmount();
    wrapper = undefined;
    vi.restoreAllMocks();
    vi.clearAllMocks();
  });

  it('navigates every page and disables navigation at document boundaries', async () => {
    wrapper = mount(PDFViewer, { props: { path: '/book.pdf' } });
    await flushPromises();

    expect(pdf.getPage).toHaveBeenLastCalledWith(1);
    expect(wrapper.get('[title="Previous Page"]').attributes('disabled')).toBeDefined();
    expect(wrapper.get('[title="Next Page"]').attributes('disabled')).toBeUndefined();

    await wrapper.get('[title="Next Page"]').trigger('click');
    await flushPromises();
    expect(pdf.getPage).toHaveBeenLastCalledWith(2);
    expect(wrapper.text()).toContain('2 / 2');
    expect(wrapper.get('[title="Next Page"]').attributes('disabled')).toBeDefined();

    await wrapper.get('[title="Previous Page"]').trigger('click');
    await flushPromises();
    expect(pdf.getPage).toHaveBeenLastCalledWith(1);
    expect(wrapper.text()).toContain('1 / 2');
  });
});
