import sendIcon from "@/assets/bouton-send.png";
import stopIcon from "@/assets/bouton-arret.png";
import "./chat.css";

type ButtonState = "hidden" | "send" | "stop";

interface SendStopButtonProps {
  state: ButtonState;
  onSend: () => void;
  onStop: () => void;
}

export function SendStopButton({ state, onSend, onStop }: SendStopButtonProps) {
  const isStop = state === "stop";
  const disabled = state === "hidden";
  return (
    <button
      className={`send-btn ${isStop ? "stop" : "send"}`}
      onClick={isStop ? onStop : onSend}
      disabled={disabled}
      style={{ opacity: disabled ? 0.35 : undefined }}
    >
      <img src={isStop ? stopIcon : sendIcon} alt="" style={{ width: 33, height: 33 }} />
    </button>
  );
}
