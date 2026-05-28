import { useEffect } from "react";
import { useVirtualPortStore } from "@/stores/virtualPort";
import { VirtualPortDetail } from "./VirtualPortDetail";
import { VirtualPortList } from "./VirtualPortList";

export function VirtualPortsPage() {
  const { ports, selectedPort, refreshPorts } = useVirtualPortStore();

  // Refresh port list periodically
  useEffect(() => {
    refreshPorts();
    const interval = setInterval(refreshPorts, 3000);
    return () => clearInterval(interval);
  }, [refreshPorts]);

  // Auto-select first port if none selected
  useEffect(() => {
    if (!selectedPort && ports.length > 0) {
      useVirtualPortStore.getState().setSelectedPort(ports[0].id);
    }
  }, [selectedPort, ports]);

  const selectedPortInfo = ports.find((p) => p.id === selectedPort);

  return (
    <div className="flex h-full">
      {/* Left panel: port list */}
      <div className="w-80 flex-shrink-0 border-r border-border">
        <VirtualPortList />
      </div>

      {/* Right panel: detail */}
      <div className="flex-1 min-w-0">
        {selectedPortInfo ? (
          <VirtualPortDetail port={selectedPortInfo} />
        ) : (
          <div className="flex items-center justify-center h-full text-text-muted text-xs">
            {ports.length === 0
              ? "Create a virtual port pair to get started"
              : "Select a port to view details"}
          </div>
        )}
      </div>
    </div>
  );
}
