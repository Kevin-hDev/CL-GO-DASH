import { type ReactNode } from "react";
import "./detail-panel.css";

interface DetailPanelProps {
  children: ReactNode;
}

export function DetailPanel({ children }: DetailPanelProps) {
  return <div className="detail-panel">{children}</div>;
}
