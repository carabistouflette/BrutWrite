import { defineStore } from 'pinia';
import { ref } from 'vue';

interface DialogOptions {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  isDestructive?: boolean;
}

interface InputDialogOptions extends DialogOptions {
  initialValue?: string;
  placeholder?: string;
}

export const useDialogStore = defineStore('dialog', () => {
  // Discriminated Union State
  type DialogState =
    | { type: 'confirm'; props: DialogOptions; resolve: (value: boolean) => void }
    | { type: 'input'; props: InputDialogOptions; resolve: (value: string | null) => void }
    | null;

  const currentDialog = ref<DialogState>(null);

  // Actions
  const confirm = (options: DialogOptions): Promise<boolean> => {
    return new Promise((resolve) => {
      currentDialog.value = {
        type: 'confirm',
        props: options,
        resolve,
      };
    });
  };

  const prompt = (options: InputDialogOptions): Promise<string | null> => {
    return new Promise((resolve) => {
      currentDialog.value = {
        type: 'input',
        props: options,
        resolve,
      };
    });
  };

  const handleConfirm = (value?: string) => {
    if (!currentDialog.value) return;

    if (currentDialog.value.type === 'confirm') {
      currentDialog.value.resolve(true);
    } else if (currentDialog.value.type === 'input') {
      currentDialog.value.resolve(value ?? '');
    }
    currentDialog.value = null;
  };

  const handleCancel = () => {
    if (!currentDialog.value) return;

    if (currentDialog.value.type === 'confirm') {
      currentDialog.value.resolve(false);
    } else if (currentDialog.value.type === 'input') {
      currentDialog.value.resolve(null);
    }
    currentDialog.value = null;
  };

  return {
    currentDialog,
    confirm,
    prompt,
    handleConfirm,
    handleCancel,
  };
});
