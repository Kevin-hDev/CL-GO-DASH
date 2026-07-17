export type OAuthProviderId = "openai" | "moonshot" | "xai";

export type OAuthClientState = "ready" | "missing" | "incompatible";

export interface OAuthProviderStatus {
  id: OAuthProviderId;
  display_name: string;
  connected: boolean;
  account: string | null;
  client_state: OAuthClientState;
  install_url: string;
}

export interface OAuthLoginProgress {
  provider_id: OAuthProviderId;
  stage: "waiting" | "verification" | "success" | "cancelled" | "error";
  hint?: string;
  verification_url?: string;
  user_code?: string;
}
