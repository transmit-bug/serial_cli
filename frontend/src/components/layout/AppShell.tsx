import { ProtocolsPage } from "@/components/protocols/ProtocolsPage";
import { ScriptsPage } from "@/components/scripts/ScriptsPage";
import { SettingsPage } from "@/components/settings/SettingsPage";
import { TerminalWorkbench } from "@/components/terminal/TerminalWorkbench";
import { VirtualPortsPage } from "@/components/virtual/VirtualPortsPage";
import { useUIStore } from "@/stores/ui";
import { Sidebar } from "./Sidebar";
import { StatusBar } from "./StatusBar";

const PAGES = {
  terminal: TerminalWorkbench,
  virtual: VirtualPortsPage,
  scripts: ScriptsPage,
  protocols: ProtocolsPage,
  settings: SettingsPage,
} as const;

export function AppShell() {
  const currentPage = useUIStore((s) => s.currentPage);
  const Page = PAGES[currentPage];

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
