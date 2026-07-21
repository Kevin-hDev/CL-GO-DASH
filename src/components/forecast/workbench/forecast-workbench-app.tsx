import { useEffect } from "react";
import { useTheme } from "@/hooks/use-theme";
import { ForecastWorkbenchWindow } from "./forecast-workbench-window";

export function ForecastWorkbenchApp() {
  useTheme();
  useEffect(() => {
    const splash = document.getElementById("splash");
    if (!splash) return;
    requestAnimationFrame(() => splash.remove());
  }, []);
  return <ForecastWorkbenchWindow />;
}
