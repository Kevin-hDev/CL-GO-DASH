export interface SeasonalityPoint {
  date: string;
  value: number;
}

export interface SeasonalityYear {
  year: number;
  values: (number | null)[];
  complete: boolean;
  emphasized: boolean;
}

export interface SeasonalityModel {
  months: string[];
  years: SeasonalityYear[];
}

export const SEASONALITY_MIN_YEAR_POINTS = 3;
/** Gate for rendering the seasonality card at all (> 24 history points). */
export const SEASONALITY_MIN_HISTORY = 24;
/** Complete years shown by default (plus the current partial year). */
export const SEASONALITY_DEFAULT_VISIBLE_YEARS = 5;
const COMPLETE_YEAR_POINTS = 12;

/**
 * Default visible subset: the last SEASONALITY_DEFAULT_VISIBLE_YEARS
 * complete years, plus the trailing partial year when there is one.
 */
export function defaultVisibleYears(years: SeasonalityYear[]): number[] {
  const visible = years
    .filter((entry) => entry.complete)
    .slice(-SEASONALITY_DEFAULT_VISIBLE_YEARS)
    .map((entry) => entry.year);
  const last = years[years.length - 1];
  if (last && !last.complete && !visible.includes(last.year)) {
    visible.push(last.year);
  }
  return visible;
}

export function toggleVisibleYear(visible: number[], year: number): number[] {
  if (visible.includes(year)) return visible.filter((entry) => entry !== year);
  return [...visible, year].sort((left, right) => left - right);
}

/**
 * Groups history by calendar year and normalizes each year to its first
 * available month (= 100), so recurring patterns line up on one axis.
 * Years with fewer than SEASONALITY_MIN_YEAR_POINTS points are skipped.
 */
export function buildSeasonalityModel(
  points: SeasonalityPoint[],
  locale: string,
): SeasonalityModel | null {
  const byYear = new Map<number, Map<number, number>>();
  for (const point of points) {
    const parsed = new Date(point.date);
    if (Number.isNaN(parsed.getTime()) || !Number.isFinite(point.value)) continue;
    const year = parsed.getFullYear();
    const month = parsed.getMonth();
    const entry = byYear.get(year) ?? new Map<number, number>();
    if (!entry.has(month)) entry.set(month, point.value);
    byYear.set(year, entry);
  }

  const years: SeasonalityYear[] = [];
  for (const [year, months] of [...byYear.entries()].sort((a, b) => a[0] - b[0])) {
    if (months.size < SEASONALITY_MIN_YEAR_POINTS) continue;
    const base = [...months.entries()].sort((a, b) => a[0] - b[0])[0][1];
    if (!Number.isFinite(base) || base === 0) continue;
    const values = Array.from({ length: 12 }, (_, month) => {
      const value = months.get(month);
      return value == null ? null : (value / base) * 100;
    });
    years.push({
      year,
      values,
      complete: months.size >= COMPLETE_YEAR_POINTS,
      emphasized: false,
    });
  }
  if (!years.length) return null;

  const emphasized =
    [...years].reverse().find((entry) => entry.complete) ?? years[years.length - 1];
  emphasized.emphasized = true;

  return { months: monthNames(locale), years };
}

export function monthNames(locale: string): string[] {
  const formatter = new Intl.DateTimeFormat(locale, { month: "short" });
  return Array.from({ length: 12 }, (_, month) =>
    formatter.format(new Date(2024, month, 1)),
  );
}
