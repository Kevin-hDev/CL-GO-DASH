import { useTranslation } from "react-i18next";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { IS_MAC } from "@/lib/platform";

interface PathListEditorProps {
  paths: string[];
  onChange: (paths: string[]) => void;
}

const DEFAULT_PATH = IS_MAC || navigator.userAgent.includes("Linux") ? "/" : "C:\\";

function formatLabel(path: string, t: (key: string) => string): string {
  if (path === "/" || path === "C:\\") return t("settings.advanced.fullDisk");
  return path;
}

export function PathListEditor({ paths, onChange }: PathListEditorProps) {
  const { t } = useTranslation();

  const handleAdd = async () => {
    const selected = await openFileDialog({ directory: true });
    if (typeof selected === "string" && !paths.includes(selected)) {
      onChange([...paths, selected]);
    }
  };

  const handleRemove = (index: number) => {
    onChange(paths.filter((_, i) => i !== index));
  };

  const handleReset = () => {
    onChange([DEFAULT_PATH]);
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
      {paths.map((p, i) => (
        <div key={p} style={{
          display: "flex",
          alignItems: "center",
          gap: 8,
          padding: "6px 10px",
          background: "var(--surface)",
          borderRadius: "var(--radius-sm)",
          border: "1px solid var(--edge)",
        }}>
          <span style={{
            flex: 1,
            fontSize: "var(--text-sm)",
            color: "var(--ink)",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}>
            {formatLabel(p, t)}
          </span>
          <span style={{
            fontSize: "var(--text-xs)",
            color: "var(--ink-muted)",
            flexShrink: 0,
            marginRight: 4,
          }}>
            {p !== formatLabel(p, t) ? "" : p}
          </span>
          <button
            onClick={() => handleRemove(i)}
            style={{
              background: "none",
              border: "none",
              cursor: "pointer",
              color: "var(--ink-muted)",
              fontSize: 16,
              lineHeight: 1,
              padding: "0 2px",
              flexShrink: 0,
            }}
            title={t("common.delete")}
          >
            ×
          </button>
        </div>
      ))}

      <div style={{ display: "flex", gap: 8, marginTop: 4 }}>
        <button onClick={handleAdd} style={{
          padding: "6px 14px",
          fontSize: "var(--text-sm)",
          fontWeight: 500,
          color: "var(--ink)",
          background: "var(--surface)",
          border: "1px solid var(--edge)",
          borderRadius: "var(--radius-sm)",
          cursor: "pointer",
        }}>
          + {t("settings.advanced.addPath")}
        </button>
        <button onClick={handleReset} style={{
          padding: "6px 14px",
          fontSize: "var(--text-sm)",
          fontWeight: 500,
          color: "var(--ink-muted)",
          background: "none",
          border: "1px solid var(--edge)",
          borderRadius: "var(--radius-sm)",
          cursor: "pointer",
        }}>
          {t("settings.advanced.resetPaths")}
        </button>
      </div>
    </div>
  );
}
