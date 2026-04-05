import { useTranslation } from "react-i18next";
import type { PersonalityFile } from "@/types/personality";
import { FileText } from "@/components/ui/icons";
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
  const { t } = useTranslation();
  return (
    <>
      <div className="pers-header">
        <span className="pers-title">{t("personality.files")}</span>
      </div>
      <div className="pers-content">
        {files.map((f) => (
          <div
            key={f.name}
            className={`pers-item ${selectedPath === f.path ? "active" : ""}`}
            onClick={() => onSelect(f.path)}
          >
            <div className="pers-icon"><FileText size={16} weight="duotone" /></div>
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
