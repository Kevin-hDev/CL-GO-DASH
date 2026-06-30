export interface ReleaseNoteSection {
  title: string | null;
  items: string[];
}

const MAX_SECTIONS = 3;
const MAX_ITEMS = 6;
const MAX_TEXT_LENGTH = 180;

export function parseReleaseNotes(notes?: string | null): ReleaseNoteSection[] {
  if (!notes?.trim()) return [];

  const sections: ReleaseNoteSection[] = [];
  let current: ReleaseNoteSection | null = null;
  let itemCount = 0;

  for (const raw of notes.split(/\r?\n/)) {
    const line = raw.trim();
    if (!line || line === "---") continue;

    const heading = readHeading(line);
    if (heading) {
      if (sections.length >= MAX_SECTIONS) {
        current = null;
        continue;
      }
      current = { title: cleanText(heading), items: [] };
      sections.push(current);
      continue;
    }

    const bullet = readBullet(line) ?? line;
    if (!current) {
      if (sections.length >= MAX_SECTIONS) continue;
      current = { title: null, items: [] };
      sections.push(current);
    }
    if (itemCount >= MAX_ITEMS) continue;
    const text = cleanText(bullet);
    if ([...text].length > MAX_TEXT_LENGTH) continue;
    current.items.push(text);
    itemCount += 1;
  }

  return sections.filter((section) => section.items.length > 0);
}

function readHeading(line: string): string | null {
  const match = /^(?:#{2,3})\s+(.+)$/.exec(line);
  return match?.[1]?.trim() || null;
}

function readBullet(line: string): string | null {
  const match = /^[-*]\s+(.+)$/.exec(line);
  return match?.[1]?.trim() || null;
}

function cleanText(text: string): string {
  return text
    .replace(/\*\*/g, "")
    .replace(/`/g, "")
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1");
}
