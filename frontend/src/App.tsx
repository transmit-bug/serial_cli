import { Toaster } from "sonner";
import { AppShell } from "@/components/layout/AppShell";
import { useKeyboardShortcuts } from "@/hooks/useKeyboardShortcuts";

export function App() {
  useKeyboardShortcuts();

  return (
    <>
      <AppShell />
      <Toaster
        position="bottom-right"
        theme="dark"
        toastOptions={{
          style: {
            background: "var(--color-surface)",
            border: "1px solid var(--color-border)",
            color: "var(--color-text)",
          },
        }}
      />
    </>
  );
}
