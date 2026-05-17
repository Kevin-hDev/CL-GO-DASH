import { describe, expect, it } from "vitest";
import {
  clampForecastZoomWindow,
  forecastZoomSliderValue,
  FORECAST_CHART_MIN_ZOOM_SPAN,
} from "../forecast-chart-zoom-utils";

describe("forecast chart zoom utils", () => {
  it("borne le zoom au minimum visible par la barre", () => {
    const zoom = clampForecastZoomWindow(45, 46);
    expect(zoom.end - zoom.start).toBe(FORECAST_CHART_MIN_ZOOM_SPAN);
  });

  it("garde une fenetre trop zoomee dans les bornes du graphe", () => {
    expect(clampForecastZoomWindow(98, 99)).toEqual({ start: 85, end: 100 });
  });

  it("conserve le dezoom complet", () => {
    expect(clampForecastZoomWindow(-10, 130)).toEqual({ start: 0, end: 100 });
  });

  it("borne la valeur visuelle du slider", () => {
    expect(forecastZoomSliderValue(1)).toBe(100 - FORECAST_CHART_MIN_ZOOM_SPAN);
  });
});
