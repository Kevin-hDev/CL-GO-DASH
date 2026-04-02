import { useState, useRef, useCallback, useEffect } from "react";

interface DatetimeInputProps {
  value: string;
  onChange: (value: string) => void;
  className?: string;
}

/**
 * Input datetime-local non-contrôlé pendant la saisie.
 * Sauvegarde uniquement au blur ou Enter.
 * Ne re-render pas pendant que l'utilisateur tape.
 */
export function DatetimeInput({ value, onChange, className }: DatetimeInputProps) {
  const ref = useRef<HTMLInputElement>(null);
  const [localVal, setLocalVal] = useState(value);
  const mountedRef = useRef(false);

  // Sync from parent only on mount or if value changes externally
  useEffect(() => {
    if (!mountedRef.current) {
      mountedRef.current = true;
      return;
    }
    // Only sync if the input is NOT focused (user not editing)
    if (ref.current !== document.activeElement) {
      setLocalVal(value);
    }
  }, [value]);

  const handleBlur = useCallback(() => {
    if (localVal && localVal !== value) {
      onChange(localVal);
    }
  }, [localVal, value, onChange]);

  return (
    <input
      ref={ref}
      type="datetime-local"
      className={className}
      value={localVal}
      onChange={(e) => setLocalVal(e.target.value)}
      onBlur={handleBlur}
    />
  );
}
