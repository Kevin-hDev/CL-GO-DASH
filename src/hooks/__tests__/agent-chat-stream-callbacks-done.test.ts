import { describe, it, expect, vi } from "vitest";
import { applyStreamEvent, finishPartialStream, createManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";
import type { ManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";
import type { StreamEvent } from "@/types/agent";

vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

function makeState(overrides: Partial<ManagedStreamState> = {}): ManagedStreamState {
  return { ...createManagedStreamState([], 0), streamStartedAt: null, segmentStartedAt: null, ...overrides };
}

function doneEvent(overrides: Partial<StreamEvent & { event: "done" } extends { data: infer D } ? D : never> = {}): StreamEvent {
  return { event: "done", data: { evalCount: 0, evalDurationNs: 0, finalTps: 5, promptTokens: 0, contextTokens: 0, ...overrides } };
}

describe("done", () => {
  it("finalise le stream (isStreaming=false, completed=true)", () => {
    const { state: s } = applyStreamEvent(makeState(), doneEvent({ finalTps: 12 }));
    expect(s.isStreaming).toBe(false);
    expect(s.completed).toBe(true);
  });

  it("crée un assistantMessage si du contenu existe", () => {
    const result = applyStreamEvent(makeState({ currentContent: "réponse finale" }), doneEvent({ finalTps: 10 }));
    expect(result.assistantMessage).toBeDefined();
    expect(result.assistantMessage?.role).toBe("assistant");
    expect(result.assistantMessage?.content).toBe("réponse finale");
  });

  it("ne crée pas de message si aucun contenu", () => {
    const result = applyStreamEvent(makeState(), doneEvent());
    expect(result.assistantMessage).toBeUndefined();
  });

  it("respecte MAX_MESSAGES_PER_SESSION (2000)", () => {
    const messages = Array.from({ length: 2000 }, (_, i) => ({
      id: `msg-${i}`, role: "user" as const, content: `msg ${i}`,
      files: [], timestamp: new Date().toISOString(), tokens: 0,
    }));
    const result = applyStreamEvent(makeState({ messages, currentContent: "nouveau" }), doneEvent());
    expect(result.state.messages.length).toBeLessThanOrEqual(2000);
  });

  it("utilise contextTokens pour tokenCount si fourni", () => {
    const result = applyStreamEvent(makeState({ tokenCount: 100 }), doneEvent({ contextTokens: 999 }));
    expect(result.state.tokenCount).toBe(999);
  });
});

describe("error", () => {
  it("met à jour state.error avec clé i18n générique", () => {
    const result = applyStreamEvent(makeState(), { event: "error", data: { message: "message_inconnu" } });
    expect(result.state.error).toBe("errors.streamInterrupted");
  });

  it("utilise la clé i18n connue pour ollama_connection_lost", () => {
    const result = applyStreamEvent(makeState(), { event: "error", data: { message: "ollama_connection_lost" } });
    expect(result.state.error).toBe("errors.ollamaConnectionLost");
  });

  it("utilise la clé i18n connue pour auth_failed", () => {
    const result = applyStreamEvent(makeState(), { event: "error", data: { message: "auth_failed" } });
    expect(result.state.error).toBe("errors.authFailed");
  });

  it("marque isConnectionError=true quand le flag est présent", () => {
    const result = applyStreamEvent(makeState(), {
      event: "error", data: { message: "err", isConnection: true } as unknown as { message: string },
    });
    expect(result.state.isConnectionError).toBe(true);
  });

  it("finalise le stream (isStreaming=false) même sur erreur", () => {
    const result = applyStreamEvent(makeState(), { event: "error", data: { message: "crash" } });
    expect(result.state.isStreaming).toBe(false);
  });
});

describe("events neutres", () => {
  it("sessionSnapshot ne modifie pas l'état observable", () => {
    const state = makeState({ currentContent: "contenu existant" });
    const { state: s } = applyStreamEvent(state, {
      event: "sessionSnapshot", data: { messages: [], tokenCount: 0 },
    });
    expect(s.currentContent).toBe("contenu existant");
  });

  it("subagentSpawned ne modifie pas l'état observable", () => {
    const state = makeState({ currentContent: "contenu existant" });
    const { state: s } = applyStreamEvent(state, {
      event: "subagentSpawned",
      data: { subagentSessionId: "s1", subagentName: "test", subagentType: "explorer", promptPreview: "..." },
    });
    expect(s.currentContent).toBe("contenu existant");
  });
});

describe("finishPartialStream", () => {
  it("finalise un stream incomplet (isStreaming=false, completed=true)", () => {
    const result = finishPartialStream(makeState({ currentContent: "partiel" }));
    expect(result.state.isStreaming).toBe(false);
    expect(result.state.completed).toBe(true);
  });

  it("crée un assistantMessage depuis le contenu partiel", () => {
    const result = finishPartialStream(makeState({ currentContent: "contenu partiel" }));
    expect(result.assistantMessage).toBeDefined();
    expect(result.assistantMessage?.content).toBe("contenu partiel");
  });

  it("ne crée pas de message si l'état est vide", () => {
    const result = finishPartialStream(makeState());
    expect(result.assistantMessage).toBeUndefined();
  });

  it("combine les segments complétés et le contenu courant", () => {
    const state = makeState({
      completedSegments: [{ thinking: "", tools: [], content: "segment 1" }],
      currentContent: "segment 2",
    });
    const result = finishPartialStream(state);
    expect(result.assistantMessage?.content).toContain("segment 1");
    expect(result.assistantMessage?.content).toContain("segment 2");
  });
});

describe("done — limite messages assertion précise", () => {
  it("le 2001ème message évince msg-0 et place le nouveau en dernière position", () => {
    const messages = Array.from({ length: 2000 }, (_, i) => ({
      id: `msg-${i}`, role: "user" as const, content: `msg ${i}`,
      files: [], timestamp: new Date().toISOString(), tokens: 0,
    }));
    const result = applyStreamEvent(
      makeState({ messages, currentContent: "message 2001" }),
      doneEvent({ finalTps: 5 }),
    );
    expect(result.state.messages).toHaveLength(2000);
    // msg-0 doit être évincé
    expect(result.state.messages.find((m) => m.id === "msg-0")).toBeUndefined();
    // le nouveau message est le dernier
    expect(result.state.messages[1999].content).toBe("message 2001");
  });

  it("sans contextTokens, tokenCount = tokenCount initial + evalCount", () => {
    const result = applyStreamEvent(
      makeState({ tokenCount: 100, currentContent: "réponse" }),
      doneEvent({ evalCount: 50, contextTokens: 0 }),
    );
    expect(result.state.tokenCount).toBe(150);
  });
});

describe("error — clés connues supplémentaires", () => {
  it("rate_limit utilise la clé i18n errors.rateLimited", () => {
    const result = applyStreamEvent(makeState(), { event: "error", data: { message: "rate_limit" } });
    expect(result.state.error).toBe("errors.rateLimited");
  });

  it("model_not_found utilise la clé i18n errors.modelNotFound", () => {
    const result = applyStreamEvent(makeState(), { event: "error", data: { message: "model_not_found" } });
    expect(result.state.error).toBe("errors.modelNotFound");
  });

  it("error préserve le contenu accumulé (currentContent est vidé dans le state final mais assistantMessage est absent si pas de segments)", () => {
    // une erreur ne doit pas créer un assistantMessage parasite si currentContent était vide
    const result = applyStreamEvent(makeState({ currentContent: "" }), { event: "error", data: { message: "crash" } });
    expect(result.assistantMessage).toBeUndefined();
  });

  it("error avec du contenu courant — currentContent est finalisé (pas de perte silencieuse)", () => {
    // le contenu accumulé avant l'erreur doit être dans l'assistantMessage s'il existe
    const result = applyStreamEvent(
      makeState({ currentContent: "réponse partielle" }),
      { event: "error", data: { message: "crash" } },
    );
    // le message est créé depuis le contenu accumulé
    expect(result.assistantMessage).toBeDefined();
    expect(result.assistantMessage?.content).toBe("réponse partielle");
  });
});
