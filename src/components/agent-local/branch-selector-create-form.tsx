import type { KeyboardEvent, RefObject } from "react";

interface BranchSelectorCreateFormProps {
  inputRef: RefObject<HTMLInputElement | null>;
  value: string;
  error: string;
  placeholder: string;
  onValueChange: (value: string) => void;
  onSubmit: () => void;
  onCancel: () => void;
}

export function BranchSelectorCreateForm({
  inputRef,
  value,
  error,
  placeholder,
  onValueChange,
  onSubmit,
  onCancel,
}: BranchSelectorCreateFormProps) {
  const handleKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter") onSubmit();
    if (event.key === "Escape") {
      event.stopPropagation();
      onCancel();
    }
  };

  return (
    <div className="bs-create-form">
      <input
        ref={inputRef}
        className="bs-create-input"
        placeholder={placeholder}
        value={value}
        onChange={(event) => onValueChange(event.target.value)}
        onKeyDown={handleKeyDown}
      />
      {error && <div className="bs-create-error">{error}</div>}
    </div>
  );
}
