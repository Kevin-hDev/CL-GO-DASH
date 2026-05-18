type TauriCleanup = () => void | Promise<void>;

export type TauriUnlisten = Promise<TauriCleanup>;

const cleaned = new WeakSet<TauriUnlisten>();

function ignoreCleanupFailure(result: void | Promise<void>) {
  if (result && typeof result.then === "function") {
    void result.catch(() => {});
  }
}

export function cleanupTauriListener(unlisten: TauriUnlisten) {
  if (cleaned.has(unlisten)) return;
  cleaned.add(unlisten);
  void unlisten
    .then((fn) => {
      try {
        ignoreCleanupFailure(fn());
      } catch {
        // Listener cleanup must never crash the app.
      }
    })
    .catch(() => {});
}
