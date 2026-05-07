import { describe, it, expect } from "vitest";
import {
  isSubagentInjectedMessage,
  extractSubagentsFromMessages,
} from "./subagent-message-utils";
import type { AgentMessage } from "@/types/agent";

// ─── Helpers ────────────────────────────────────────────────────────────────

function makeMsg(
  content: string,
  role: AgentMessage["role"] = "user",
): AgentMessage {
  return {
    id: "test-id",
    role,
    content,
    files: [],
    timestamp: new Date().toISOString(),
  };
}

// ─── isSubagentInjectedMessage ───────────────────────────────────────────────

describe("isSubagentInjectedMessage", () => {
  it("retourne true pour un message contenant le préfixe de rapport", () => {
    const msg = makeMsg('[Rapport du sous-agent "monAgent" — terminé — sid:abc123]\nContenu du rapport.');
    expect(isSubagentInjectedMessage(msg)).toBe(true);
  });

  it("retourne true pour un message de synthèse finale", () => {
    const msg = makeMsg("Tous les sous-agents ont terminé. Voici la synthèse.");
    expect(isSubagentInjectedMessage(msg)).toBe(true);
  });

  it("retourne false pour un message utilisateur normal", () => {
    const msg = makeMsg("Bonjour, peux-tu m'aider avec ce problème ?");
    expect(isSubagentInjectedMessage(msg)).toBe(false);
  });

  it("retourne false pour un message vide", () => {
    const msg = makeMsg("");
    expect(isSubagentInjectedMessage(msg)).toBe(false);
  });

  it("retourne false si le message est du rôle assistant même avec le bon préfixe", () => {
    const msg = makeMsg('[Rapport du sous-agent "x" — terminé — sid:y]', "assistant");
    expect(isSubagentInjectedMessage(msg)).toBe(false);
  });

  it("retourne false si le rôle est tool", () => {
    const msg = makeMsg("Tous les sous-agents ont terminé.", "tool");
    expect(isSubagentInjectedMessage(msg)).toBe(false);
  });
});

// ─── extractSubagentsFromMessages ───────────────────────────────────────────

describe("extractSubagentsFromMessages", () => {
  it("extrait correctement sessionId, name et status depuis un rapport terminé", () => {
    const msgs = [
      makeMsg('[Rapport du sous-agent "explorer-1" — terminé — sid:session-abc]\nResume de l\'agent.'),
    ];
    const result = extractSubagentsFromMessages(msgs);

    expect(result).toHaveLength(1);
    expect(result[0].sessionId).toBe("session-abc");
    expect(result[0].name).toBe("explorer-1");
    expect(result[0].status).toBe("completed");
  });

  it("extrait correctement le status échoué", () => {
    const msgs = [
      makeMsg('[Rapport du sous-agent "coder-2" — échoué — sid:session-xyz]\nErreur rencontrée.'),
    ];
    const result = extractSubagentsFromMessages(msgs);

    expect(result).toHaveLength(1);
    expect(result[0].status).toBe("failed");
    expect(result[0].name).toBe("coder-2");
    expect(result[0].sessionId).toBe("session-xyz");
  });

  it("retourne un tableau vide si aucun message ne correspond", () => {
    const msgs = [
      makeMsg("Message normal sans rapport."),
      makeMsg("Tous les sous-agents ont terminé."),
    ];
    const result = extractSubagentsFromMessages(msgs);
    expect(result).toHaveLength(0);
  });

  it("retourne un tableau vide pour une liste de messages vide", () => {
    expect(extractSubagentsFromMessages([])).toHaveLength(0);
  });

  it("gère plusieurs rapports dans des messages différents", () => {
    const msgs = [
      makeMsg('[Rapport du sous-agent "agent-A" — terminé — sid:sid-1]\nOK'),
      makeMsg("Message intermédiaire normal."),
      makeMsg('[Rapport du sous-agent "agent-B" — échoué — sid:sid-2]\nKO'),
    ];
    const result = extractSubagentsFromMessages(msgs);

    expect(result).toHaveLength(2);
    expect(result[0].name).toBe("agent-A");
    expect(result[0].sessionId).toBe("sid-1");
    expect(result[0].status).toBe("completed");
    expect(result[1].name).toBe("agent-B");
    expect(result[1].sessionId).toBe("sid-2");
    expect(result[1].status).toBe("failed");
  });

  it("ignore les messages assistant même si le contenu ressemble à un rapport", () => {
    const msgs = [
      makeMsg('[Rapport du sous-agent "x" — terminé — sid:y]', "assistant"),
    ];
    const result = extractSubagentsFromMessages(msgs);
    expect(result).toHaveLength(0);
  });

  it("le champ promptPreview est toujours une chaîne vide", () => {
    const msgs = [
      makeMsg('[Rapport du sous-agent "agent-C" — terminé — sid:sid-3]\nContenu'),
    ];
    const result = extractSubagentsFromMessages(msgs);
    expect(result[0].promptPreview).toBe("");
  });

  it("le type est toujours explorer (valeur par défaut de l'utilitaire)", () => {
    const msgs = [
      makeMsg('[Rapport du sous-agent "agent-D" — terminé — sid:sid-4]\nContenu'),
    ];
    const result = extractSubagentsFromMessages(msgs);
    expect(result[0].type).toBe("explorer");
  });
});

// ─── Note sur globalStore / evictGlobalStore ─────────────────────────────────
//
// `MAX_STORE_ENTRIES` et `evictGlobalStore` sont des symboles module-privés
// dans use-subagents.ts (pas exportés). Le hook dépend en plus de Tauri
// (`listen`, `invoke`) qui n'est pas disponible en environnement jsdom.
// Ces éléments ne peuvent pas être testés unitairement sans :
//   1. Les exporter depuis use-subagents.ts, ou
//   2. Mocker l'intégralité de @tauri-apps/api et du cycle de vie du listener.
// Cette couverture est donc volontairement hors scope de ce fichier de tests.
