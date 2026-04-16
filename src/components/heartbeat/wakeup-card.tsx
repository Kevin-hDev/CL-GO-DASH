import type { ScheduledWakeup } from "@/types/wakeup";
import { ActiveBadge, ScheduleBadge } from "./badges";

interface WakeupCardProps {
  wakeup: ScheduledWakeup;
  onClick: () => void;
}

export function WakeupCard({ wakeup, onClick }: WakeupCardProps) {
  return (
    <button className="wk-card" onClick={onClick} type="button">
      <div className="wk-card-row wk-card-row-top">
        <span className="wk-card-model">{wakeup.model}</span>
        <ActiveBadge active={wakeup.active} />
      </div>
      <div className="wk-card-row">
        <ScheduleBadge schedule={wakeup.schedule} />
      </div>
      <div className="wk-card-row wk-card-desc">
        {wakeup.description || wakeup.name}
      </div>
    </button>
  );
}
