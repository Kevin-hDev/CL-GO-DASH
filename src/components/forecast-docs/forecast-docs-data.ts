import overview from "@/content/forecast-docs/overview.md?raw";
import datasets from "@/content/forecast-docs/datasets.md?raw";
import models from "@/content/forecast-docs/models.md?raw";
import predictions from "@/content/forecast-docs/predictions.md?raw";
import uncertainty from "@/content/forecast-docs/uncertainty.md?raw";
import covariates from "@/content/forecast-docs/covariates.md?raw";
import multiseries from "@/content/forecast-docs/multiseries.md?raw";
import scenarios from "@/content/forecast-docs/scenarios.md?raw";
import agentWorkflows from "@/content/forecast-docs/agent-workflows.md?raw";
import toolContracts from "@/content/forecast-docs/tool-contracts.md?raw";
import troubleshooting from "@/content/forecast-docs/troubleshooting.md?raw";
import limits from "@/content/forecast-docs/limits.md?raw";
import { parseForecastDoc } from "./forecast-docs-parser";
import type { ForecastDocPage } from "./forecast-docs-types";

const DOC_SOURCES = [
  ["overview", "Vue d'ensemble", overview],
  ["datasets", "Datasets", datasets],
  ["models", "Modèles", models],
  ["predictions", "Prévisions", predictions],
  ["uncertainty", "Incertitude", uncertainty],
  ["covariates", "Covariables", covariates],
  ["multiseries", "Multi-séries", multiseries],
  ["scenarios", "Scénarios", scenarios],
  ["agent-workflows", "Agents LLM", agentWorkflows],
  ["tool-contracts", "Tools Forecast", toolContracts],
  ["troubleshooting", "Diagnostic", troubleshooting],
  ["limits", "Limites", limits],
] as const;

export const FORECAST_DOC_PAGES: ForecastDocPage[] = DOC_SOURCES.map(
  ([id, navLabel, markdown]) => parseForecastDoc({ id, navLabel, markdown }),
);
