import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { mount, type VueWrapper } from '@vue/test-utils';
import { VueDraggableNext } from 'vue-draggable-next';
import FileTree from '../FileTree.vue';
import type { FileNode } from '../../../types';

let wrapper: VueWrapper | undefined;

const node = (id: string, children: FileNode[] = []): FileNode => ({
  id,
  name: id,
  filename: `${id}.md`,
  word_count: 7,
  children,
});

beforeEach(() => {
  setActivePinia(createPinia());
});

afterEach(() => {
  wrapper?.unmount();
  wrapper = undefined;
});

describe('root chapter reparenting', () => {
  it.each(['add-first', 'remove-first'] as const)(
    'preserves the moved subtree and destination children when sortable emits %s',
    (eventOrder) => {
      const source = node('source', [node('source-scene')]);
      const destination = node('destination', [node('destination-scene')]);
      wrapper = mount(FileTree, { props: { modelValue: [source, destination] } });
      const rootDraggable = wrapper.getComponent(VueDraggableNext);
      const destinationTree = wrapper
        .findAllComponents(FileTree)
        .find((tree) =>
          tree.props('modelValue').some((child: FileNode) => child.id === 'destination-scene')
        )!;
      const destinationDraggable = destinationTree.getComponent(VueDraggableNext);
      const add = () => destinationDraggable.vm.spliceList(0, 0, source);
      const remove = () => rootDraggable.vm.spliceList(0, 1);

      // Sortable's add/remove callbacks run synchronously, before Vue refreshes props.
      if (eventOrder === 'add-first') {
        add();
        remove();
      } else {
        remove();
        add();
      }

      const updates = wrapper.emitted('update:modelValue')!;
      expect(updates[updates.length - 1][0]).toEqual([
        node('destination', [source, node('destination-scene')]),
      ]);
    }
  );
});
