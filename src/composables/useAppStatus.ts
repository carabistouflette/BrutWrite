import { ref } from 'vue';

export type StatusType = 'success' | 'error' | 'info';

export interface AppStatus {
    message: string;
    type: StatusType;
    id: number;
}

const notifications = ref<AppStatus[]>([]);

export function useAppStatus() {

    const notify = (message: string, type: StatusType = 'info', duration = 3000) => {
        const id = Date.now();
        notifications.value.push({ message, type, id });

        if (duration > 0) {
            setTimeout(() => {
                removeNotification(id);
            }, duration);
        }
    };

    const notifyError = (message: string, error?: any) => {
        console.error(message, error);
        notify(message, 'error', 5000);
    };

    const notifySuccess = (message: string) => {
        notify(message, 'success', 3000);
    };

    const removeNotification = (id: number) => {
        notifications.value = notifications.value.filter(n => n.id !== id);
    };

    return {
        notifications,
        notify,
        notifyError,
        notifySuccess,
        removeNotification
    };
}
