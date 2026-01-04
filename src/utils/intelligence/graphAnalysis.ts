import type { GraphAlert, GraphNode, GraphMetrics } from '../../types/intelligence';

/**
 * Analyzes the character graph metrics and nodes to generate actionable alerts.
 *
 * @param metrics - Graph metrics payload
 * @param nodes - List of graph nodes
 * @param ghosts - List of unmapped nodes
 * @param projectProtagonists - List of protagonist graph nodes (filtered from nodes based on project data)
 */
export function getGraphAlerts(
  metrics: GraphMetrics,
  ghosts: GraphNode[],
  projectProtagonists: GraphNode[]
): GraphAlert[] {
  const result: GraphAlert[] = [];

  // 1. Solipsism: High isolation
  if (metrics.isolationRatio > 0.5) {
    result.push({
      code: 'SOLIPSISM',
      primaryText: 'Low Connectivity',
      tooltip: 'Over 50% of your cast has no meaningful interactions.',
    });
  }

  // 2. Satellite: Disconnected subgraphs
  if (metrics.connectedComponents > 1) {
    result.push({
      code: 'SATELLITE',
      primaryText: 'Isolated Subplots',
      tooltip: `There are ${metrics.connectedComponents} groups of characters that never interact with the main cast.`,
    });
  }

  // 3. Ghost: Defined but unused
  if (ghosts.length > 0) {
    result.push({
      code: 'GHOST',
      primaryText: 'Unmapped Characters',
      tooltip: `${ghosts.length} character(s) declared but never mentioned.`,
    });
  }

  // 4. Protagonist Fading: Low mention count for MCs
  if (projectProtagonists.length > 0 && projectProtagonists.every((p) => p.mentionCount < 3)) {
    result.push({
      code: 'PROTAGONIST_ABSENT',
      primaryText: 'Protagonist Fading',
      tooltip: 'Your protagonist has very few mentions.',
    });
  }

  return result;
}
