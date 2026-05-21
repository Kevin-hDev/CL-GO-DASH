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
  return skills.filter((skill) => skillTokenRegex(skill.name).test(text));
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
  const pattern = skillNames
    .filter(Boolean)
    .sort((a, b) => b.length - a.length)
    .map(escapeRegex)
    .join("|");
  if (!pattern) return [{ kind: "text" as const, text }];

  const regex = new RegExp(`(^|\\s)(/(${pattern}))(?=$|\\s|[.,;:!?])`, "g");
  const parts: Array<{ kind: "text" | "skill"; text: string }> = [];
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = regex.exec(text)) !== null) {
    const prefix = match[1] ?? "";
    const skill = match[2] ?? "";
    const skillStart = match.index + prefix.length;
    if (skillStart > lastIndex) {
      parts.push({ kind: "text", text: text.slice(lastIndex, skillStart) });
    }
    parts.push({ kind: "skill", text: skill });
    lastIndex = skillStart + skill.length;
  }

  if (lastIndex < text.length) {
    parts.push({ kind: "text", text: text.slice(lastIndex) });
  }
  return parts.length > 0 ? parts : [{ kind: "text", text }];
}

function skillTokenRegex(skillName: string): RegExp {
  return new RegExp(`(^|\\s)/${escapeRegex(skillName)}(?=$|\\s|[.,;:!?])`);
}

function escapeRegex(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
