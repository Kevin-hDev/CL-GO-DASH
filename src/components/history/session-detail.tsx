import { useEffect, useRef, useCallback, useMemo, useState } from "react";
import { useLottie } from "lottie-react";
import { useToast } from "@/components/ui/toast";
import { DownloadSimple } from "@/components/ui/icons";
import { useLiveSession } from "@/hooks/use-live-session";
import type { SessionDetail } from "@/types/session";
import thinkingAnimation from "@/assets/thinking-loader.json";
import scrollDownIcon from "@/assets/fleche.png";
import "./session-detail.css";

interface SessionDetailViewProps {
  detail: SessionDetail;
  isLive?: boolean;
}

function formatDuration(min: number): string {
  if (min < 1) return "<1min";
  if (min < 60) return `${Math.round(min)}min`;
  const h = Math.floor(min / 60);
  const m = Math.round(min % 60);
  return m > 0 ? `${h}h${m}` : `${h}h`;
}

function toMarkdown(detail: SessionDetail): string {
  const { meta, messages, tools_used, files_modified } = detail;
  const name = meta.custom_name ?? `Session ${meta.id.slice(0, 8)}`;
  const lines: string[] = [
    `# ${name}`, "",
    `| Info | Value |`, `|------|-------|`,
    `| Date | ${meta.start} |`,
    `| Duration | ${formatDuration(meta.duration_minutes)} |`,
    `| Mode | --${meta.mode} |`,
    `| Messages | ${meta.message_count} |`,
    `| Version | ${meta.version || "?"} |`,
  ];
  if (tools_used.length > 0) {
    lines.push("", `**Tools** : ${tools_used.join(", ")}`);
  }
  if (files_modified.length > 0) {
    lines.push("", `**Files modified** : ${files_modified.join(", ")}`);
  }
  lines.push("", "---", "");
  for (const msg of messages) {
    const author = msg.role === "user" ? "Prompt" : "Jackson";
    lines.push(`## ${author}`, "", msg.content, "");
  }
  return lines.join("\n");
}

export function SessionDetailView({ detail, isLive }: SessionDetailViewProps) {
  const { meta, messages: loadedMessages } = detail;
  const liveMessages = useLiveSession(isLive === true);

  const messages = useMemo(() => {
    if (!isLive || liveMessages.length === 0) return loadedMessages;
    return [...loadedMessages, ...liveMessages];
  }, [loadedMessages, liveMessages, isLive]);

  const bottomRef = useRef<HTMLDivElement>(null);
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const prevCountRef = useRef(0);
  const [isAtBottom, setIsAtBottom] = useState(true);

  // Only auto-scroll if user is already at the bottom
  useEffect(() => {
    if (messages.length > prevCountRef.current && isAtBottom) {
      bottomRef.current?.scrollIntoView({ behavior: "smooth" });
    }
    prevCountRef.current = messages.length;
  }, [messages.length, isAtBottom]);

  const handleScroll = useCallback(() => {
    const el = scrollContainerRef.current;
    if (!el) return;
    const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 80;
    setIsAtBottom(nearBottom);
  }, []);

  const scrollToBottom = useCallback(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, []);

  const toast = useToast();

  const handleDownload = useCallback(() => {
    const md = toMarkdown(detail);
    const blob = new Blob([md], { type: "text/markdown;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    const date = meta.start.split("T")[0] || "session";
    a.href = url;
    a.download = `clgo-${meta.mode}-${date}-${meta.id.slice(0, 8)}.md`;
    a.click();
    URL.revokeObjectURL(url);
    toast.show("", "check");
  }, [detail, meta, toast]);

  return (
    <>
      <div className="sd-header">
        <div className="sd-title">
          {meta.custom_name ?? `Session ${meta.start.split("T")[0]}`}
        </div>
        {isLive ? (
          <div className="sd-live-badge">
            <span className="sd-live-dot" /> LIVE
          </div>
        ) : (
          <button
            className="sd-download-btn"
            onClick={handleDownload}
            title="Download as Markdown"
          >
            <DownloadSimple size={16} />
          </button>
        )}
      </div>
      <div style={{ position: "relative", flex: 1, minHeight: 0 }}>
        <div className="sd-content" ref={scrollContainerRef} onScroll={handleScroll} style={{ height: "100%" }}>
          <div className="sd-meta-grid">
            <MetaCard value={formatDuration(meta.duration_minutes)} label="Duration" />
            <MetaCard value={String(meta.message_count)} label="Messages" />
            <MetaCard value={`--${meta.mode}`} label="Mode" />
            <MetaCard value={meta.version || "?"} label="Version" />
          </div>
          <div className="sd-conversation">
            {messages.map((msg, i) => (
              <div key={i} className={`sd-msg sd-msg-${msg.role}`}>
                <div className="sd-msg-role">
                  {msg.role === "user" ? "Prompt" : "Jackson"}
                </div>
                <div className="sd-msg-text">{msg.content}</div>
              </div>
            ))}
            {isLive && <ThinkingIndicator />}
            {messages.length < 1 && !isLive && (
              <div className="sd-empty">No messages in this session</div>
            )}
            <div ref={bottomRef} />
          </div>
        </div>
        {!isAtBottom && (
          <button
            onClick={scrollToBottom}
            style={{
              position: "absolute",
              bottom: 20,
              right: 20,
              width: 50,
              height: 50,
              borderRadius: "var(--radius-md)",
              border: "1px solid var(--edge)",
              background: "var(--surface)",
              boxShadow: "var(--shadow-card)",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              opacity: 0.85,
              zIndex: 10,
            }}
          >
            <img src={scrollDownIcon} alt="Scroll to bottom" style={{ width: 25, height: 25 }} />
          </button>
        )}
      </div>
    </>
  );
}

function ThinkingIndicator() {
  const { View } = useLottie({
    animationData: thinkingAnimation,
    loop: true,
    className: "sd-thinking-lottie",
  });

  return (
    <div className="sd-msg sd-msg-assistant sd-thinking">
      <div className="sd-msg-role">Jackson</div>
      {View}
    </div>
  );
}

function MetaCard({ value, label }: { value: string; label: string }) {
  return (
    <div className="sd-meta-card">
      <div className="sd-meta-value">{value}</div>
      <div className="sd-meta-label">{label}</div>
    </div>
  );
}
