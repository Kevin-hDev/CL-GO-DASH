import type { ForecastChartPalette } from "./forecast-chart-types";

export function buildForecastChartPalette(root: CSSStyleDeclaration): ForecastChartPalette {
  return {
    lineHistory: root.getPropertyValue("--fc-line-history").trim(),
    linePredict: root.getPropertyValue("--fc-line-predict").trim(),
    pointPredict: root.getPropertyValue("--fc-point-predict").trim(),
    band90: root.getPropertyValue("--fc-band-90").trim(),
    separator: root.getPropertyValue("--fc-separator").trim(),
    annotationUser: root.getPropertyValue("--pulse").trim(),
    annotationLlm: root.getPropertyValue("--fc-annotation").trim(),
    edge: root.getPropertyValue("--edge").trim(),
    inkMuted: root.getPropertyValue("--ink-faint").trim(),
    scenarios: readPalette(root, ["--fc-scenario-a", "--fc-scenario-b", "--fc-scenario-c"]),
    variables: readPalette(root, [
      "--fc-variable-a",
      "--fc-variable-b",
      "--fc-variable-c",
      "--fc-variable-d",
    ]),
  };
}

function readPalette(root: CSSStyleDeclaration, tokens: string[]): string[] {
  return tokens.map((token) => root.getPropertyValue(token).trim()).filter(Boolean);
}
