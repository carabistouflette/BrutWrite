<script setup lang="ts">
import { useDialogStore } from '../../stores/dialog';
import ConfirmationModal from './ConfirmationModal.vue';
import InputModal from './InputModal.vue';
import { storeToRefs } from 'pinia';

const store = useDialogStore();
const { currentDialog } = storeToRefs(store);
</script>

<template>
  <ConfirmationModal
    v-if="currentDialog?.type === 'confirm'"
    :show="true"
    :title="currentDialog.props.title"
    :message="currentDialog.props.message"
    :confirm-label="currentDialog.props.confirmLabel"
    :cancel-label="currentDialog.props.cancelLabel"
    :is-destructive="currentDialog.props.isDestructive"
    @close="store.handleCancel"
    @cancel="store.handleCancel"
    @confirm="store.handleConfirm()"
  />

  <InputModal
    v-if="currentDialog?.type === 'input'"
    :show="true"
    :title="currentDialog.props.title"
    :message="currentDialog.props.message"
    :initial-value="currentDialog.props.initialValue"
    :placeholder="currentDialog.props.placeholder"
    :confirm-label="currentDialog.props.confirmLabel"
    :cancel-label="currentDialog.props.cancelLabel"
    @close="store.handleCancel"
    @cancel="store.handleCancel"
    @confirm="(val) => store.handleConfirm(val)"
  />
</template>
