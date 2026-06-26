import { ClipboardText } from "@/components/ui/icons";
import { ChatMarkdown } from "./assistant-message";
import type { AgentPlanPreview } from "@/types/agent";
import "./plan-preview-bubble.css";

interface PlanPreviewBubbleProps {
  plan: AgentPlanPreview;
}

export function PlanPreviewBubble({ plan }: PlanPreviewBubbleProps) {
  return (
    <div className="chat-bubble ppb-root">
      <div className="ppb-header">
        <ClipboardText size={16} weight="regular" />
        <span className="ppb-title">{plan.title}</span>
      </div>
      <div className="ppb-content chat-md">
        <ChatMarkdown content={plan.content} />
      </div>
    </div>
  );
}
