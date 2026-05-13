import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

const DOCS_WINDOW_LABEL = "forecast-docs";

export async function openForecastDocsWindow(title: string) {
  const existing = await WebviewWindow.getByLabel(DOCS_WINDOW_LABEL);
  if (existing) {
    await existing.setFocus();
    return;
  }

  const docsWindow = new WebviewWindow(DOCS_WINDOW_LABEL, {
    url: "/#/forecast-docs",
    title,
    width: 1180,
    height: 820,
    minWidth: 960,
    minHeight: 640,
    decorations: true,
    transparent: false,
  });

  await new Promise<void>((resolve, reject) => {
    const removeCreated = docsWindow.once("tauri://created", () => {
      cleanup();
      resolve();
    });
    const removeError = docsWindow.once<string>("tauri://error", (event) => {
      cleanup();
      reject(new Error(event.payload));
    });

    function cleanup() {
      void removeCreated.then((fn) => fn()).catch(() => {});
      void removeError.then((fn) => fn()).catch(() => {});
    }
  });
}
