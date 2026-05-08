import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { filterTree } from "@/lib/file-tree-filter";
import { FileTreeFilter } from "./file-tree-filter";
import { FileTreeNode } from "./file-tree-node";
import type { useFileTree } from "@/hooks/use-file-tree";
import "./file-tree-panel.css";

const MAX_RENDER_DEPTH = 50;

interface FileTreePanelProps {
  tree: ReturnType<typeof useFileTree>;
  onFileSelect: (path: string) => void;
  activePath: string | null;
}

export function FileTreePanel({ tree, onFileSelect, activePath }: FileTreePanelProps) {
  const { t } = useTranslation();
  const [debouncedFilter, setDebouncedFilter] = useState("");
  const timerRef = useRef(0);

  const handleFilterChange = (value: string) => {
    tree.setFilter(value);
    clearTimeout(timerRef.current);
    timerRef.current = window.setTimeout(() => setDebouncedFilter(value), 150);
  };

  useEffect(() => {
    if (tree.filter === "") setDebouncedFilter("");
  }, [tree.filter]);

  useEffect(() => {
    return () => clearTimeout(timerRef.current);
  }, []);

  const filtered = useMemo(
    () => filterTree(tree.rootEntries, tree.childrenMap, debouncedFilter),
    [tree.rootEntries, tree.childrenMap, debouncedFilter],
  );

  const mergedExpanded = useMemo(() => {
    const merged = new Set(tree.expandedPaths);
    for (const p of filtered.expandedPaths) merged.add(p);
    return merged;
  }, [tree.expandedPaths, filtered.expandedPaths]);

  return (
    <aside
      className={`ft-panel ${tree.open ? "open" : ""} ${tree.resizing ? "resizing" : ""}`}
      style={{ "--ft-width": `${tree.width}px` } as React.CSSProperties}
      aria-hidden={!tree.open}
    >
      <div className="ft-resize" onPointerDown={tree.startResize} />
      <div className="ft-head">
        <FileTreeFilter value={tree.filter} onChange={handleFilterChange} />
      </div>
      <div className="ft-body" role="tree">
        {tree.loadError ? (
          <div className="ft-empty">{t("fileTree.loadError")}</div>
        ) : filtered.entries.length === 0 && debouncedFilter ? (
          <div className="ft-empty">{t("fileTree.noResults")}</div>
        ) : (
          filtered.entries.map((entry) => (
            <FileTreeNode
              key={entry.path}
              entry={entry}
              depth={0}
              maxDepth={MAX_RENDER_DEPTH}
              expanded={mergedExpanded.has(entry.path)}
              active={entry.path === activePath}
              children={tree.childrenMap.get(entry.path)}
              expandedPaths={mergedExpanded}
              childrenMap={tree.childrenMap}
              activePath={activePath}
              onToggle={tree.toggleExpand}
              onSelect={onFileSelect}
            />
          ))
        )}
      </div>
    </aside>
  );
}
