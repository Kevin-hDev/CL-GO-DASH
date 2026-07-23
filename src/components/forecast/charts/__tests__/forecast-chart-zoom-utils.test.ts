import { describe, expect, it } from "vitest";
import {
  clampForecastZoomWindow,
  forecastZoomSliderValue,
  isFullExtentZoomWindow,
  shouldIgnoreRoamAtFullExtent,
  FORECAST_CHART_MIN_ZOOM_SPAN,
} from "../forecast-chart-zoom-utils";

describe("forecast chart zoom utils", () => {
  it("borne le zoom au minimum visible par la barre", () => {
    const zoom = clampForecastZoomWindow(45, 46);
    expect(zoom.end - zoom.start).toBe(FORECAST_CHART_MIN_ZOOM_SPAN);
  });

  it("garde une fenetre trop zoomee dans les bornes du graphe", () => {
    expect(clampForecastZoomWindow(98, 99)).toEqual({ start: 90, end: 100 });
  });

  it("conserve le dezoom complet", () => {
    expect(clampForecastZoomWindow(-10, 130)).toEqual({ start: 0, end: 100 });
  });

  it("borne la valeur visuelle du slider", () => {
    expect(forecastZoomSliderValue(1)).toBe(100 - FORECAST_CHART_MIN_ZOOM_SPAN);
  });
});

describe("isFullExtentZoomWindow", () => {
  it("reconnait la fenetre pleine etendue", () => {
    expect(isFullExtentZoomWindow({ start: 0, end: 100 })).toBe(true);
    expect(isFullExtentZoomWindow({ start: 0, end: 99.9 })).toBe(false);
    expect(isFullExtentZoomWindow({ start: 0.5, end: 99.5 })).toBe(false);
    expect(isFullExtentZoomWindow({ start: 20, end: 80 })).toBe(false);
  });
});

describe("shouldIgnoreRoamAtFullExtent", () => {
  it("ignore le zoom molette entrant depuis la pleine etendue", () => {
    expect(
      shouldIgnoreRoamAtFullExtent({ start: 0, end: 100 }, { start: 18, end: 100 }),
    ).toBe(true);
  });

  it("laisse passer les cas sans zoom effectif ou hors pleine etendue", () => {
    expect(
      shouldIgnoreRoamAtFullExtent({ start: 0, end: 100 }, { start: 0, end: 100 }),
    ).toBe(false);
    expect(
      shouldIgnoreRoamAtFullExtent({ start: 10, end: 90 }, { start: 20, end: 80 }),
    ).toBe(false);
    expect(
      shouldIgnoreRoamAtFullExtent({ start: 10, end: 90 }, { start: 0, end: 100 }),
    ).toBe(false);
  });
});
