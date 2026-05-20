import type { AvailableModel } from "@/hooks/use-available-models";

export type ReasoningMode = "off" | "auto" | "low" | "medium" | "high" | "xhigh";

export interface ReasoningModeOption {
  mode: ReasoningMode;
  labelKey: string;
}

const LABELS: Record<ReasoningMode, string> = {
  off: "agentLocal.reasoningOff",
  auto: "agentLocal.reasoningAuto",
  low: "agentLocal.reasoningLow",
  medium: "agentLocal.reasoningMedium",
  high: "agentLocal.reasoningHigh",
  xhigh: "agentLocal.reasoningXhigh",
};

function option(mode: ReasoningMode): ReasoningModeOption {
  return { mode, labelKey: LABELS[mode] };
}

function options(modes: ReasoningMode[]): ReasoningModeOption[] {
  return modes.map(option);
}

function modelName(model: AvailableModel | null): string {
  return model?.id.toLowerCase() ?? "";
}

function isGptOss(model: AvailableModel | null): boolean {
  return modelName(model).includes("gpt-oss");
}

function isGrokMultiAgent(model: AvailableModel | null): boolean {
  const name = modelName(model);
  return name.includes("reasoning") || name.includes("multi-agent") || name.includes("4.20-reasoning");
}

export function reasoningModeOptions(model: AvailableModel | null): ReasoningModeOption[] {
  if (!model?.supports_thinking) return [];
  switch (model.provider_id) {
    case "codex-oauth":
      return options(["low", "medium", "high", "xhigh"]);
    case "ollama":
      return isGptOss(model)
        ? options(["low", "medium", "high"])
        : options(["off", "auto"]);
    case "openai":
      return options(["off", "low", "medium", "high", "xhigh"]);
    case "xai":
      return isGrokMultiAgent(model)
        ? options(["low", "medium", "high", "xhigh"])
        : options(["off", "low", "medium", "high"]);
    case "mistral":
      return options(["off", "high"]);
    case "zai":
      return options(["off", "auto"]);
    default:
      return options(["off", "auto"]);
  }
}

export function normalizeReasoningMode(
  requested: string | null | undefined,
  options: ReasoningModeOption[],
): ReasoningMode | null {
  if (options.length === 0) return null;
  if (requested && options.some((option) => option.mode === requested)) {
    return requested as ReasoningMode;
  }
  if (options.some((option) => option.mode === "medium")) return "medium";
  return options.some((option) => option.mode === "off") ? "off" : options[0].mode;
}
