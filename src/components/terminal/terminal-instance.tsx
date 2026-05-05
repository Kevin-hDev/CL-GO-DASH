import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { IS_MAC } from "@/lib/platform";
import i18n from "@/i18n";
import "@xterm/xterm/css/xterm.css";

interface TerminalInstanceProps {
  tabId: string;
  cwd: string;
  isVisible: boolean;
  onPtyReady: (tabId: string, ptyId: number) => void;
  onExit: (tabId: string) => void;
  onTogglePanel?: () => void;
}

function getThemeColors() {
  const style = getComputedStyle(document.documentElement);
  return {
    background: style.getPropertyValue("--void").trim() || "#050b0f",
    foreground: style.getPropertyValue("--ink").trim() || "#e8e6e3",
    cursor: style.getPropertyValue("--ink").trim() || "#e8e6e3",
    cursorAccent: style.getPropertyValue("--void").trim() || "#050b0f",
    selectionBackground: "rgba(255,255,255,0.15)",
  };
}

export function TerminalInstance({
  tabId,
  cwd,
  isVisible,
  onPtyReady,
  onExit,
  onTogglePanel,
}: TerminalInstanceProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  const ptyIdRef = useRef<number | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const term = new Terminal({
      theme: getThemeColors(),
      fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
      fontSize: 13,
      cursorBlink: true,
      allowProposedApi: true,
      rightClickSelectsWord: true,
    });

    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(containerRef.current);

    fit.fit();
    termRef.current = term;
    fitRef.current = fit;

    term.attachCustomKeyEventHandler((e) => {
      if (e.type !== "keydown") return true;

      const toggleMod = IS_MAC ? e.metaKey : e.ctrlKey;
      if (toggleMod && e.code === "KeyJ") {
        onTogglePanel?.();
        return false;
      }

      const copyPasteMod = IS_MAC ? e.metaKey : e.ctrlKey;

      if (copyPasteMod && e.code === "KeyC") {
        const selection = term.getSelection();
        if (selection) {
          navigator.clipboard.writeText(selection).catch(() => {});
          return false;
        }
        return true;
      }

      if (copyPasteMod && e.code === "KeyV") {
        return true;
      }

      return true;
    });

    const pasteHandler = (e: ClipboardEvent) => {
      const text = e.clipboardData?.getData("text");
      if (text && ptyIdRef.current !== null) {
        invoke("pty_write", { id: ptyIdRef.current, data: text }).catch(() => {});
        e.preventDefault();
      }
    };
    containerRef.current.addEventListener("paste", pasteHandler);

    let disposed = false;

    invoke<number>("pty_spawn", {
      cwd: cwd || null,
      cols: term.cols || 80,
      rows: term.rows || 24,
    }).then((id) => {
      if (disposed) {
        invoke("pty_kill", { id }).catch(() => {});
        return;
      }
      ptyIdRef.current = id;
      onPtyReady(tabId, id);

      term.onData((data) => {
        invoke("pty_write", { id, data }).catch(() => {});
      });

      term.onResize(({ cols, rows }) => {
        invoke("pty_resize", { id, cols, rows }).catch(() => {});
      });
    }).catch(() => {
      if (!disposed) {
        term.writeln(`\r\n${i18n.t("terminal.failedToStart")}\r\n`);
      }
    });

    const unlisten1 = listen<{ id: number; data: string }>("pty-output", (event) => {
      if (event.payload.id === ptyIdRef.current) {
        term.write(event.payload.data);
      }
    });

    const unlisten2 = listen<{ id: number; code: number }>("pty-exit", (event) => {
      if (event.payload.id === ptyIdRef.current) {
        term.writeln(`\r\n[${i18n.t("terminal.processExited", { code: event.payload.code })}]`);
        ptyIdRef.current = null;
        onExit(tabId);
      }
    });

    let resizeTimer: ReturnType<typeof setTimeout>;
    const resizeObserver = new ResizeObserver(() => {
      clearTimeout(resizeTimer);
      resizeTimer = setTimeout(() => {
        if (containerRef.current && containerRef.current.offsetWidth > 0) {
          fit.fit();
        }
      }, 100);
    });
    resizeObserver.observe(containerRef.current);

    const container = containerRef.current;
    return () => {
      disposed = true;
      clearTimeout(resizeTimer);
      container?.removeEventListener("paste", pasteHandler);
      resizeObserver.disconnect();
      void unlisten1.then((fn) => fn());
      void unlisten2.then((fn) => fn());
      if (ptyIdRef.current !== null) {
        invoke("pty_kill", { id: ptyIdRef.current }).catch(() => {});
      }
      term.dispose();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps -- PTY mount-only setup
  }, []);

  useEffect(() => {
    if (isVisible && fitRef.current) {
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          fitRef.current?.fit();
          termRef.current?.focus();
        });
      });
    }
  }, [isVisible]);

  useEffect(() => {
    const observer = new MutationObserver(() => {
      if (termRef.current) {
        termRef.current.options.theme = getThemeColors();
      }
    });
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
    return () => observer.disconnect();
  }, []);

  return (
    <div
      ref={containerRef}
      style={{
        width: "100%",
        height: "100%",
        visibility: isVisible ? "visible" : "hidden",
        position: isVisible ? "relative" : "absolute",
        top: 0,
        left: 0,
      }}
    />
  );
}
