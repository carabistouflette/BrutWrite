import { getCurrentWindow } from '@tauri-apps/api/window';

export function useWindowControls() {
    // Check if running in Tauri environment
    // @ts-ignore
    const isTauri = !!(window.__TAURI_INTERNALS__);

    const minimize = () => {
        if (isTauri) {
            getCurrentWindow().minimize();
        } else {
            console.log('Mock: Minimize Window');
        }
    };

    const toggleMaximize = () => {
        if (isTauri) {
            getCurrentWindow().toggleMaximize();
        } else {
            console.log('Mock: Toggle Maximize Window');
        }
    };

    const close = () => {
        if (isTauri) {
            getCurrentWindow().close();
        } else {
            console.log('Mock: Close Window');
        }
    };

    return {
        minimize,
        toggleMaximize,
        close,
        isTauri
    };
}
