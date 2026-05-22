import type React from "react";
import type { SkillInfo } from "@/types/agent";

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

export function highlightSkillText(
  text: string,
  skillNames: string[] | undefined,
  className = "skill-inline-highlight",
): React.ReactNode[] {
  if (!skillNames || skillNames.length < 1) return [text];
  return splitSkillText(text, skillNames).map((part, index) => (
    part.kind === "skill"
      ? <span key={index} className={className}>{part.text}</span>
      : part.text
  ));
}

export function highlightSkillNodes(
  nodes: React.ReactNode[],
  skillNames: string[] | undefined,
): React.ReactNode[] {
  if (!skillNames || skillNames.length < 1) return nodes;
  const highlighted: React.ReactNode[] = [];
  nodes.forEach((node, nodeIndex) => {
    if (typeof node !== "string") {
      highlighted.push(node);
      return;
    }
    splitSkillText(node, skillNames).forEach((part, partIndex) => {
      highlighted.push(
        part.kind === "skill"
          ? <span key={`${nodeIndex}-${partIndex}`} className="msg-skill-inline">{part.text}</span>
          : part.text,
      );
    });
  });
  return highlighted;
}

function splitSkillText(text: string, skillNames: string[]) {
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
