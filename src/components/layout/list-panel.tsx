import { type ReactNode } from "react";
import "./list-panel.css";

interface ListPanelProps {
  children: ReactNode;
}

export function ListPanel({ children }: ListPanelProps) {
  return <div className="list-panel">{children}</div>;
}
