import { describe, expect, it } from "vitest";
import {
  normalizeBrowserUrl,
  parseBrowserSession,
  parseLocalSiteScan,
} from "../browser-types";

const TAB_ID = "0123456789abcdef0123456789abcdef";

function session(overrides: Record<string, unknown> = {}) {
  return {
    tabs: [{
      id: TAB_ID,
      title: "Exemple",
      url: "https://example.com/",
      loading: false,
      canGoBack: false,
      canGoForward: true,
      released: false,
    }],
    activeTabId: TAB_ID,
    generation: 2,
    ...overrides,
  };
}

describe("browser types", () => {
  it("accepte uniquement les URL HTTP et HTTPS sans identifiants", () => {
    expect(normalizeBrowserUrl("https://example.com")).toBe("https://example.com/");
    expect(normalizeBrowserUrl("http://localhost:3000/a")).toBe("http://localhost:3000/a");
    expect(normalizeBrowserUrl("https://user:secret@example.com")).toBeNull();
    expect(normalizeBrowserUrl(" file:///tmp/test ")).toBeNull();
    expect(normalizeBrowserUrl(`https://example.com/${"a".repeat(2_048)}`)).toBeNull();
  });

  it("rejette les sessions anciennes, dupliquées ou non bornées", () => {
    expect(parseBrowserSession(session())).toEqual(session());
    expect(parseBrowserSession(session({ generation: 0 }))).toBeNull();
    expect(parseBrowserSession(session({ activeTabId: "f".repeat(32) }))).toBeNull();
    expect(parseBrowserSession(session({ tabs: Array(11).fill(session().tabs[0]) }))).toBeNull();
    expect(parseBrowserSession(session({ tabs: [session().tabs[0], session().tabs[0]] }))).toBeNull();
  });

  it("borne et valide les sites localhost avant affichage", () => {
    const scan = {
      sites: [{ url: "http://localhost:3000/", title: "Application", port: 3000, protocol: "http" }],
      generation: 3,
      changed: true,
    };
    expect(parseLocalSiteScan(scan)).toEqual(scan);
    expect(parseLocalSiteScan({ ...scan, sites: Array(33).fill(scan.sites[0]) })).toBeNull();
    expect(parseLocalSiteScan({
      ...scan,
      sites: [{ ...scan.sites[0], url: "http://192.168.1.2:3000/" }],
    })).toBeNull();
  });
});
