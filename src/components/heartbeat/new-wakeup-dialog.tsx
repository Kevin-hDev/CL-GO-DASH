import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { X } from "@/components/ui/icons";
import type { CreateWakeupInput, ScheduledWakeup, WakeupSchedule } from "@/types/wakeup";
import { SchedulePicker } from "./schedule-picker";

interface OllamaModel {
  name: string;
}

interface NewWakeupDialogProps {
  initial: ScheduledWakeup | null;
  onClose: () => void;
  onCreate: (input: CreateWakeupInput) => Promise<void>;
  onUpdate: (wakeup: ScheduledWakeup) => Promise<void>;
}

function defaultSchedule(): WakeupSchedule {
  return { kind: "daily", time: "08:00" };
}

export function NewWakeupDialog({
  initial,
  onClose,
  onCreate,
  onUpdate,
}: NewWakeupDialogProps) {
  const { t } = useTranslation();
  const [name, setName] = useState(initial?.name ?? "");
  const [model, setModel] = useState(initial?.model ?? "");
  const [prompt, setPrompt] = useState(initial?.prompt ?? "");
  const [description, setDescription] = useState(initial?.description ?? "");
  const [schedule, setSchedule] = useState<WakeupSchedule>(
    initial?.schedule ?? defaultSchedule(),
  );
  const [models, setModels] = useState<string[]>([]);
  const [ollamaError, setOllamaError] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<OllamaModel[]>("list_ollama_models")
      .then((ms) => {
        setModels(ms.map((m) => m.name));
        setOllamaError(false);
      })
      .catch(() => {
        setModels([]);
        setOllamaError(true);
      });
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key.startsWith("Esc")) {
        e.preventDefault();
        onClose();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      if (initial) {
        await onUpdate({ ...initial, name, model, prompt, description, schedule });
      } else {
        await onCreate({ name, model, provider: "ollama", prompt, description, schedule });
      }
      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setSubmitting(false);
    }
  };

  const disabled = submitting || ollamaError;
  const title = initial ? t("heartbeat.form.editTitle") : t("heartbeat.form.createTitle");

  return (
    <div className="wk-dialog-overlay" onClick={onClose}>
      <div className="wk-dialog" onClick={(e) => e.stopPropagation()} role="dialog">
        <header className="wk-dialog-header">
          <span>{title}</span>
          <button type="button" className="wk-dialog-close" onClick={onClose}>
            <X size={16} />
          </button>
        </header>

        <form className="wk-form" onSubmit={handleSubmit}>
          <div className="wk-form-field">
            <label className="wk-form-label">{t("heartbeat.form.name")}</label>
            <input
              type="text"
              className="wk-input"
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
              autoFocus
            />
          </div>

          <div className="wk-form-row">
            <div className="wk-form-field">
              <label className="wk-form-label">{t("heartbeat.form.provider")}</label>
              <select className="wk-input" value="ollama" disabled>
                <option value="ollama">Ollama</option>
              </select>
            </div>

            <div className="wk-form-field">
              <label className="wk-form-label">{t("heartbeat.form.model")}</label>
              <select
                className="wk-input"
                value={model}
                onChange={(e) => setModel(e.target.value)}
                required
                disabled={ollamaError}
              >
                <option value="" disabled>
                  {ollamaError ? t("heartbeat.form.ollamaDown") : t("heartbeat.form.pickModel")}
                </option>
                {models.map((m) => (
                  <option key={m} value={m}>{m}</option>
                ))}
              </select>
            </div>
          </div>

          <div className="wk-form-field">
            <label className="wk-form-label">{t("heartbeat.form.prompt")}</label>
            <textarea
              className="wk-input wk-textarea"
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              rows={4}
              required
            />
          </div>

          <SchedulePicker value={schedule} onChange={setSchedule} />

          <div className="wk-form-field">
            <label className="wk-form-label">{t("heartbeat.form.description")}</label>
            <input
              type="text"
              className="wk-input"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              maxLength={200}
            />
          </div>

          {error && <div className="wk-form-error">{error}</div>}

          <footer className="wk-dialog-footer">
            <button type="button" className="wk-btn-secondary" onClick={onClose}>
              {t("heartbeat.form.cancel")}
            </button>
            <button type="submit" className="wk-btn-primary" disabled={disabled}>
              {initial ? t("heartbeat.form.save") : t("heartbeat.form.create")}
            </button>
          </footer>
        </form>
      </div>
    </div>
  );
}
