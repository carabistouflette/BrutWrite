import { save } from '@tauri-apps/plugin-dialog';
import { writeFile } from '@tauri-apps/plugin-fs';

export function useTimelineExport() {
  async function exportTimeline(
    container: HTMLElement,
    format: 'png' | 'pdf',
    defaultNamePrefix: string = 'timeline_export'
  ) {
    try {
      // Dynamic imports for heavy libraries
      const html2canvas = (await import('html2canvas')).default;
      const { jsPDF } = await import('jspdf');

      const canvas = await html2canvas(container, {
        scale: 2,
        backgroundColor: '#1a1a1a',
        ignoreElements: (element: Element) => element.classList.contains('narrative-overlay'),
      });

      const defaultName = `${defaultNamePrefix}_${new Date().toISOString().split('T')[0]}`;
      let fileData: Uint8Array;
      let filters = [];

      if (format === 'png') {
        const dataUrl = canvas.toDataURL('image/png');
        // Convert DataURL to Uint8Array
        const res = await fetch(dataUrl);
        const blob = await res.blob();
        fileData = new Uint8Array(await blob.arrayBuffer());
        filters = [{ name: 'PNG Image', extensions: ['png'] }];
      } else {
        const imgData = canvas.toDataURL('image/png');
        const pdf = new jsPDF({
          orientation: 'landscape',
          unit: 'px',
          format: [canvas.width, canvas.height],
        });
        pdf.addImage(imgData, 'PNG', 0, 0, canvas.width, canvas.height);
        fileData = new Uint8Array(pdf.output('arraybuffer'));
        filters = [{ name: 'PDF Document', extensions: ['pdf'] }];
      }

      // Open Save Dialog
      const path = await save({
        defaultPath: defaultName,
        filters: filters,
      });

      if (path) {
        await writeFile(path, fileData);
        // Export saved successfully
        return true;
      }
      return false;
    } catch (e) {
      console.error('Export failed:', e);
      throw e;
    }
  }

  return {
    exportTimeline,
  };
}
