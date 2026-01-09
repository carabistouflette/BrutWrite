import { useTiptapMentions } from './useTiptapMentions';
import type { Transaction } from '@tiptap/pm/state';
import type { AnyExtension } from '@tiptap/core'; // or Extension
import { getBaseExtensions, EDITOR_PROPS } from '../../config/editor';

export function useTiptapConfig(
  onUpdate: (props: { transaction: Transaction }) => void,
  onSelectionUpdate: () => void
) {
  const { mentionExtension } = useTiptapMentions();

  let extensions: AnyExtension[] = [...getBaseExtensions(), ...mentionExtension];

  // Robustly deduplicate extensions by name to resolve any internal or package-level duplication
  const extensionNames = new Set<string>();
  extensions = extensions.filter((extension) => {
    if (!extension) return false;
    const name = extension.name;
    if (!name || extensionNames.has(name)) {
      return false;
    }
    extensionNames.add(name);
    return true;
  });

  return {
    extensions,
    editorProps: EDITOR_PROPS,
    onUpdate,
    onSelectionUpdate,
  };
}
