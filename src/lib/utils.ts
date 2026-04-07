import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/** Compare deux IDs non-secrets (UUIDs de session/tab) */
export function idMatch(a: string | null | undefined, b: string): boolean {
  if (!a) return false;
  return a.localeCompare(b) === 0; // eslint-disable-line security/detect-possible-timing-attacks
}
