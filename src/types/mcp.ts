export type McpCategory =
  | "productivity"
  | "design"
  | "devtools"
  | "communication"
  | "ai-ml"
  | "scraping"
  | "community";

export type McpAuthType = "oauth" | "token" | "none";

export interface McpConnectorSpec {
  id: string;
  display_name: string;
  category: McpCategory;
  auth_type: McpAuthType;
  short_description: string;
  short_description_en: string;
  author: string;
  url: string;
  tools: string[];
  endpoint?: string;
  install_command?: string;
  env_keys?: string[];
  os_restrict?: "macos";
  coming_soon?: boolean;
}

export type McpConnectorStatus = "connected" | "disconnected";

export interface ConfiguredMcp {
  id: string;
  status: McpConnectorStatus;
  enabled_in_chat: boolean;
  endpoint?: string;
  install_command?: string;
  env_keys?: string[];
}

export type ConfiguredMcpFull = McpConnectorSpec & ConfiguredMcp;

export function getMcpDescription(spec: McpConnectorSpec, lang: string): string {
  return lang === "fr" ? spec.short_description : spec.short_description_en;
}
