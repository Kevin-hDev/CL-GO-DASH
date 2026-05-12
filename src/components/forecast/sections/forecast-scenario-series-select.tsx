import { useTranslation } from "react-i18next";

interface ForecastScenarioSeriesSelectProps {
  seriesIds: string[];
  selectedSeries: string;
  onSelectedSeriesChange: (value: string) => void;
}

export function ForecastScenarioSeriesSelect({
  seriesIds,
  selectedSeries,
  onSelectedSeriesChange,
}: ForecastScenarioSeriesSelectProps) {
  const { t } = useTranslation();
  if (seriesIds.length <= 1) return null;

  return (
    <div className="fcs-series-bar">
      <label className="fcs-series-label" htmlFor="fcs-series-select">
        {t("forecast.view.series")}
      </label>
      <select
        id="fcs-series-select"
        className="fcs-series-select"
        value={selectedSeries || seriesIds[0]}
        onChange={(event) => onSelectedSeriesChange(event.target.value)}
      >
        {seriesIds.map((seriesId) => (
          <option key={seriesId} value={seriesId}>
            {seriesId}
          </option>
        ))}
      </select>
    </div>
  );
}
