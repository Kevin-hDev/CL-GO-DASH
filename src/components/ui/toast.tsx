import { useState, useEffect, useCallback, createContext, useContext } from "react";
import type { ReactNode } from "react";
import { Check } from "@/components/ui/icons";
import { registerToast } from "@/lib/toast-emitter";
import "./toast.css";

type ToastType = "success" | "error" | "info" | "check";

interface Toast {
  id: number;
  message: string;
  type: ToastType;
}

interface ToastContextValue {
  show: (message: string, type?: ToastType) => void;
}

const ToastContext = createContext<ToastContextValue>({ show: () => {} });

export function useToast() {
  return useContext(ToastContext);
}

let nextId = 0;

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const show = useCallback((message: string, type: ToastType = "info") => {
    const id = nextId++;
    setToasts((prev) => [...prev, { id, message, type }]);
  }, []);

  useEffect(() => { registerToast(show); }, [show]);

  const remove = useCallback((id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  return (
    <ToastContext.Provider value={{ show }}>
      {children}
      <div className="toast-container">
        {toasts.map((t) => (
          <ToastItem key={t.id} toast={t} onDone={() => remove(t.id)} />
        ))}
      </div>
    </ToastContext.Provider>
  );
}

function ToastItem({ toast, onDone }: { toast: Toast; onDone: () => void }) {
  const duration = toast.type === "check" ? 2000 : 3000;

  useEffect(() => {
    const timer = setTimeout(onDone, duration);
    return () => clearTimeout(timer);
  }, [onDone, duration]);

  if (toast.type === "check") {
    return (
      <div className="toast toast-check">
        <Check size={18} weight="bold" />
      </div>
    );
  }

  return (
    <div className={`toast toast-${toast.type}`}>
      {toast.message}
    </div>
  );
}
