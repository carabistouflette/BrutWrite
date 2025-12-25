<script setup lang="ts">
import { useDialogStore } from '../../stores/dialog';
import ConfirmationModal from './ConfirmationModal.vue';
import InputModal from './InputModal.vue';
import { storeToRefs } from 'pinia';

const store = useDialogStore();
const { activeDialog, dialogProps } = storeToRefs(store);

// Helper to cast props
const confirmProps = dialogProps as unknown as typeof dialogProps.value;
const inputProps = dialogProps as unknown as typeof dialogProps.value & {
  initialValue: string;
  placeholder: string;
};
</script>

<template>
  <ConfirmationModal
    :show="activeDialog === 'confirm'"
    :title="confirmProps.title"
    :message="confirmProps.message"
    :confirm-label="confirmProps.confirmLabel"
    :cancel-label="confirmProps.cancelLabel"
    :is-destructive="confirmProps.isDestructive"
    @close="store.handleCancel"
    @cancel="store.handleCancel"
    @confirm="store.handleConfirm()"
  />

  <InputModal
    :show="activeDialog === 'input'"
    :title="inputProps.title"
    :message="inputProps.message"
    :initial-value="inputProps.initialValue"
    :placeholder="inputProps.placeholder"
    :confirm-label="inputProps.confirmLabel"
    :cancel-label="inputProps.cancelLabel"
    @close="store.handleCancel"
    @cancel="store.handleCancel"
    @confirm="(val) => store.handleConfirm(val)"
  />
</template>
