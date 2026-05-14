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
  ["overview", "forecast.docs.nav.overview", overview],
  ["datasets", "forecast.docs.nav.datasets", datasets],
  ["models", "forecast.docs.nav.models", models],
  ["predictions", "forecast.docs.nav.predictions", predictions],
  ["uncertainty", "forecast.docs.nav.uncertainty", uncertainty],
  ["covariates", "forecast.docs.nav.covariates", covariates],
  ["multiseries", "forecast.docs.nav.multiseries", multiseries],
  ["scenarios", "forecast.docs.nav.scenarios", scenarios],
  ["agent-workflows", "forecast.docs.nav.agentWorkflows", agentWorkflows],
  ["tool-contracts", "forecast.docs.nav.toolContracts", toolContracts],
  ["troubleshooting", "forecast.docs.nav.troubleshooting", troubleshooting],
  ["limits", "forecast.docs.nav.limits", limits],
] as const;

export const FORECAST_DOC_PAGES: ForecastDocPage[] = DOC_SOURCES.map(
  ([id, navLabel, markdown]) => parseForecastDoc({ id, navLabel, markdown }),
);
