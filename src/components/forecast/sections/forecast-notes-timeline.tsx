import type { TFunction } from "i18next";
import type { CSSProperties } from "react";
import type { ForecastNote, NoteRange } from "./forecast-notes-types";
import { formatNoteDate, midpointDate, notePosition } from "./forecast-notes-utils";
import "./forecast-notes-timeline.css";

interface ForecastNotesTimelineProps {
  notes: ForecastNote[];
  selectedId: string | null;
  range: NoteRange;
  height: number;
  locale: string;
  t: TFunction;
  onSelect: (id: string) => void;
}

export function ForecastNotesTimeline({
  notes,
  selectedId,
  range,
  height,
  locale,
  t,
  onSelect,
}: ForecastNotesTimelineProps) {
  return (
    <section className="fcn-timeline" aria-label={t("forecast.notes.timeline")}>
      <div className="fcn-rail" style={{ height }}>
        <div className="fcn-rail-track">
          <span className="fcn-rail-line" />
          {notes.map((note, index) => (
            <button
              key={note.id}
              type="button"
              className={`fcn-dot is-${note.source} ${note.id === selectedId ? "is-selected" : ""}`}
              style={dotStyle(note, range, index)}
              title={`${note.title} · ${formatNoteDate(note.date, locale)}`}
              onClick={() => onSelect(note.id)}
            />
          ))}
        </div>
        {notes.length === 0 && (
          <span className="fcn-rail-empty">{t("forecast.notes.timelineEmpty")}</span>
        )}
      </div>
      <div className="fcn-axis">
        <div className="fcn-axis-track">
          {buildMarkers(notes, range).map((marker) => (
            <span
              key={marker.date}
              className={`fcn-axis-mark is-${marker.edge}`}
              style={{ left: `${marker.position}%` }}
            >
              {formatNoteDate(marker.date, locale)}
            </span>
          ))}
        </div>
      </div>
    </section>
  );
}

function buildMarkers(notes: ForecastNote[], range: NoteRange) {
  const values = [
    new Date(range.start).toISOString(),
    midpointDate(range),
    new Date(range.end).toISOString(),
    ...notes.map((note) => note.date),
  ];
  const unique = Array.from(new Set(values.map((date) => date.slice(0, 10))));
  return unique
    .map((date) => ({
      date,
      position: notePosition(date, range),
      edge: notePosition(date, range) <= 2 ? "start" : notePosition(date, range) >= 98 ? "end" : "middle",
    }))
    .sort((a, b) => a.position - b.position);
}

function dotStyle(note: ForecastNote, range: NoteRange, index: number): CSSProperties {
  return {
    left: `${notePosition(note.date, range)}%`,
    top: "50%",
    zIndex: index + 1,
  };
}
