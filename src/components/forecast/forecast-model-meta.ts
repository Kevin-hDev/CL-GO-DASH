export interface ForecastProviderEntry {
  id: string;
  display_name: string;
  configured?: boolean;
}

export interface ForecastModelEntry {
  id: string;
  provider_id: string;
  family_id?: string;
  display_name: string;
  params: string;
  size_mb: number;
  ram_mb: number;
  vram_mb: number | null;
  cpu_supported: boolean;
  gpu_supported: boolean;
  multivariate: boolean;
  covariates: boolean;
  horizon_max: number;
  frequencies: string;
  is_cloud: boolean;
  installed: boolean;
  size_on_disk?: number;
  provider_configured?: boolean;
  engine_kind?: "local_chronos_bolt" | "local_chronos2" | "cloud_api";
  capabilities?: {
    past_covariates: boolean;
    future_covariates: boolean;
    multivariate: boolean;
    probabilistic: boolean;
    backtesting_ready: boolean;
    anomalies_ready: boolean;
    fine_tuning_ready: boolean;
  };
}

export interface ForecastCapabilitySet {
  context: boolean;
  futureContext: boolean;
  multivariate: boolean;
  probabilistic: boolean;
  backtesting: boolean;
}

export interface ForecastModelsResponse {
  providers: ForecastProviderEntry[];
  models: ForecastModelEntry[];
  configured_provider_ids: string[];
}

export interface ForecastModelGroup {
  id: string;
  titleKey: string;
  models: ForecastModelEntry[];
}

const FAMILY_ORDER = ["chronos", "timegpt"];

export function getForecastFamilyId(model: ForecastModelEntry): string {
  if (model.family_id) return model.family_id;
  if (model.id.startsWith("chronos")) return "chronos";
  if (model.id.startsWith("timegpt")) return "timegpt";
  return model.provider_id;
}

export function getForecastFamilyKey(familyId: string): string {
  return `forecast.models.families.${familyId}`;
}

export function groupForecastModels(models: ForecastModelEntry[]): ForecastModelGroup[] {
  const grouped = new Map<string, ForecastModelEntry[]>();
  models.forEach((model) => {
    const familyId = getForecastFamilyId(model);
    grouped.set(familyId, [...(grouped.get(familyId) ?? []), model]);
  });
  return [...grouped.entries()]
    .sort((a, b) => sortFamily(a[0]) - sortFamily(b[0]))
    .map(([id, familyModels]) => ({
      id,
      titleKey: getForecastFamilyKey(id),
      models: familyModels.sort(sortModels),
    }));
}

export function isForecastModelSelectable(model: ForecastModelEntry): boolean {
  return model.is_cloud ? Boolean(model.provider_configured) : model.installed;
}

export function getForecastModelSummaryKey(modelId: string): string {
  return `forecast.models.descriptions.${modelId}`;
}

export function getForecastHardwareKey(model: ForecastModelEntry): string {
  if (model.is_cloud) return "forecast.models.hardware.remote";
  if (model.size_mb <= 64) return "forecast.models.hardware.light";
  if (model.size_mb <= 512) return "forecast.models.hardware.balanced";
  return "forecast.models.hardware.heavy";
}

export function getModelCapabilities(model: ForecastModelEntry): ForecastCapabilitySet {
  return {
    context: model.covariates,
    futureContext: model.covariates,
    multivariate: model.multivariate,
    probabilistic: true,
    backtesting: model.is_cloud,
  };
}

export function getForecastEngineKey(model: ForecastModelEntry): string {
  if (model.engine_kind === "local_chronos2") return "forecast.models.engines.localChronos2";
  if (model.engine_kind === "local_chronos_bolt") return "forecast.models.engines.localChronosBolt";
  return "forecast.models.engines.cloudApi";
}

function sortFamily(familyId: string): number {
  const index = FAMILY_ORDER.indexOf(familyId);
  return index === -1 ? FAMILY_ORDER.length : index;
}

function sortModels(a: ForecastModelEntry, b: ForecastModelEntry): number {
  if (a.is_cloud !== b.is_cloud) return a.is_cloud ? 1 : -1;
  if (a.is_cloud) return a.display_name.localeCompare(b.display_name);
  return a.size_mb - b.size_mb;
}
