import { describe, expect, it } from "vitest";
import {
  buildSeasonalityModel,
  monthNames,
  SEASONALITY_MIN_YEAR_POINTS,
} from "./forecast-seasonality-data";

function point(year: number, month: number, value: number) {
  return { date: new Date(year, month, 15).toISOString(), value };
}

function yearPoints(year: number, months: number[], base = 100) {
  return months.map((month) => point(year, month, base * (1 + month / 100)));
}

describe("buildSeasonalityModel", () => {
  it("groups by calendar year and normalizes the first month to 100", () => {
    const model = buildSeasonalityModel(
      [...yearPoints(2023, [0, 1, 2, 3]), ...yearPoints(2024, [0, 1, 2, 3])],
      "en",
    );

    expect(model?.years).toHaveLength(2);
    expect(model?.years[0].values[0]).toBe(100);
    expect(model?.years[0].values[1]).toBeCloseTo((101 / 100) * 100);
    expect(model?.years[0].values[11]).toBeNull();
  });

  it("skips years with too few points", () => {
    const model = buildSeasonalityModel(
      [
        ...yearPoints(2022, [10, 11]),
        ...yearPoints(2023, [0, 1, 2]),
        ...yearPoints(2024, [0, 1, 2, 3, 4]),
      ],
      "en",
    );

    expect(model?.years.map((entry) => entry.year)).toEqual([2023, 2024]);
    expect(SEASONALITY_MIN_YEAR_POINTS).toBe(3);
  });

  it("skips years whose base value is zero", () => {
    const model = buildSeasonalityModel(
      [point(2023, 0, 0), point(2023, 1, 5), point(2023, 2, 6), ...yearPoints(2024, [0, 1, 2])],
      "en",
    );

    expect(model?.years.map((entry) => entry.year)).toEqual([2024]);
  });

  it("emphasizes the most recent complete year", () => {
    const model = buildSeasonalityModel(
      [
        ...yearPoints(2023, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
        ...yearPoints(2024, [0, 1, 2]),
      ],
      "en",
    );

    expect(model?.years[0].complete).toBe(true);
    expect(model?.years[0].emphasized).toBe(true);
    expect(model?.years[1].emphasized).toBe(false);
  });

  it("falls back to the most recent year when none is complete", () => {
    const model = buildSeasonalityModel(
      [...yearPoints(2023, [0, 1, 2]), ...yearPoints(2024, [0, 1, 2, 3])],
      "en",
    );

    expect(model?.years[1].emphasized).toBe(true);
  });

  it("returns null without usable years", () => {
    expect(buildSeasonalityModel([], "en")).toBeNull();
    expect(buildSeasonalityModel(yearPoints(2024, [0, 1]), "en")).toBeNull();
  });

  it("ignores invalid dates", () => {
    const model = buildSeasonalityModel(
      [{ date: "not-a-date", value: 1 }, ...yearPoints(2024, [0, 1, 2])],
      "en",
    );
    expect(model?.years).toHaveLength(1);
  });
});

describe("monthNames", () => {
  it("returns 12 locale-aware short month names", () => {
    expect(monthNames("en")).toHaveLength(12);
    expect(monthNames("en")[0]).toBe("Jan");
    expect(monthNames("fr")[0]).toBe("janv.");
  });
});
