import { describe, expect, it } from "vitest";
import { forecastXAxisSplitNumber } from "./forecast-chart-option";

describe("forecastXAxisSplitNumber", () => {
  it("only changes the chart density when a width threshold is crossed", () => {
    expect(forecastXAxisSplitNumber(540)).toBe(4);
    expect(forecastXAxisSplitNumber(600)).toBe(4);
    expect(forecastXAxisSplitNumber(675)).toBe(5);
  });

  it("keeps the density within readable bounds", () => {
    expect(forecastXAxisSplitNumber(0)).toBe(4);
    expect(forecastXAxisSplitNumber(200)).toBe(3);
    expect(forecastXAxisSplitNumber(2000)).toBe(8);
  });
});
