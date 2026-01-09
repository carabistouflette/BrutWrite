/**
 * Handles cross-platform filename extraction.
 * Supports both forward slashes (/) and backslashes (\).
 *
 * @param path - The full file path
 * @param removeExtension - Whether to drop the file extension (default: false)
 * @returns The filename part of the path
 */
export function extractFilename(path: string, removeExtension = false): string {
  if (!path) return '';

  // Split by both separators
  const parts = path.split(/[/\\]/);
  let filename = parts.pop() || '';

  if (removeExtension && filename.includes('.')) {
    // Remove only the last extension
    filename = filename.replace(/\.[^/.]+$/, '');
  }

  return filename;
}

/**
 * Ensures a consistent format for project names derived from paths.
 * Replaces underscores/dashes with spaces for display if needed?
 * For now, just uses filename cleaning.
 */
export function formatProjectName(path: string): string {
  const filename = extractFilename(path, true);
  // Optional: Add logic to prettify "my_project" -> "My Project" if desired.
  // Keeping it simple for now to match legacy behavior but robustly.
  return filename || 'Untitled Project';
}
