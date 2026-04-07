import { useRef } from "react";
import { X } from "@/components/ui/icons";
import { useClickOutside } from "@/hooks/use-click-outside";
import { useKeyboard } from "@/hooks/use-keyboard";

interface FilePreviewProps {
  src: string;
  name: string;
  isImage: boolean;
  onClose: () => void;
}

export function FilePreview({ src, name, isImage, onClose }: FilePreviewProps) {
  const ref = useRef<HTMLDivElement>(null);
  useClickOutside(ref, onClose);
  useKeyboard({ onEscape: onClose });

  return (
    <div style={{
      position: "fixed", inset: 0, zIndex: 50,
      display: "flex", alignItems: "center", justifyContent: "center",
      background: "rgba(0, 0, 0, 0.7)",
    }}>
      <div ref={ref} style={{ position: "relative", maxWidth: "90vw", maxHeight: "90vh" }}>
        <button
          onClick={onClose}
          style={{
            position: "absolute", top: -12, right: -12, zIndex: 10,
            padding: 6, borderRadius: "50%",
            background: "var(--shell)", border: "1px solid var(--edge)",
            cursor: "pointer", display: "flex",
          }}
        >
          <X size={14} />
        </button>
        {isImage ? (
          <img src={src} alt={name}
            style={{ maxWidth: "100%", maxHeight: "85vh", borderRadius: "var(--radius-md)" }} />
        ) : (
          <pre style={{
            padding: "var(--space-lg)", background: "var(--shell)",
            borderRadius: "var(--radius-md)", fontSize: "var(--text-sm)",
            color: "var(--ink)", maxHeight: "85vh", overflow: "auto",
          }}>
            {src}
          </pre>
        )}
        <div style={{
          textAlign: "center", marginTop: 8,
          fontSize: "var(--text-xs)", color: "var(--ink-faint)",
        }}>
          {name}
        </div>
      </div>
    </div>
  );
}
