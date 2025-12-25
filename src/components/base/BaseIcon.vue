<script setup lang="ts">
import { computed } from 'vue';
import { icons, type IconName } from '../../config/icons';

const props = defineProps<{
  name: IconName;
  size?: number | string;
}>();

const iconData = computed(() => icons[props.name]);

const style = computed(() => {
  if (!props.size) return {};
  const s = typeof props.size === 'number' ? `${props.size}px` : props.size;
  return { width: s, height: s };
});
</script>

<template>
  <!-- eslint-disable vue/no-v-html -->
  <svg
    v-if="iconData"
    xmlns="http://www.w3.org/2000/svg"
    :viewBox="iconData.viewBox"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    :style="style"
    v-html="iconData.content"
  ></svg>
</template>
