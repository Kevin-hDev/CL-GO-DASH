import { useTranslation } from "react-i18next";
import { ConfirmButton } from "@/components/settings/confirm-button";
import { SettingsCard } from "@/components/settings/settings-card";
import "./ollama.css";

interface ModelfileViewProps {
  modelName: string;
  systemPrompt: string;
  parameters: { key: string; value: string }[];
  modelfile: string;
  deleting: boolean;
  onDelete: () => void;
  onEditSystem: () => void;
  onEditParameters: () => void;
  onEditModelfile: () => void;
}

export function ModelfileView({
  modelName,
  systemPrompt,
  parameters,
  modelfile,
  deleting,
  onDelete,
  onEditSystem,
  onEditParameters,
  onEditModelfile,
}: ModelfileViewProps) {
  const { t } = useTranslation();

  return (
    <div style={{ padding: 24, overflowY: "auto", flex: 1 }}>
      <div style={{ maxWidth: 600, width: "100%", margin: "0 auto" }}>
        <div style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          marginBottom: 28,
        }}>
          <h2 style={{
            fontSize: "var(--text-xl)",
            fontWeight: 700,
            color: "var(--ink)",
            margin: 0,
          }}>
            {modelName}
          </h2>
          <div style={{ display: "flex", gap: 8 }}>
            <ConfirmButton
              className="ollama-btn"
              label={t("ollama.remove")}
              confirmLabel={t("settings.confirm.deleteModel")}
              onConfirm={onDelete}
              disabled={deleting}
            />
            <button className="ollama-btn" onClick={onEditModelfile}>
              {t("ollama.editModelfile")}
            </button>
          </div>
        </div>

        <SettingsCard>
          <ViewSection title={t("ollama.systemPrompt")} editLabel={t("ollama.edit")} onEdit={onEditSystem}>
            <div style={{
              fontSize: "var(--text-sm)",
              color: systemPrompt ? "var(--ink)" : "var(--ink-faint)",
              whiteSpace: "pre-wrap",
              lineHeight: 1.5,
              fontStyle: systemPrompt ? "normal" : "italic",
              maxHeight: 200,
              overflow: "auto",
            }}>
              {systemPrompt || t("ollama.noSystemPrompt")}
            </div>
          </ViewSection>

          <ViewSection title={t("ollama.parameters")} editLabel={t("ollama.edit")} onEdit={onEditParameters} last>
            {parameters.length === 0 ? (
              <div style={{ fontStyle: "italic", color: "var(--ink-faint)", fontSize: "var(--text-sm)" }}>
                {t("ollama.noParameters")}
              </div>
            ) : (
              <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                {parameters.map((p, i) => (
                  <div key={i} style={{
                    display: "flex", gap: 12,
                    fontSize: "var(--text-xs)",
                    fontFamily: "var(--font-mono)",
                  }}>
                    <span style={{ color: "var(--ink-muted)", minWidth: 140 }}>{p.key}</span>
                    <span style={{ color: "var(--ink)" }}>{p.value}</span>
                  </div>
                ))}
              </div>
            )}
          </ViewSection>
        </SettingsCard>

        <pre style={{
          marginTop: 16, padding: "var(--space-md)",
          fontSize: "var(--text-xs)", fontFamily: "var(--font-mono)",
          color: "var(--ink-faint)", whiteSpace: "pre-wrap",
          background: "var(--shell)", borderRadius: "var(--radius-md)",
          border: "1px solid var(--edge)",
        }}>
          {modelfile}
        </pre>
      </div>
    </div>
  );
}

function ViewSection({
  title, editLabel, onEdit, children, last,
}: {
  title: string; editLabel: string; onEdit: () => void;
  children: React.ReactNode; last?: boolean;
}) {
  return (
    <div style={{ padding: "12px 20px", borderBottom: last ? "none" : "1px solid var(--edge)" }}>
      <div style={{
        display: "flex", alignItems: "center",
        justifyContent: "space-between", marginBottom: "var(--space-sm)",
      }}>
        <span style={{ fontSize: "var(--text-sm)", fontWeight: 600, color: "var(--ink)" }}>
          {title}
        </span>
        <button className="ollama-btn ollama-btn-primary" onClick={onEdit}>
          {editLabel}
        </button>
      </div>
      {children}
    </div>
  );
}
