type ToastType = "success" | "error" | "info" | "check";
type ToastFn = (message: string, type?: ToastType) => void;

let _show: ToastFn = () => {};

/** Called by ToastProvider to register the show function */
export function registerToast(fn: ToastFn) {
  _show = fn;
}

/** Show a toast from anywhere (hooks, services, etc.) */
export function showToast(message: string, type: ToastType = "error") {
  _show(message, type);
}
