import { useState } from "react";
import { useTranslation } from "react-i18next";
import { CaretLeft, Pencil, Trash } from "@/components/ui/icons";
import type { ScheduledWakeup } from "@/types/wakeup";
import { formatSchedule } from "@/lib/wakeup-format";
import { ActiveBadge } from "./badges";

interface WakeupDetailsProps {
  wakeup: ScheduledWakeup;
  disableToggle: boolean;
  onBack: () => void;
  onToggle: (active: boolean) => void;
  onEdit: () => void;
  onDelete: () => void;
}

export function WakeupDetails({
  wakeup,
  disableToggle,
  onBack,
  onToggle,
  onEdit,
  onDelete,
}: WakeupDetailsProps) {
  const { t } = useTranslation();
  const [confirmDelete, setConfirmDelete] = useState(false);

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
        <button className="wk-back" onClick={onBack} type="button">
          <CaretLeft size={16} weight="regular" />
        </button>
        <div className="wk-details-title">
          <span className="wk-details-model">{wakeup.model}</span>
          <ActiveBadge active={wakeup.active} />
        </div>
        <div className="wk-details-actions">
          <button
            className="wk-icon-btn"
            onClick={onEdit}
            title={t("heartbeat.edit")}
            type="button"
          >
            <Pencil size={16} />
          </button>
          {confirmDelete ? (
            <button
              className="wk-confirm-delete"
              onClick={handleDelete}
              type="button"
            >
              <Trash size={14} />
              {t("heartbeat.confirmDelete")}
            </button>
          ) : (
            <button
              className="wk-icon-btn wk-icon-btn-danger"
              onClick={handleDelete}
              title={t("heartbeat.delete")}
              type="button"
            >
              <Trash size={16} />
            </button>
          )}
          <button
            className="wk-toggle-pill"
            data-active={wakeup.active}
            disabled={disableToggle}
            onClick={() => onToggle(!wakeup.active)}
            title={disableToggle ? t("heartbeat.pausedHint") : ""}
            type="button"
          >
            <span className="wk-toggle-dot" />
          </button>
        </div>
      </div>

      <div className="wk-details-body">
        <Field label={t("heartbeat.fields.name")} value={wakeup.name} />
        <Field label={t("heartbeat.fields.provider")} value={wakeup.provider} />
        <Field label={t("heartbeat.fields.model")} value={wakeup.model} />
        <Field label={t("heartbeat.fields.schedule")} value={formatSchedule(wakeup.schedule)} />
        <Field label={t("heartbeat.fields.description")} value={wakeup.description || "—"} />
        <Field
          label={t("heartbeat.fields.prompt")}
          value={wakeup.prompt}
          multiline
        />
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
