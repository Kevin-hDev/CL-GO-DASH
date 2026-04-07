import { CaretRight, X } from "@/components/ui/icons";
import "./chat.css";

type ButtonState = "hidden" | "send" | "stop";

interface SendStopButtonProps {
  state: ButtonState;
  onSend: () => void;
  onStop: () => void;
}

export function SendStopButton({ state, onSend, onStop }: SendStopButtonProps) {
  if (state === "hidden") return null;

  const isSend = state === "send";
  return (
    <button
      className={`send-btn ${isSend ? "send" : "stop"}`}
      onClick={isSend ? onSend : onStop}
    >
      {isSend ? <CaretRight size={16} weight="bold" /> : <X size={14} />}
    </button>
  );
}
