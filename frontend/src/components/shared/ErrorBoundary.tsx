import { Component, type ErrorInfo, type ReactNode } from "react";
import { tauriApi } from "@/lib/tauri-api";

interface Props {
  children: ReactNode;
}

interface State {
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("Uncaught error:", error, info.componentStack);
  }

  render() {
    if (this.state.error) {
      return (
        <div className="flex flex-col items-center justify-center h-full gap-4 p-8 bg-base text-text">
          <h1 className="text-lg font-semibold text-danger">
            Something went wrong
          </h1>
          <pre className="text-xs text-text-muted max-w-xl overflow-auto bg-surface p-3 rounded border border-border">
            {this.state.error.message}
          </pre>
          <button
            className="px-4 py-2 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30"
            onClick={() => this.setState({ error: null })}
          >
            Try Again
          </button>
          <button
            className="px-4 py-2 rounded text-xs text-text-muted hover:text-text"
            onClick={() => {
              this.setState({ error: null });
              tauriApi.getConfig().catch(() => {});
              window.location.reload();
            }}
          >
            Reload App
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
