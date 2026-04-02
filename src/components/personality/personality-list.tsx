import type { PersonalityFile } from "@/types/personality";
import "./personality-list.css";

interface PersonalityListProps {
  files: PersonalityFile[];
  selectedPath: string | null;
  onSelect: (path: string) => void;
}

export function PersonalityList({
  files,
  selectedPath,
  onSelect,
}: PersonalityListProps) {
  return (
    <>
      <div className="pers-header">
        <span className="pers-title">Fichiers</span>
      </div>
      <div className="pers-content">
        {files.map((f) => (
          <div
            key={f.name}
            className={`pers-item ${selectedPath === f.path ? "active" : ""}`}
            onClick={() => onSelect(f.path)}
          >
            <div className="pers-icon">📄</div>
            <div className="pers-item-body">
              <div className="pers-item-name">{f.name}</div>
              <div className="pers-item-desc">{f.description}</div>
            </div>
          </div>
        ))}
      </div>
    </>
  );
}
