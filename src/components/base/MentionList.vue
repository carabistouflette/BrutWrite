<template>
  <div
    class="bg-paper/95 backdrop-blur-xl border border-white/20 shadow-2xl rounded-xl overflow-hidden min-w-[200px] flex flex-col p-1 text-sm text-ink font-medium"
  >
    <template v-if="items.length">
      <button
        v-for="(item, index) in items"
        :key="index"
        class="w-full text-left px-3 py-2 rounded-lg transition-colors flex items-center justify-between"
        :class="{
          'bg-accent text-white': index === selectedIndex,
          'hover:bg-ink/5': index !== selectedIndex,
        }"
        @click="selectItem(index)"
      >
        <span>{{ item.label || item.name }}</span>
        <span v-if="item.role" class="text-xs opacity-60 uppercase tracking-wider ml-2">{{
          item.role
        }}</span>
      </button>
    </template>
    <div v-else class="px-3 py-2 text-ink/40 text-xs italic">No characters found</div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';

interface MentionItem {
  id: string;
  name: string;
  role?: string;
  label?: string;
}

const props = defineProps<{
  items: MentionItem[];
  command: (props: { id: string; label: string }) => void;
}>();

const selectedIndex = ref(0);

watch(
  () => props.items,
  () => {
    selectedIndex.value = 0;
  }
);

const onKeyDown = ({ event }: { event: KeyboardEvent }) => {
  if (event.key === 'ArrowUp') {
    upHandler();
    return true;
  }

  if (event.key === 'ArrowDown') {
    downHandler();
    return true;
  }

  if (event.key === 'Enter') {
    enterHandler();
    return true;
  }

  return false;
};

const upHandler = () => {
  selectedIndex.value = (selectedIndex.value + props.items.length - 1) % props.items.length;
};

const downHandler = () => {
  selectedIndex.value = (selectedIndex.value + 1) % props.items.length;
};

const enterHandler = () => {
  selectItem(selectedIndex.value);
};

const selectItem = (index: number) => {
  const item = props.items[index];

  if (item) {
    props.command({ id: item.id, label: item.name });
  }
};

defineExpose({
  onKeyDown,
});
</script>
