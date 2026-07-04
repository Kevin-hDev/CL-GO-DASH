import type React from "react";
import type { SkillInfo } from "@/types/agent";
import type { SkillTokenSource } from "@/components/agent-local/skill-chip-ranges";

export function replaceSlashToken(text: string, skillName: string): string {
  const lastSlash = text.lastIndexOf("/");
  if (lastSlash < 0) return `${text}/${skillName}`;
  const before = text.slice(0, lastSlash);
  const after = text.slice(lastSlash);
  const trailing = after.match(/^\/[^\s/]*/)?.[0] ?? "";
  return `${before}/${skillName}${after.slice(trailing.length)}`;
}

export function activeSkillsInText(
  text: string,
  skills: SkillInfo[],
): SkillInfo[] {
  const activeNames = new Set(
    splitSkillText(text, skills.map((skill) => skill.name))
      .filter((part) => part.kind === "skill")
      .map((part) => part.text.slice(1)),
  );
  return skills.filter((skill) => activeNames.has(skill.name));
}

export interface ChatChipOptions {
  /** Optional built-in command names rendered with a distinct icon. */
  builtInNames?: string[];
}

/**
 * Render skill (and optionally built-in) chips inside the chat messages.
 *
 * Pure React render: no caret to align, so each "/name" token becomes an
 * inline chip directly.
 */
export function highlightSkillNodes(
  nodes: React.ReactNode[],
  skillNames: string[] | undefined,
  options?: ChatChipOptions,
): React.ReactNode[] {
  const all = collectChipNames(skillNames, options);
  if (all.length < 1) return nodes;

  const builtInSet = new Set(options?.builtInNames ?? []);
  const highlighted: React.ReactNode[] = [];
  nodes.forEach((node, nodeIndex) => {
    if (typeof node !== "string") {
      highlighted.push(node);
      return;
    }
    splitSkillText(node, all).forEach((part, partIndex) => {
      if (part.kind !== "skill") {
        highlighted.push(part.text);
        return;
      }
      const name = part.text.startsWith("/") ? part.text.slice(1) : part.text;
      const source: SkillTokenSource = builtInSet.has(name) ? "built-in" : "skill";
      highlighted.push(renderChatChip(name, source, `${nodeIndex}-${partIndex}`));
    });
  });
  return highlighted;
}

function collectChipNames(
  skillNames: string[] | undefined,
  options?: ChatChipOptions,
): string[] {
  const names = [...(skillNames ?? []), ...(options?.builtInNames ?? [])];
  return names.filter(Boolean);
}

function renderChatChip(name: string, source: SkillTokenSource, key: React.Key) {
  const Icon = source === "built-in" ? ClockIcon : MagicWandIcon;
  return (
    <span key={key} className={`skill-chip${source === "built-in" ? " skill-chip-built-in" : ""}`}>
      <Icon className="skill-chip-icon" />
      <span className="skill-chip-name">{name}</span>
    </span>
  );
}

function MagicWandIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 256 256" fill="currentColor" aria-hidden="true">
      <path d="M48,64a8,8,0,0,1,8-8H72V40a8,8,0,0,1,16,0V56h16a8,8,0,0,1,0,16H88V88a8,8,0,0,1-16,0V72H56A8,8,0,0,1,48,64ZM184,192h-8v-8a8,8,0,0,0-16,0v8h-8a8,8,0,0,0,0,16h8v8a8,8,0,0,0,16,0v-8h8a8,8,0,0,0,0-16Zm56-48H224V128a8,8,0,0,0-16,0v16H192a8,8,0,0,0,0,16h16v16a8,8,0,0,0,16,0V160h16a8,8,0,0,0,0-16ZM219.31,80,80,219.31a16,16,0,0,1-22.62,0L36.68,198.63a16,16,0,0,1,0-22.63L176,36.69a16,16,0,0,1,22.63,0l20.68,20.68A16,16,0,0,1,219.31,80Zm-54.63,32L144,91.31l-96,96L68.68,208ZM208,68.69,187.31,48l-32,32L176,100.69Z" />
    </svg>
  );
}

function ClockIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 256 256" fill="currentColor" aria-hidden="true">
      <path d="M128,24A104,104,0,1,0,232,128,104.11,104.11,0,0,0,128,24Zm0,192a88,88,0,1,1,88-88A88.1,88.1,0,0,1,128,216Zm64-88a8,8,0,0,1-8,8H128a8,8,0,0,1-8-8V72a8,8,0,0,1,16,0v48h48A8,8,0,0,1,192,128Z" />
    </svg>
  );
}

export function splitSkillText(text: string, skillNames: string[]) {
  const names = skillNames.filter(Boolean).sort((a, b) => b.length - a.length);
  if (names.length < 1) return [{ kind: "text" as const, text }];

  const parts: Array<{ kind: "text" | "skill"; text: string }> = [];
  let lastIndex = 0;

  for (let index = 0; index < text.length; index += 1) {
    if (text[index] !== "/" || !isTokenStart(text, index)) continue;
    const name = matchingSkillName(text, index + 1, names);
    if (!name) continue;

    const end = index + 1 + name.length;
    if (!isTokenEnd(text, end)) continue;

    if (index > lastIndex) {
      parts.push({ kind: "text", text: text.slice(lastIndex, index) });
    }
    parts.push({ kind: "skill", text: text.slice(index, end) });
    lastIndex = end;
    index = end - 1;
  }

  if (lastIndex < text.length) {
    parts.push({ kind: "text", text: text.slice(lastIndex) });
  }
  return parts.length > 0 ? parts : [{ kind: "text", text }];
}

function matchingSkillName(text: string, start: number, skillNames: string[]): string | null {
  for (const name of skillNames) {
    if (text.startsWith(name, start)) return name;
  }
  return null;
}

function isTokenStart(text: string, index: number): boolean {
  return index === 0 || isWhitespace(text[index - 1]);
}

function isTokenEnd(text: string, index: number): boolean {
  if (index >= text.length) return true;
  const char = text[index];
  return isWhitespace(char) || char === "." || char === "," || char === ";"
    || char === ":" || char === "!" || char === "?";
}

function isWhitespace(char: string | undefined): boolean {
  return char === " " || char === "\n" || char === "\r" || char === "\t";
}
