<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  show: boolean;
  x: number;
  y: number;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const style = computed(() => ({
  top: `${props.y}px`,
  left: `${props.x}px`,
}));

const handleClose = () => {
  emit('close');
};
</script>

<template>
  <Teleport to="#app-scale-root">
    <div
      v-if="show"
      class="fixed inset-0 z-90"
      @click="handleClose"
      @contextmenu.prevent="handleClose"
    ></div>
    <transition
      enter-active-class="transition duration-100 ease-out"
      enter-from-class="opacity-0 scale-95"
      enter-to-class="opacity-100 scale-100"
      leave-active-class="transition duration-75 ease-in"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-95"
    >
      <div
        v-if="show"
        class="context-menu-glass fixed z-9999 min-w-[180px] py-2 rounded-xl"
        :style="style"
      >
        <slot></slot>
      </div>
    </transition>
  </Teleport>
</template>

<style scoped lang="postcss">
@reference "../../style.css";

.context-menu-glass {
  background-color: rgba(255, 255, 255, 0.15);
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  box-shadow: 0 10px 40px -10px rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(255, 255, 255, 0.2);
}

:deep(.menu-item) {
  @apply px-4 py-2 text-xs font-medium cursor-pointer transition-all duration-200 mx-1 rounded-lg;
}

:deep(.menu-item-default) {
  @apply text-ink/80 hover:bg-accent hover:text-white;
}

:deep(.menu-item-danger) {
  @apply text-red-500 hover:bg-red-500 hover:text-white;
}
</style>
