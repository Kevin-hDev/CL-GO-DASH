import { memo, useMemo } from "react";
import { useTranslation } from "react-i18next";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkBreaks from "remark-breaks";
import rehypeRaw from "rehype-raw";
import rehypeSanitize from "rehype-sanitize";
import type { Components } from "react-markdown";
import { open } from "@tauri-apps/plugin-shell";
import { CodeBlock } from "./code-block";
import { ThinkingSection } from "./thinking-section";
import { MessageActions } from "./message-actions";
import { SavedToolBubble } from "./tool-bubble";
import { StreamingStats, formatTotalElapsed } from "./streaming-stats";
import { LinkPreviewCard } from "./link-preview-card";
import { useHoverClass } from "@/hooks/use-hover-class";
import type { ToolActivityRecord } from "@/types/agent";
import "./messages.css";
import "./chat-markdown.css";

const MAX_PREVIEWS = 5;
const URL_RE = /https?:\/\/[^\s<>"')\]]+/g;

function extractUrls(text: string): string[] {
  const urls: string[] = [];
  const seen = new Set<string>();
  let m;
  while ((m = URL_RE.exec(text)) !== null) {
    if (!seen.has(m[0]) && urls.length < MAX_PREVIEWS) {
      seen.add(m[0]);
      urls.push(m[0]);
    }
  }
  URL_RE.lastIndex = 0;
  return urls;
}

function isPreviewEnabled(): boolean {
  return localStorage.getItem("clgo-link-preview") !== "false";
}

function closeUnclosedCodeBlocks(text: string): string {
  const count = (text.match(/```/g) || []).length;
  if (count % 2 !== 0) return text + "\n```";
  return text;
}

const mdComponents: Components = {
  table({ children }) {
    return (
      <div className="chat-md-table-scroll">
        <table>{children}</table>
      </div>
    );
  },
  th({ children }) {
    return <th>{formatTableCell(children)}</th>;
  },
  td({ children }) {
    return <td>{formatTableCell(children)}</td>;
  },
  pre({ children }) {
    const child = children as React.ReactElement<{ className?: string; children?: React.ReactNode }>;
    const className = child?.props?.className || "";
    const lang = /language-(\w+)/.exec(className)?.[1] || "";
    const raw = child?.props?.children;
    const code = (typeof raw === "string" ? raw : "").replace(/\n$/, "");
    return <CodeBlock language={lang} code={code} />;
  },
  a({ href, children }) {
    return (
      <a
        className="chat-link"
        href={href ?? "#"}
        title={href ?? ""}
        onClick={(e) => { e.preventDefault(); if (href) void open(href); }}
      >
        {children}
      </a>
    );
  },
};

const remarkPlugins = [remarkGfm, remarkBreaks];
const rehypePlugins = [rehypeRaw, rehypeSanitize];

function formatTableCell(children: React.ReactNode): React.ReactNode {
  if (typeof children !== "string") return children;
  const parts = splitCompactList(children);
  if (parts.length <= 1) return children;
  return parts.map((part, index) => (
    index === 0 ? part : <span key={index} className="chat-md-cell-list-break">{part}</span>
  ));
}

function splitCompactList(text: string): string[] {
  const marker = /(\s+)(?=(?:[2-9]|[1-9]\d)[.)]\s|[2-9]️⃣\s)/g;
  return text.split(marker).filter((part) => part.trim().length > 0);
}

function ChatMarkdown({ content }: { content: string }) {
  const prepared = useMemo(() => closeUnclosedCodeBlocks(content), [content]);
  const urls = useMemo(() => (isPreviewEnabled() ? extractUrls(content) : []), [content]);

  return (
    <>
      <ReactMarkdown remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins} components={mdComponents}>
        {prepared}
      </ReactMarkdown>
      {urls.length > 0 && (
        <div className="chat-previews-block">
          {urls.map((url) => <LinkPreviewCard key={url} url={url} />)}
        </div>
      )}
    </>
  );
}

interface AssistantMessageProps {
  content: string;
  thinking?: string;
  toolActivities?: ToolActivityRecord[];
  isStreaming?: boolean;
  onReload?: () => void;
  tokens?: number;
  tps?: number;
  totalElapsedMs?: number;
  streamStartedAt?: number | null;
  liveTokenCount?: number;
}

function formatTokens(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
  return String(n);
}

export const AssistantMessage = memo(function AssistantMessage({
  content, thinking, toolActivities, isStreaming, onReload,
  tokens, tps, totalElapsedMs, streamStartedAt, liveTokenCount,
}: AssistantMessageProps) {
  const { t } = useTranslation();
  const hoverRef = useHoverClass();
  const hasTokens = tokens != null && tokens > 0;
  const hasTps = tps != null && tps > 0.1;
  const totalTime = formatTotalElapsed(totalElapsedMs ?? 0);

  return (
    <div className="msg-assistant" ref={hoverRef}>
      {thinking && <ThinkingSection content={thinking} isActive={isStreaming && !content} />}
      {toolActivities && toolActivities.length > 0 && (
        <SavedToolBubble tools={toolActivities} />
      )}
      <div className="msg-assistant-content chat-md">
        {content && <ChatMarkdown content={content} />}
        {isStreaming && (
          <>
            <span style={{ animation: "pulse-dot 1s infinite" }}>&#9610;</span>
            {!content && (
              <StreamingStats segmentStartedAt={streamStartedAt ?? null} liveTokenCount={liveTokenCount ?? 0} />
            )}
          </>
        )}
      </div>
      {!isStreaming && content.trim() && (
        <MessageActions messageRole="assistant" content={content} onReload={onReload}>
          {(hasTokens || hasTps || totalTime) && (
            <span className="msg-stats-inline">
              {totalTime && <><span>{totalTime}</span><span>&middot;</span></>}
              {hasTokens && <span>{formatTokens(tokens)} {t("agentLocal.tokens")}</span>}
              {hasTokens && hasTps && <span>&middot;</span>}
              {hasTps && <span>{tps.toFixed(1)} {t("agentLocal.tps")}</span>}
            </span>
          )}
        </MessageActions>
      )}
    </div>
  );
});
