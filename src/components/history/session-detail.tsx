import type { SessionDetail } from "@/types/session";
import "./session-detail.css";

interface SessionDetailViewProps {
  detail: SessionDetail;
}

function formatDuration(min: number): string {
  if (min < 1) return "<1min";
  if (min < 60) return `${Math.round(min)}min`;
  const h = Math.floor(min / 60);
  const m = Math.round(min % 60);
  return m > 0 ? `${h}h${m}` : `${h}h`;
}

export function SessionDetailView({ detail }: SessionDetailViewProps) {
  const { meta, messages } = detail;

  return (
    <>
      <div className="sd-header">
        <div className="sd-title">
          {meta.custom_name ?? `Session ${meta.start.split("T")[0]}`}
        </div>
      </div>
      <div className="sd-content">
        <div className="sd-meta-grid">
          <MetaCard value={formatDuration(meta.duration_minutes)} label="Durée" />
          <MetaCard value={String(meta.message_count)} label="Messages" />
          <MetaCard value={`--${meta.mode}`} label="Mode" />
          <MetaCard value={meta.version || "?"} label="Version" />
        </div>
        <div className="sd-conversation">
          {messages.map((msg, i) => (
            <div key={i} className={`sd-msg sd-msg-${msg.role}`}>
              <div className="sd-msg-role">
                {msg.role === "user" ? "Kevin" : "Jackson"}
              </div>
              <div className="sd-msg-text">{msg.content}</div>
            </div>
          ))}
          {messages.length < 1 && (
            <div className="sd-empty">Aucun message dans cette session</div>
          )}
        </div>
      </div>
    </>
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
