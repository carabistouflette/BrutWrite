import { ref } from 'vue';

const isDragging = ref(false);

export function useDragState() {
  const setDragging = (value: boolean) => {
    isDragging.value = value;
  };

  return {
    isDragging,
    setDragging,
  };
}
