import { cn } from "@/lib/utils";

export type SignalState = "idle" | "live" | "error" | "ok";

const COLORS: Record<SignalState, string> = {
  idle: "bg-[var(--signal-idle)]",
  live: "bg-[var(--signal-live)] animate-pulse",
  error: "bg-[var(--signal-error)]",
  ok: "bg-[var(--signal-ok)]",
};

interface SignalDotProps {
  state: SignalState;
}

export function SignalDot({ state }: SignalDotProps) {
  return <div className={cn("size-2 rounded-full shrink-0", COLORS[state])} />;
}
