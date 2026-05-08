export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  extension: string | null;
}

export interface FilterResult {
  entries: FileEntry[];
  expandedPaths: Set<string>;
}

export function filterTree(
  rootEntries: FileEntry[],
  childrenMap: Map<string, FileEntry[]>,
  query: string,
): FilterResult {
  if (!query.trim()) {
    return { entries: rootEntries, expandedPaths: new Set() };
  }

  const lower = query.toLowerCase();
  const expandedPaths = new Set<string>();
  const matchingPaths = new Set<string>();

  function searchRecursive(entries: FileEntry[], ancestors: string[]): boolean {
    let anyMatch = false;
    for (const entry of entries) {
      const nameMatches = entry.name.toLowerCase().includes(lower);
      let childMatch = false;

      if (entry.is_dir) {
        const children = childrenMap.get(entry.path) ?? [];
        childMatch = searchRecursive(children, [...ancestors, entry.path]);
      }

      if (nameMatches || childMatch) {
        matchingPaths.add(entry.path);
        anyMatch = true;
        if (childMatch) {
          expandedPaths.add(entry.path);
        }
        for (const ancestor of ancestors) {
          expandedPaths.add(ancestor);
          matchingPaths.add(ancestor);
        }
      }
    }
    return anyMatch;
  }

  searchRecursive(rootEntries, []);

  const filtered = rootEntries.filter((e) => matchingPaths.has(e.path));
  return { entries: filtered, expandedPaths };
}
