import { useState, useRef } from "react";
import { useClickOutside } from "@/hooks/use-click-outside";
import { useKeyboard } from "@/hooks/use-keyboard";
import { Check } from "@/components/ui/icons";
import { useOllamaModels } from "@/hooks/use-ollama-models";
import { idMatch } from "@/lib/utils";
import type { OllamaModel } from "@/types/agent";

interface ModelSelectorProps {
  selectedModel: string;
  onSelect: (model: string) => void;
  thinkingEnabled: boolean;
  onToggleThinking: () => void;
}

export function ModelSelector({
  selectedModel, onSelect, thinkingEnabled, onToggleThinking,
}: ModelSelectorProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const { groupedByFamily } = useOllamaModels();

  useClickOutside(ref, () => setOpen(false));
  useKeyboard({ onEscape: () => setOpen(false) });

  const hasThinking = Object.values(groupedByFamily)
    .flat()
    .find((m) => idMatch(m.name, selectedModel))
    ?.capabilities.includes("thinking");

  return (
    <div ref={ref} style={{ position: "relative" }}>
      <button
        onClick={() => setOpen(!open)}
        style={{
          fontSize: "var(--text-xs)", color: "var(--ink-faint)",
          background: "none", border: "none", cursor: "pointer",
        }}
      >
        {selectedModel}
        {thinkingEnabled && hasThinking && (
          <span style={{ marginLeft: 4, color: "var(--pulse)" }}>Étendue</span>
        )}
      </button>
      {open && (
        <div style={{
          position: "absolute", bottom: "100%", right: 0, marginBottom: 4,
          width: 260, maxHeight: 320, overflowY: "auto",
          borderRadius: "var(--radius-md)", border: "1px solid var(--edge)",
          background: "var(--shell)", boxShadow: "var(--shadow-card)", zIndex: 50,
        }}>
          {hasThinking && (
            <div
              onClick={onToggleThinking}
              style={{
                display: "flex", justifyContent: "space-between", alignItems: "center",
                padding: "var(--space-sm) var(--space-md)",
                fontSize: "var(--text-xs)", cursor: "pointer",
                borderBottom: "1px solid var(--edge)",
              }}
            >
              <span>Réflexion étendue</span>
              <span style={{ color: thinkingEnabled ? "var(--pulse)" : "var(--ink-faint)" }}>
                {thinkingEnabled ? "ON" : "OFF"}
              </span>
            </div>
          )}
          {Object.entries(groupedByFamily).map(([family, models]) => (
            <div key={family}>
              <div style={{
                padding: "var(--space-xs) var(--space-md)",
                fontSize: "var(--text-xs)", fontWeight: 600,
                color: "var(--ink-faint)", textTransform: "uppercase",
                letterSpacing: "0.5px",
              }}>
                {family}
              </div>
              {models.map((m: OllamaModel) => {
                const sel = idMatch(m.name, selectedModel);
                return (
                  <div
                    key={m.name}
                    onClick={() => { onSelect(m.name); setOpen(false); }}
                    style={{
                      display: "flex", justifyContent: "space-between", alignItems: "center",
                      padding: "6px var(--space-md)",
                      fontSize: "var(--text-xs)", cursor: "pointer",
                      color: sel ? "var(--pulse)" : "var(--ink)",
                      transition: "background var(--ease-smooth)",
                    }}
                    onMouseEnter={(e) => { e.currentTarget.style.background = "var(--pulse-muted)"; }}
                    onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}
                  >
                    <span>{m.name}</span>
                    <span style={{ display: "flex", alignItems: "center", gap: 4 }}>
                      <span style={{ color: "var(--ink-faint)" }}>{m.parameter_size}</span>
                      {sel && <Check size={12} />}
                    </span>
                  </div>
                );
              })}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
