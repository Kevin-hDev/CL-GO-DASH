// Types frontend alignés sur le backend Rust (src-tauri/src/services/api_keys.rs).
// Les provider_ids doivent rester cohérents entre Rust et TS.

export type LlmProviderId =
  | "groq"
  | "google"
  | "mistral"
  | "cerebras"
  | "openrouter"
  | "openai"
  | "deepseek";

export type SearchProviderId =
  | "brave"
  | "exa"
  | "firecrawl"
  | "serpapi"
  | "google_cse";

export type ProviderId = LlmProviderId | SearchProviderId | string;

export type ProviderCategory = "llm" | "search" | "scraping";

/**
 * Spec d'un provider (miroir du catalog Rust).
 */
export interface ProviderSpec {
  id: string;
  display_name: string;
  category: ProviderCategory;
  signup_url: string;
  free_tier_label: string;
  short_description: string;
  /** Pour les providers LLM — absent pour search/scraping */
  base_url?: string;
  models_endpoint?: string;
}

/**
 * Provider configuré (clé enregistrée dans le keystore OS).
 * Ne contient JAMAIS la clé en clair — juste l'identifiant.
 */
export interface ConfiguredProvider {
  id: ProviderId;
}

/**
 * Résultat d'un test de connexion.
 */
export type TestKeyResult =
  | { ok: true }
  | { ok: false; error: string };
