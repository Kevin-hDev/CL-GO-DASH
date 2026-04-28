const CONTEXT_LINES = 3;

type Edit = { type: "add" | "del" | "ctx"; line: string };

function computeLCS(oldLines: string[], newLines: string[]): number[][] {
  const m = oldLines.length;
  const n = newLines.length;
  const dp: number[][] = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0));
  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      if (oldLines[i - 1] === newLines[j - 1]) {
        dp[i][j] = dp[i - 1][j - 1] + 1;
      } else {
        dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1]);
      }
    }
  }
  return dp;
}

function buildEdits(oldLines: string[], newLines: string[]): Edit[] {
  const dp = computeLCS(oldLines, newLines);
  const edits: Edit[] = [];
  let i = oldLines.length;
  let j = newLines.length;
  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && oldLines[i - 1] === newLines[j - 1]) {
      edits.push({ type: "ctx", line: oldLines[i - 1] });
      i--;
      j--;
    } else if (j > 0 && (i === 0 || dp[i][j - 1] >= dp[i - 1][j])) {
      edits.push({ type: "add", line: newLines[j - 1] });
      j--;
    } else {
      edits.push({ type: "del", line: oldLines[i - 1] });
      i--;
    }
  }
  return edits.reverse();
}

interface HunkRange {
  oldStart: number;
  oldCount: number;
  newStart: number;
  newCount: number;
  lines: string[];
}

export function generateHunks(oldContent: string, newContent: string): string[] {
  const oldLines = splitLines(oldContent);
  const newLines = splitLines(newContent);

  if (oldLines.join("\n") === newLines.join("\n")) return [];

  const edits = buildEdits(oldLines, newLines);

  const changeIndices: number[] = [];
  edits.forEach((e, idx) => {
    if (e.type !== "ctx") changeIndices.push(idx);
  });

  if (changeIndices.length === 0) return [];

  const hunks: HunkRange[] = [];
  let hunkStart = -1;
  let hunkEnd = -1;

  for (const idx of changeIndices) {
    const start = Math.max(0, idx - CONTEXT_LINES);
    const end = Math.min(edits.length - 1, idx + CONTEXT_LINES);
    if (hunkStart === -1) {
      hunkStart = start;
      hunkEnd = end;
    } else if (start <= hunkEnd + 1) {
      hunkEnd = Math.max(hunkEnd, end);
    } else {
      hunks.push(buildHunk(edits, hunkStart, hunkEnd));
      hunkStart = start;
      hunkEnd = end;
    }
  }
  if (hunkStart !== -1) {
    hunks.push(buildHunk(edits, hunkStart, hunkEnd));
  }

  return hunks.map(formatHunk);
}

function buildHunk(edits: Edit[], from: number, to: number): HunkRange {
  let oldStart = 1;
  let newStart = 1;
  for (let k = 0; k < from; k++) {
    const e = edits[k];
    if (e.type === "ctx" || e.type === "del") oldStart++;
    if (e.type === "ctx" || e.type === "add") newStart++;
  }

  let oldCount = 0;
  let newCount = 0;
  const lines: string[] = [];

  for (let k = from; k <= to; k++) {
    const e = edits[k];
    if (e.type === "ctx") {
      lines.push(" " + e.line);
      oldCount++;
      newCount++;
    } else if (e.type === "del") {
      lines.push("-" + e.line);
      oldCount++;
    } else {
      lines.push("+" + e.line);
      newCount++;
    }
  }

  return { oldStart, oldCount, newStart, newCount, lines };
}

function formatHunk(h: HunkRange): string {
  const header = `@@ -${h.oldStart},${h.oldCount} +${h.newStart},${h.newCount} @@`;
  return [header, ...h.lines].join("\n");
}

function splitLines(content: string): string[] {
  if (!content) return [];
  return content.split(/\r?\n/);
}

export function buildOldContent(currentContent: string, oldText: string | undefined, newText: string | undefined): string {
  if (!oldText && !newText) return currentContent;
  const nt = newText ?? "";
  const ot = oldText ?? "";
  const idx = currentContent.indexOf(nt);
  if (idx === -1) return currentContent;
  return currentContent.slice(0, idx) + ot + currentContent.slice(idx + nt.length);
}
