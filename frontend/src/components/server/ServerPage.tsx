import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useServerStore, setupServerEventListener } from "@/stores/server";
import { useConnectionStore } from "@/stores/connection";
import { ServerStatus } from "./ServerStatus";
import { ServerConfig } from "./ServerConfig";
import { ServerStats } from "./ServerStats";
import { ServerConnections } from "./ServerConnections";

export function ServerPage() {
  const { t } = useTranslation();
  const { status, refreshStatus } = useServerStore();
  const setServerOccupiedPorts = useConnectionStore(
    (s) => s.setServerOccupiedPorts,
  );

  useEffect(() => {
    // Setup event listener
    const unlisten = setupServerEventListener();

    // Initial status fetch
    refreshStatus();

    // Poll for status updates every 2 seconds when running
    const interval = status.running ? setInterval(refreshStatus, 2000) : null;

    return () => {
      unlisten.then((un) => un());
      if (interval) clearInterval(interval);
    };
  }, [status.running, refreshStatus]);

  // Update occupied ports in connection store
  useEffect(() => {
    if (status.running && status.connections) {
      const occupiedPorts = status.connections
        .map((conn) => conn.port_id)
        .filter((port): port is string => port !== null);
      setServerOccupiedPorts(occupiedPorts);
    } else {
      setServerOccupiedPorts([]);
    }
  }, [status.running, status.connections, setServerOccupiedPorts]);

  return (
    <div className="flex flex-col h-full bg-base">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-border">
        <h1 className="text-lg font-semibold text-text">{t("server.title")}</h1>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {/* Status Section */}
        <ServerStatus />

        {/* Two Column Layout */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {/* Left Column - Config & Stats */}
          <div className="space-y-4">
            <ServerConfig />
            <ServerStats />
          </div>

          {/* Right Column - Connections */}
          <div>
            <ServerConnections />
          </div>
        </div>
      </div>
    </div>
  );
}
