import { Component, type ReactNode } from "react";

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

  componentDidCatch(error: Error) {
    console.error("UI Error:", error.message);
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
          Une erreur est survenue. Relance l'application.
        </div>
      );
    }
    return this.props.children;
  }
}
