<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
    size?: 'sm' | 'md' | 'lg';
    block?: boolean;
    disabled?: boolean;
    loading?: boolean;
    icon?: string;
  }>(),
  {
    variant: 'primary',
    size: 'md',
    block: false,
    disabled: false,
    loading: false,
  }
);

const emit = defineEmits<{
  (e: 'click', event: MouseEvent): void;
}>();

const classes = computed(() => {
  return [
    'base-button',
    `variant-${props.variant}`,
    `size-${props.size}`,
    { 'w-full': props.block },
    { 'is-loading': props.loading },
    { 'is-disabled': props.disabled },
  ];
});
</script>

<template>
  <button :class="classes" :disabled="disabled || loading" @click="emit('click', $event)">
    <div v-if="loading" class="loading-spinner"></div>
    <span v-else-if="$slots.icon || icon" class="icon-wrapper">
      <slot name="icon">
        <!-- If using a centralized icon system, usage would be here -->
        <!-- For now relying on slots or simple svg passthrough -->
      </slot>
    </span>
    <span class="content">
      <slot></slot>
    </span>
  </button>
</template>

<style scoped>
.base-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px; /* Rounded corners for modern feel, or 0 for brutalist? Keeping mild round per existing */
  font-weight: 600;
  transition: all 0.2s ease;
  cursor: pointer;
  position: relative;
  overflow: hidden;
  border: 1px solid transparent; /* Prepare for borders */
  gap: 0.5rem;
}

.base-button:focus-visible {
  outline: 2px solid var(--color-accent);
  outline-offset: 2px;
}

.base-button.is-disabled {
  opacity: 0.5;
  cursor: not-allowed;
  filter: grayscale(100%);
}

/* --- VARIANTS --- */

/* Primary: Ink background, Paper text */
.variant-primary {
  background-color: var(--color-ink);
  color: var(--color-paper);
}
.variant-primary:hover:not(.is-disabled) {
  background-color: var(--color-ink); /* Typically same but relying on transform/shadow */
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}
.variant-primary:active:not(.is-disabled) {
  transform: translateY(0);
}

/* Secondary: Glass/Outline */
.variant-secondary {
  background-color: rgba(255, 255, 255, 0.4);
  color: var(--color-ink);
  border-color: rgba(var(--color-ink-rgb), 0.1);
  backdrop-filter: blur(8px);
}
.variant-secondary:hover:not(.is-disabled) {
  background-color: rgba(255, 255, 255, 0.8);
  border-color: rgba(var(--color-ink-rgb), 0.3);
}

/* Ghost: No background */
.variant-ghost {
  background-color: transparent;
  color: var(--color-ink);
}
.variant-ghost:hover:not(.is-disabled) {
  background-color: rgba(var(--color-ink-rgb), 0.05);
}

/* Danger */
.variant-danger {
  background-color: var(--color-danger);
  color: white;
}
.variant-danger:hover:not(.is-disabled) {
  filter: brightness(1.1);
}

/* --- SIZES --- */
.size-sm {
  padding: 0.25rem 0.75rem;
  font-size: 0.75rem;
  height: 28px;
}
.size-md {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  height: 40px;
}
.size-lg {
  padding: 0.75rem 1.5rem;
  font-size: 1rem;
  height: 56px; /* Matches WelcomeScreen big buttons */
  border-radius: 12px;
}

/* --- UTILS --- */
.loading-spinner {
  width: 1em;
  height: 1em;
  border: 2px solid currentColor;
  border-right-color: transparent;
  border-radius: 50%;
  animation: spin 0.75s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
