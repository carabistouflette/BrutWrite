import type { FileNode, Manifest, Chapter } from '../types';

export function findNode(nodes: FileNode[], id: string): FileNode | undefined {
  const stack = [...nodes];
  while (stack.length > 0) {
    const node = stack.pop()!;
    if (node.id === id) return node;
    if (node.children && node.children.length > 0) {
      for (const child of node.children) {
        stack.push(child);
      }
    }
  }
  return undefined;
}

export function traverse(
  nodes: FileNode[],
  callback: (node: FileNode, parentId?: string) => void,
  parentId?: string
) {
  const stack: { node: FileNode; pid?: string }[] = nodes.map((n) => ({ node: n, pid: parentId }));
  while (stack.length > 0) {
    const { node, pid } = stack.pop()!;
    callback(node, pid);
    if (node.children && node.children.length > 0) {
      // Push in reverse order to maintain visual order if popping from end
      for (let i = node.children.length - 1; i >= 0; i--) {
        stack.push({ node: node.children[i], pid: node.id });
      }
    }
  }
}

export function buildNodeMap(nodes: FileNode[]): Map<string, FileNode> {
  const map = new Map<string, FileNode>();
  traverse(nodes, (node) => {
    map.set(node.id, node);
  });
  return map;
}

export function deleteFromList(nodes: FileNode[], id: string): boolean {
  const index = nodes.findIndex((n) => n.id === id);
  if (index !== -1) {
    nodes.splice(index, 1);
    return true;
  }
  for (const node of nodes) {
    if (node.children && deleteFromList(node.children, id)) return true;
  }
  return false;
}

export function findAndAdd(nodes: FileNode[], parentId: string, newNode: FileNode): boolean {
  const index = nodes.findIndex((n) => n.id === parentId);
  if (index !== -1) {
    if (!nodes[index].children) nodes[index].children = [];
    nodes[index].children!.push(newNode);
    return true;
  }
  for (const node of nodes) {
    if (node.children && findAndAdd(node.children, parentId, newNode)) return true;
  }
  return false;
}

export function findAndRename(nodes: FileNode[], id: string, newName: string): boolean {
  const node = nodes.find((n) => n.id === id);
  if (node) {
    node.name = newName;
    return true;
  }
  for (const n of nodes) {
    if (n.children && findAndRename(n.children, id, newName)) return true;
  }
  return false;
}

export function reconstructHierarchy(chapters: Chapter[]): FileNode[] {
  const sortedChapters = [...chapters].sort((a, b) => a.order - b.order);
  const nodeMap = new Map<string, FileNode>();
  const rootNodes: FileNode[] = [];

  // First pass: create all nodes
  for (const c of sortedChapters) {
    const node: FileNode = {
      id: c.id,
      name: c.title,
      filename: c.filename,
      word_count: c.word_count || 0,
      chronological_date: c.chronological_date,
      abstract_timeframe: c.abstract_timeframe,
      duration: c.duration,
      plotline_tag: c.plotline_tag,
      depends_on: c.depends_on,
      pov_character_id: c.pov_character_id,
      children: [],
    };
    nodeMap.set(c.id, node);
  }

  // Second pass: link parents/children
  for (const c of sortedChapters) {
    const node = nodeMap.get(c.id);
    if (!node) continue;
    if (c.parent_id) {
      const parent = nodeMap.get(c.parent_id);
      parent?.children?.push(node);
    } else {
      rootNodes.push(node);
    }
  }

  return rootNodes;
}

export function projectToManifest(nodes: FileNode[]): Manifest {
  const chapters: Chapter[] = [];
  let order = 0;

  traverse(nodes, (node, parentId) => {
    chapters.push({
      id: node.id,
      parent_id: parentId,
      title: node.name,
      filename: node.filename || `${node.id}.md`,
      word_count: node.word_count || 0,
      order: order++,
      chronological_date: node.chronological_date,
      abstract_timeframe: node.abstract_timeframe,
      duration: node.duration,
      plotline_tag: node.plotline_tag,
      depends_on: node.depends_on,
      pov_character_id: node.pov_character_id,
    });
  });

  return { chapters };
}
