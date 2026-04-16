import type { WakeupSchedule } from "@/types/wakeup";

const MONTHS_FR = [
  "jan", "fév", "mar", "avr", "mai", "jun",
  "jul", "aoû", "sep", "oct", "nov", "déc",
];
const WEEKDAYS_FR = ["lun", "mar", "mer", "jeu", "ven", "sam", "dim"];

function parseHM(time: string): { h: number; m: number } | null {
  const match = /^(\d{2}):(\d{2})$/.exec(time);
  if (!match) return null;
  return { h: parseInt(match[1], 10), m: parseInt(match[2], 10) };
}

function formatTime(h: number, m: number): string {
  return `${h.toString().padStart(2, "0")}h${m.toString().padStart(2, "0")}`;
}

export function formatSchedule(schedule: WakeupSchedule): string {
  switch (schedule.kind) {
    case "once": {
      const m = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})$/.exec(schedule.datetime);
      if (!m) return schedule.datetime;
      const month = MONTHS_FR[parseInt(m[2], 10) - 1] ?? "?";
      return `${parseInt(m[3], 10)} ${month} · ${formatTime(parseInt(m[4], 10), parseInt(m[5], 10))}`;
    }
    case "daily": {
      const parsed = parseHM(schedule.time);
      if (!parsed) return schedule.time;
      return `${formatTime(parsed.h, parsed.m)} · Journalier`;
    }
    case "weekly": {
      const parsed = parseHM(schedule.time);
      const day = WEEKDAYS_FR[schedule.weekday] ?? "?";
      if (!parsed) return day;
      return `${day} ${formatTime(parsed.h, parsed.m)} · Hebdo`;
    }
  }
}
