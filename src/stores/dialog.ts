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
  // State
  const activeDialog = ref<'confirm' | 'input' | null>(null);

  const dialogProps = ref<DialogOptions | InputDialogOptions>({
    title: '',
    message: '',
  });

  // Resolvers
  let resolveConfirm: ((value: boolean) => void) | null = null;
  let resolvePrompt: ((value: string | null) => void) | null = null;

  // Actions
  const confirm = (options: DialogOptions): Promise<boolean> => {
    activeDialog.value = 'confirm';
    dialogProps.value = options;

    return new Promise((resolve) => {
      resolveConfirm = resolve;
    });
  };

  const prompt = (options: InputDialogOptions): Promise<string | null> => {
    activeDialog.value = 'input';
    dialogProps.value = options;

    return new Promise((resolve) => {
      resolvePrompt = resolve;
    });
  };

  const handleConfirm = (value?: string) => {
    activeDialog.value = null;

    if (resolveConfirm) {
      resolveConfirm(true);
      resolveConfirm = null;
    }
    if (resolvePrompt) {
      resolvePrompt(value ?? '');
      resolvePrompt = null;
    }
  };

  const handleCancel = () => {
    activeDialog.value = null;

    if (resolveConfirm) {
      resolveConfirm(false);
      resolveConfirm = null;
    }
    if (resolvePrompt) {
      resolvePrompt(null);
      resolvePrompt = null;
    }
  };

  return {
    activeDialog,
    dialogProps,
    confirm,
    prompt,
    handleConfirm,
    handleCancel,
  };
});
