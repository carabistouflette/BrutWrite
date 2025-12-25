import { onMounted, onBeforeUnmount, watch, type Ref } from 'vue';

export function useAutoSave(callback: () => Promise<void> | void, intervalSeconds: Ref<number>) {
  let intervalId: ReturnType<typeof setInterval> | undefined;

  const start = () => {
    stop();
    // Minimum 5 seconds to avoid performance killer loops
    const ms = Math.max(intervalSeconds.value, 5) * 1000;

    intervalId = setInterval(async () => {
      await callback();
    }, ms);
  };

  const stop = () => {
    if (intervalId) {
      clearInterval(intervalId);
      intervalId = undefined;
    }
  };

  watch(intervalSeconds, start);

  onMounted(start);
  onBeforeUnmount(stop);

  return {
    start,
    stop,
    restart: start,
  };
}
