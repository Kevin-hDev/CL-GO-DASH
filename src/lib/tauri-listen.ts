export type TauriUnlisten = Promise<() => void>;

export function cleanupTauriListener(unlisten: TauriUnlisten) {
  void unlisten.then((fn) => fn()).catch(() => {});
}
