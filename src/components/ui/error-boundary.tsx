import { Component, type ErrorInfo, type ReactNode } from "react";
import i18n from "@/i18n";
import { getRecentFrontendDiagnostics, recordFrontendDiagnostic } from "@/lib/frontend-diagnostics";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false };

  static getDerivedStateFromError(): State {
    return { hasError: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    recordFrontendDiagnostic("error-boundary.catch", {
      message: error.message,
      stack: error.stack,
      componentStack: info.componentStack,
      recent: getRecentFrontendDiagnostics(),
    });
  }

  render() {
    if (this.state.hasError) {
      return (
        <div style={{
          padding: "var(--space-2xl)",
          color: "var(--signal-error)",
          textAlign: "center",
          fontSize: "var(--text-sm)",
        }}>
          {i18n.t("errors.crashMessage")}
        </div>
      );
    }
    return this.props.children;
  }
}
