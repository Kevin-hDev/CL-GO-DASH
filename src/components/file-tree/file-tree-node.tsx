import { useEffect, useRef, useState } from "react";
import { ChevronRight } from "lucide-react";
import { FileIcon } from "@/components/file-preview/file-icon";
import { Icon } from "@iconify/react";
import type { IconifyIcon } from "@iconify/react";
import type { FileEntry } from "@/lib/file-tree-filter";
import defaultFolderRaw from "@iconify/icons-vscode-icons/default-folder.js";
import defaultFolderOpenedRaw from "@iconify/icons-vscode-icons/default-folder-opened.js";

const defaultFolder = (defaultFolderRaw as unknown as { default: IconifyIcon }).default ?? (defaultFolderRaw as unknown as IconifyIcon);
const defaultFolderOpened = (defaultFolderOpenedRaw as unknown as { default: IconifyIcon }).default ?? (defaultFolderOpenedRaw as unknown as IconifyIcon);

interface FileTreeNodeProps {
  entry: FileEntry;
  depth: number;
  maxDepth: number;
  expanded: boolean;
  active: boolean;
  children?: FileEntry[];
  expandedPaths: Set<string>;
  childrenMap: Map<string, FileEntry[]>;
  activePath: string | null;
  onToggle: (path: string) => void;
  onSelect: (path: string) => void;
}

export function FileTreeNode({
  entry,
  depth,
  maxDepth,
  expanded,
  active,
  children,
  expandedPaths,
  childrenMap,
  activePath,
  onToggle,
  onSelect,
}: FileTreeNodeProps) {
  const childrenRef = useRef<HTMLDivElement>(null);
  const [maxHeight, setMaxHeight] = useState<string>(expanded ? "none" : "0");

  const childCount = children?.length ?? 0;

  useEffect(() => {
    if (!childrenRef.current) return;
    if (expanded) {
      const h = childrenRef.current.scrollHeight;
      setMaxHeight(`${h}px`);
      const timer = setTimeout(() => setMaxHeight("none"), 200);
      return () => clearTimeout(timer);
    }
    const h = childrenRef.current.scrollHeight;
    setMaxHeight(`${h}px`);
    requestAnimationFrame(() => setMaxHeight("0"));
  }, [expanded, childCount]);

  const handleClick = () => {
    if (entry.is_dir) {
      onToggle(entry.path);
    } else {
      onSelect(entry.path);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      handleClick();
    }
  };

  const folderIcon = expanded ? defaultFolderOpened : defaultFolder;

  if (depth >= maxDepth) return null;

  return (
    <div>
      <div
        className={`ft-node ${active ? "ft-node-active" : ""}`}
        style={{ paddingLeft: depth * 16 + 8 }}
        onClick={handleClick}
        onKeyDown={handleKeyDown}
        role="treeitem"
        tabIndex={0}
        title={entry.name}
        aria-expanded={entry.is_dir ? expanded : undefined}
      >
        {entry.is_dir ? (
          <span className={`ft-chevron ${expanded ? "expanded" : ""}`}>
            <ChevronRight size={14} />
          </span>
        ) : (
          <span className="ft-chevron-placeholder" />
        )}
        {entry.is_dir ? (
          <Icon icon={folderIcon} width={16} height={16} />
        ) : (
          <FileIcon name={entry.name} size={16} />
        )}
        <span className="ft-node-name">{entry.name}</span>
      </div>
      {entry.is_dir && children && (
        <div
          ref={childrenRef}
          className={`ft-children ${expanded ? "" : "collapsed"}`}
          style={{ maxHeight: expanded ? maxHeight : "0" }}
          role="group"
        >
          {children.map((child) => (
            <FileTreeNode
              key={child.path}
              entry={child}
              depth={depth + 1}
              maxDepth={maxDepth}
              expanded={expandedPaths.has(child.path)}
              active={child.path === activePath}
              children={childrenMap.get(child.path)}
              expandedPaths={expandedPaths}
              childrenMap={childrenMap}
              activePath={activePath}
              onToggle={onToggle}
              onSelect={onSelect}
            />
          ))}
          {children.length === 0 && expanded && (
            <div className="ft-empty" style={{ paddingLeft: (depth + 1) * 16 + 8 }}>—</div>
          )}
        </div>
      )}
    </div>
  );
}
