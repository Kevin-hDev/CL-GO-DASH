import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  zoomJumpBarCount,
  zoomJumpCurrentIndex,
  zoomJumpTarget,
  zoomJumpVisible,
} from "./forecast-zoom-jump-utils";
import "./forecast-zoom-jump-bars.css";

interface ForecastZoomJumpBarsProps {
  window: { start: number; end: number } | null;
  onJump: (start: number) => void;
}

const BASE_HEIGHT = 10;
const HOVER_HEIGHTS = [20, 15, 12.5];

export function ForecastZoomJumpBars({ window, onJump }: ForecastZoomJumpBarsProps) {
  const { t } = useTranslation();
  const [hovered, setHovered] = useState<number | null>(null);
  const span = window ? window.end - window.start : 100;
  const start = window?.start ?? 0;
  const visible = zoomJumpVisible(span);
  const count = zoomJumpBarCount(span);
  const current = zoomJumpCurrentIndex(start, span, count);

  const bars = Array.from({ length: count }, (_, index) => {
    const dist = hovered === null ? null : Math.abs(index - hovered);
    const height =
      dist === null ? BASE_HEIGHT : (HOVER_HEIGHTS[dist] ?? BASE_HEIGHT);
    const classes = ["fzjb-bar"];
    if (index === current) classes.push("is-current");
    if (dist === 0) classes.push("is-hovered");
    return (
      <button
        key={index}
        type="button"
        className={classes.join(" ")}
        data-dist={dist ?? undefined}
        style={{ height }}
        tabIndex={visible ? 0 : -1}
        aria-label={t("forecast.zoomJump.goTo", { index: index + 1, count })}
        onClick={() => onJump(zoomJumpTarget(index, count, span))}
        onMouseEnter={() => setHovered(index)}
        onMouseLeave={() => setHovered(null)}
        onFocus={() => setHovered(index)}
        onBlur={() => setHovered(null)}
      />
    );
  });

  return (
    <div
      className={`fzjb-root ${visible ? "is-visible" : ""}`}
      role="group"
      aria-label={t("forecast.zoomJump.label")}
    >
      {bars}
    </div>
  );
}
