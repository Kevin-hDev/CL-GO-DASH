import { useState } from "react";
import { useTranslation } from "react-i18next";
import { CaretLeft, Pencil, Trash } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import type { ScheduledWakeup, WakeupRun, WakeupStatusSummary } from "@/types/wakeup";
import { formatDateTime, formatRunStatus, formatSchedule } from "@/lib/wakeup-format";
import { ActiveBadge } from "./badges";
import { WakeupHistory } from "./wakeup-history";

interface WakeupDetailsProps {
  wakeup: ScheduledWakeup;
  summary?: WakeupStatusSummary;
  runs: WakeupRun[];
  disableToggle: boolean;
  onBack: () => void;
  onToggle: (active: boolean) => void;
  onEdit: () => void;
  onDelete: () => void;
}

export function WakeupDetails({
  wakeup,
  summary,
  runs,
  disableToggle,
  onBack,
  onToggle,
  onEdit,
  onDelete,
}: WakeupDetailsProps) {
  const { t } = useTranslation();
  const [confirmDelete, setConfirmDelete] = useState(false);
  const lastRun = summary?.last_run ?? null;

  const handleDelete = () => {
    if (confirmDelete) {
      onDelete();
    } else {
      setConfirmDelete(true);
      window.setTimeout(() => setConfirmDelete(false), 3000);
    }
  };

  return (
    <div className="wk-details">
      <div className="wk-details-header">
        <Tooltip label={t("heartbeat.back")}>
          <button className="wk-back" onClick={onBack} type="button">
            <CaretLeft size="var(--icon-md)" weight="regular" />
          </button>
        </Tooltip>
        <div className="wk-details-title">
          <span className="wk-details-model">{wakeup.model}</span>
          {wakeup.provider !== "ollama" && (
            <span
              style={{
                fontSize: 10,
                padding: "2px 6px",
                background: "var(--surface)",
                borderRadius: 4,
                color: "var(--ink-muted)",
                textTransform: "uppercase",
                letterSpacing: "0.3px",
              }}
            >
              {wakeup.provider}
            </span>
          )}
          <ActiveBadge active={wakeup.active} />
        </div>
        <div className="wk-details-actions">
          <Tooltip label={t("heartbeat.edit")}>
            <button
              className="wk-icon-btn"
              onClick={onEdit}
              type="button"
            >
              <Pencil size="var(--icon-md)" />
            </button>
          </Tooltip>
          {confirmDelete ? (
            <button
              className="wk-confirm-delete"
              onClick={handleDelete}
              type="button"
            >
              <Trash size="var(--icon-sm)" />
              {t("heartbeat.confirmDelete")}
            </button>
          ) : (
            <Tooltip label={t("heartbeat.delete")}>
              <button
                className="wk-icon-btn wk-icon-btn-danger"
                onClick={handleDelete}
                type="button"
              >
                <Trash size="var(--icon-md)" />
              </button>
            </Tooltip>
          )}
          {disableToggle ? (
            <Tooltip label={t("heartbeat.pausedHint")} align="right">
              <button
                className="wk-toggle-pill"
                data-active={wakeup.active}
                disabled
                onClick={() => onToggle(!wakeup.active)}
                type="button"
              >
                <span className="wk-toggle-dot" />
              </button>
            </Tooltip>
          ) : (
            <Tooltip label={t("heartbeat.toggle")} align="right">
              <button
                className="wk-toggle-pill"
                data-active={wakeup.active}
                onClick={() => onToggle(!wakeup.active)}
                type="button"
              >
                <span className="wk-toggle-dot" />
              </button>
            </Tooltip>
          )}
        </div>
      </div>

      <div className="wk-details-body">
        <Field label={t("heartbeat.fields.name")} value={wakeup.name} />
        <Field label={t("heartbeat.fields.provider")} value={wakeup.provider} />
        <Field label={t("heartbeat.fields.model")} value={wakeup.model} />
        <Field label={t("heartbeat.fields.schedule")} value={formatSchedule(wakeup.schedule)} />
        <Field
          label={t("heartbeat.fields.nextFire")}
          value={formatDateTime(summary?.next_fire_at)}
        />
        <Field
          label={t("heartbeat.fields.lastStatus")}
          value={formatRunStatus(lastRun?.status)}
        />
        <Field
          label={t("heartbeat.fields.lastRun")}
          value={formatDateTime(lastRun?.fired_at)}
        />
        <Field label={t("heartbeat.fields.description")} value={wakeup.description || "—"} />
        <Field
          label={t("heartbeat.fields.prompt")}
          value={wakeup.prompt}
          multiline
        />
        <WakeupHistory runs={runs} />
      </div>
    </div>
  );
}

interface FieldProps {
  label: string;
  value: string;
  multiline?: boolean;
}

function Field({ label, value, multiline }: FieldProps) {
  return (
    <div className="wk-field">
      <div className="wk-field-label">{label}</div>
      <div className={multiline ? "wk-field-value wk-field-multi" : "wk-field-value"}>
        {value}
      </div>
    </div>
  );
}
