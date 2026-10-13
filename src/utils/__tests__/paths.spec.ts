import { describe, expect, it } from 'vitest';
import { normalizeProjectPath } from '../paths';

describe('project directory selected from a manifest', () => {
  it.each([
    ['/draft/novel/project.json', '/draft/novel'],
    ['C:\\Draft\\novel\\project.json', 'C:\\Draft\\novel'],
    ['\\\\server\\share\\novel\\project.json', '\\\\server\\share\\novel'],
    ['/project.json', '/'],
    ['C:\\project.json', 'C:\\'],
    ['project.json', '.'],
  ])('loads the directory containing %s', (selected, directory) => {
    expect(normalizeProjectPath(selected)).toBe(directory);
  });

  it('preserves legacy project folders named with a JSON extension', () => {
    expect(normalizeProjectPath('/draft/MyStory.json')).toBe('/draft/MyStory.json');
  });

  it('preserves an explicitly selected directory with a trailing separator', () => {
    expect(normalizeProjectPath('/draft/project.json/')).toBe('/draft/project.json/');
  });
});
