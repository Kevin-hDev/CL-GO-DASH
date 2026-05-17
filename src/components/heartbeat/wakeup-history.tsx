import { useTranslation } from "react-i18next";
import type { WakeupRun } from "@/types/wakeup";
import { formatDateTime, formatRunStatus } from "@/lib/wakeup-format";

interface WakeupHistoryProps {
  runs: WakeupRun[];
}

export function WakeupHistory({ runs }: WakeupHistoryProps) {
  const { t } = useTranslation();
  const recent = runs.slice(0, 8);

  return (
    <section className="wk-history">
      <div className="wk-history-title">{t("heartbeat.history.title")}</div>
      {recent.length === 0 ? (
        <div className="wk-history-empty">{t("heartbeat.history.empty")}</div>
      ) : (
        <div className="wk-history-list">
          {recent.map((run) => (
            <div className="wk-history-row" key={`${run.wakeup_id}-${run.fired_at}`}>
              <span className={`wk-history-status wk-history-status-${run.status}`}>
                {formatRunStatus(run.status)}
              </span>
              <span className="wk-history-time">{formatDateTime(run.fired_at)}</span>
              {run.error && <span className="wk-history-error">{run.error}</span>}
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
