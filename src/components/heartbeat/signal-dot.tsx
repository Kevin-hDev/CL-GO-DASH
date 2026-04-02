import "./signal-dot.css";

export type SignalState = "idle" | "live" | "error" | "ok";

interface SignalDotProps {
  state: SignalState;
}

export function SignalDot({ state }: SignalDotProps) {
  return <div className={`signal-dot signal-${state}`} />;
}
