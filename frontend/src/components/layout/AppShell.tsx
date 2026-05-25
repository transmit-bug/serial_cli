import { EditorPage } from "@/components/editor/EditorPage";
import { SettingsPage } from "@/components/settings/SettingsPage";
import { TerminalWorkbench } from "@/components/terminal/TerminalWorkbench";
import { VirtualPortsPage } from "@/components/virtual/VirtualPortsPage";
import { useConnectionStore } from "@/stores/connection";
import { useVirtualPortStore } from "@/stores/virtualPort";
import { useUIStore } from "@/stores/ui";
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { toast } from "sonner";
import { Sidebar } from "./Sidebar";
import { StatusBar } from "./StatusBar";

const PAGES = {
  terminal: TerminalWorkbench,
  virtual: VirtualPortsPage,
  editor: EditorPage,
  settings: SettingsPage,
} as const;

export function AppShell() {
  const currentPage = useUIStore((s) => s.currentPage);
  const refreshPorts = useConnectionStore((s) => s.refreshPorts);
  const refreshVirtualPorts = useVirtualPortStore((s) => s.refreshPorts);
  const Page = PAGES[currentPage];

  // Auto-refresh hardware ports on hot-plug
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen("ports-changed", (event) => {
      const payload = event.payload as { added: string[]; removed: string[] };
      refreshPorts();
      // If a removed port is currently connected, trigger disconnect
      if (payload?.removed) {
        const { connections, disconnect } = useConnectionStore.getState();
        for (const conn of connections) {
          if (
            conn.status === "connected" &&
            payload.removed.some((name) => conn.portName === name)
          ) {
            disconnect(conn.portId);
          }
        }
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, [refreshPorts]);

  // Auto-refresh virtual ports on mount
  useEffect(() => {
    refreshVirtualPorts();
  }, [refreshVirtualPorts]);

  // Show backend errors as toasts
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen("error-occurred", (event) => {
      const payload = event.payload as { error: string };
      if (payload?.error) {
        toast.error(payload.error);
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  return (
    <div className="flex h-full">
      <Sidebar />
      <div className="flex flex-col flex-1 min-w-0">
        <main className="flex-1 overflow-hidden">
          <Page />
        </main>
        <StatusBar />
      </div>
    </div>
  );
}
