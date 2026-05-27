import { Toaster } from "sonner";
import { AppShell } from "@/components/layout/AppShell";
import { ErrorBoundary } from "@/components/shared/ErrorBoundary";
import { ShortcutsHelp } from "@/components/shared/ShortcutsHelp";
import { useKeyboardShortcuts } from "@/hooks/useKeyboardShortcuts";
import { useTheme } from "@/hooks/useTheme";

export function App() {
  const { showHelp, setShowHelp } = useKeyboardShortcuts();
  const theme = useTheme();

  return (
    <ErrorBoundary>
      <AppShell />
      <Toaster
        position="bottom-right"
        theme={theme === "light" ? "light" : "dark"}
        toastOptions={{
          style: {
            background: "var(--color-surface)",
            border: "1px solid var(--color-border)",
            color: "var(--color-text)",
          },
        }}
      />
      {showHelp && <ShortcutsHelp onClose={() => setShowHelp(false)} />}
    </ErrorBoundary>
  );
}
