import { useState, useRef, useCallback, useEffect } from "react";
import { useClickOutside } from "@/hooks/use-click-outside";
import { useKeyboard } from "@/hooks/use-keyboard";

interface ConfirmButtonProps {
  label: string;
  confirmLabel: string;
  onConfirm: () => void;
  disabled?: boolean;
  className?: string;
}

export function ConfirmButton({
  label,
  confirmLabel,
  onConfirm,
  disabled,
  className,
}: ConfirmButtonProps) {
  const [confirming, setConfirming] = useState(false);
  const ref = useRef<HTMLButtonElement>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  const cancel = useCallback(() => setConfirming(false), []);

  useClickOutside(ref, confirming ? cancel : () => {});

  useKeyboard({
    onEscape: confirming ? cancel : undefined,
    onEnter: confirming ? onConfirm : undefined,
  });

  useEffect(() => {
    if (confirming) {
      timerRef.current = setTimeout(cancel, 3000);
    }
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [confirming, cancel]);

  const handleClick = () => {
    if (confirming) {
      onConfirm();
      setConfirming(false);
    } else {
      setConfirming(true);
    }
  };

  return (
    <button
      ref={ref}
      className={className}
      onClick={handleClick}
      disabled={disabled}
      style={confirming ? {
        color: "var(--signal-error)",
        borderColor: "var(--signal-error)",
      } : undefined}
    >
      {confirming ? confirmLabel : label}
    </button>
  );
}
