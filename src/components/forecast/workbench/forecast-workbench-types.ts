export interface ForecastWorkbenchContext {
  session_id: string;
  analysis_id: string | null;
  revision: number;
}

export interface ForecastWorkbenchSnapshot {
  context: ForecastWorkbenchContext;
  session_name: string;
  analysis_name: string | null;
}

export type ForecastWorkbenchSection =
  | "data"
  | "forecast"
  | "evaluation"
  | "comparison"
  | "scenarios"
  | "report";

export function isForecastWorkbenchSnapshot(
  value: unknown,
): value is ForecastWorkbenchSnapshot {
  if (!value || typeof value !== "object") return false;
  const snapshot = value as Partial<ForecastWorkbenchSnapshot>;
  const context = snapshot.context as Partial<ForecastWorkbenchContext> | undefined;
  const validId = (id: unknown) => typeof id === "string" &&
    id.length > 0 && id.length <= 64 && /^[a-f0-9-]+$/.test(id);
  return Boolean(context) &&
    validId(context?.session_id) &&
    (context?.analysis_id === null || validId(context?.analysis_id)) &&
    typeof context?.revision === "number" &&
    Number.isSafeInteger(context.revision) &&
    context.revision > 0 &&
    typeof snapshot.session_name === "string" &&
    snapshot.session_name.length <= 120 &&
    (snapshot.analysis_name === null || (
      typeof snapshot.analysis_name === "string" && snapshot.analysis_name.length <= 120
    ));
}
