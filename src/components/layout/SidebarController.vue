<script setup lang="ts">
import { ref, nextTick } from 'vue';
import { storeToRefs } from 'pinia';
import FileTree from '../project/FileTree.vue';
import ContextMenu from '../base/ContextMenu.vue';
import AppLogo from '../common/AppLogo.vue';
import SidebarFooter from './SidebarFooter.vue';
import AddChapterButton from '../project/AddChapterButton.vue';
import { useProjectNodeOperations } from '../../composables/domain/project/useProjectNodeOperations';
import { useProjectStore } from '../../stores/project';
import { useContextMenu } from '../../composables/ui/useContextMenu';
import { useAppStatus } from '../../composables/ui/useAppStatus';

const emit = defineEmits<{
  (e: 'open-settings'): void;
  (e: 'open-characters'): void;
  (e: 'open-character-graph'): void;
  (e: 'open-timeline'): void;
  (e: 'open-research'): void;
  (e: 'change-project'): void;
}>();

const projectStore = useProjectStore();
const { nodes: projectData, activeId } = storeToRefs(projectStore);
const {
  addChapter: addChapterLogic,
  addSection: addSectionLogic,
  deleteNode: handleDelete,
  renameNode: handleRenameLogic,
  updateStructure,
} = useProjectNodeOperations();

const {
  showMenu,
  menuPos,
  contextData: targetNodeId,
  openMenu,
  closeMenu,
} = useContextMenu<string>();

const { notifyError } = useAppStatus();

// Local State
const editingId = ref<string | null>(null);
const isAdding = ref(false);
const sidebarScrollRef = ref<HTMLElement | null>(null);

// Rename Logic
const handleRenameRequest = (id: string) => {
  editingId.value = id;
};

const handleRenameSubmit = ({ id, name }: { id: string; name: string }) => {
  handleRenameLogic(id, name);
  editingId.value = null;
};

const handleRenameCancel = () => {
  editingId.value = null;
};

// Context Menu Actions
const startRenameTarget = () => {
  if (targetNodeId.value) {
    handleRenameRequest(targetNodeId.value);
  }
  closeMenu();
};

const deleteTarget = async () => {
  if (targetNodeId.value) {
    try {
      await handleDelete(targetNodeId.value);
    } catch (e) {
      notifyError('Failed to delete node', e);
    }
  }
  closeMenu();
};

const addSection = async () => {
  if (targetNodeId.value) {
    try {
      await addSectionLogic(targetNodeId.value);
    } catch (e) {
      notifyError('Failed to add section', e);
    }
  }
  closeMenu();
};

const handleContextMenu = ({ e, id }: { e: MouseEvent; id: string }) => {
  openMenu(e, id);
};

// Add Chapter Logic
const addChapter = async () => {
  try {
    await addChapterLogic();
  } catch (e) {
    notifyError('Failed to create chapter', e);
    return;
  }

  isAdding.value = true;
  setTimeout(() => (isAdding.value = false), 600);

  nextTick(() => {
    if (sidebarScrollRef.value) {
      sidebarScrollRef.value.scrollTo({
        top: sidebarScrollRef.value.scrollHeight,
        behavior: 'smooth',
      });
    }
  });
};
</script>

<template>
  <div class="flex flex-col h-full w-full">
    <AppLogo />

    <div ref="sidebarScrollRef" class="flex-1 overflow-y-auto px-3 py-2 scroll-smooth">
      <FileTree
        :model-value="projectData"
        :active-id="activeId"
        :editing-id="editingId"
        @update:model-value="updateStructure"
        @context-menu="handleContextMenu"
        @request-rename="handleRenameRequest"
        @submit-rename="handleRenameSubmit"
        @cancel-rename="handleRenameCancel"
      />

      <AddChapterButton :is-adding="isAdding" @click="addChapter" />
    </div>

    <!-- Context Menu -->
    <ContextMenu :show="showMenu" :x="menuPos.x" :y="menuPos.y" @close="closeMenu">
      <div class="menu-item menu-item-default" @click.stop="addSection">Add Section</div>
      <div class="menu-item menu-item-default" @click.stop="startRenameTarget">Rename</div>
      <div class="h-px bg-ink/5 my-1 mx-2"></div>
      <div class="menu-item menu-item-danger" @click.stop="deleteTarget">Delete</div>
    </ContextMenu>

    <SidebarFooter
      class="mt-auto"
      @open-settings="emit('open-settings')"
      @open-characters="emit('open-characters')"
      @open-character-graph="emit('open-character-graph')"
      @open-timeline="emit('open-timeline')"
      @open-research="emit('open-research')"
      @change-project="emit('change-project')"
    />
  </div>
</template>
