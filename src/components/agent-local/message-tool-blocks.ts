export interface ToolTimelineSegment<T> {
  thinking?: string;
  content?: string;
  tools: T[];
  isCurrent?: boolean;
}

export interface ToolTimelineBlock<T> {
  thinking?: string;
  content?: string;
  tools: T[];
  isCurrent: boolean;
}

export function buildToolTimelineBlocks<T>(
  segments: Array<ToolTimelineSegment<T>>,
): Array<ToolTimelineBlock<T>> {
  const blocks: Array<ToolTimelineBlock<T>> = [];
  for (const segment of segments) {
    if (hasNarrative(segment)) {
      blocks.push({
        thinking: segment.thinking,
        content: segment.content,
        tools: [...segment.tools],
        isCurrent: segment.isCurrent === true,
      });
      continue;
    }
    if (segment.tools.length === 0) continue;
    const last = blocks[blocks.length - 1];
    if (last) {
      last.tools.push(...segment.tools);
    } else {
      blocks.push({ tools: [...segment.tools], isCurrent: false });
    }
  }
  return blocks;
}

export function hasNarrative(segment: { thinking?: string; content?: string }): boolean {
  return !!segment.thinking || !!segment.content;
}
