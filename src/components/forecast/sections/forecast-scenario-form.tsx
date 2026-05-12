import type { FormEvent } from "react";
import { useTranslation } from "react-i18next";

interface ForecastScenarioFormProps {
  name: string;
  description: string;
  adjustment: string;
  editing: boolean;
  saving: boolean;
  error: string | null;
  onNameChange: (value: string) => void;
  onDescriptionChange: (value: string) => void;
  onAdjustmentChange: (value: string) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  onCancel: () => void;
}

export function ForecastScenarioForm(props: ForecastScenarioFormProps) {
  const { t } = useTranslation();

  return (
    <form className="fcs-form" onSubmit={(event) => void props.onSubmit(event)}>
      <input
        className="fcs-input"
        value={props.name}
        maxLength={80}
        onChange={(event) => props.onNameChange(event.target.value)}
        placeholder={t("forecast.scenarios.name")}
      />
      <input
        className="fcs-input"
        value={props.description}
        maxLength={500}
        onChange={(event) => props.onDescriptionChange(event.target.value)}
        placeholder={t("forecast.scenarios.description")}
      />
      <div className="fcs-form-row">
        <input
          className="fcs-input fcs-input-number"
          type="number"
          min="-95"
          max="500"
          step="0.1"
          value={props.adjustment}
          onChange={(event) => props.onAdjustmentChange(event.target.value)}
          aria-label={t("forecast.scenarios.adjustment")}
        />
        <button
          className={`fcs-add-btn ${props.editing ? "is-primary" : ""}`}
          type="submit"
          disabled={props.saving || !props.name.trim()}
        >
          {props.saving ? t("forecast.scenarios.saving") : props.editing ? t("forecast.scenarios.update") : t("forecast.scenarios.add")}
        </button>
        {props.editing && (
          <button className="fcs-add-btn" type="button" onClick={props.onCancel}>
            {t("forecast.scenarios.cancel")}
          </button>
        )}
      </div>
      {props.error && <p className="fcs-error">{props.error}</p>}
    </form>
  );
}
