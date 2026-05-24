import { PanelRight, PanelRightClose, X } from "lucide-react";
import { useCallback } from "react";
import { Group, Panel, Separator } from "react-resizable-panels";
import { useTauriEvent } from "@/hooks/useTauriEvent";
import { useConnectionStore } from "@/stores/connection";
import { useDataStore } from "@/stores/data";
import { useUIStore } from "@/stores/ui";
import { ConnectionBar } from "./ConnectionBar";
import { RightPanel } from "./RightPanel";
import { RxViewer } from "./RxViewer";
import { TxSender } from "./TxSender";

interface DataEventPayload {
  port_id: string;
  data: number[];
  timestamp: number;
  direction: "rx" | "tx";
}

export function TerminalWorkbench() {
  const rightPanelCollapsed = useUIStore((s) => s.rightPanelCollapsed);
  const toggleRightPanel = useUIStore((s) => s.toggleRightPanel);
  const connections = useConnectionStore((s) => s.connections);
  const activePortId = useConnectionStore((s) => s.activePortId);
  const setActivePort = useConnectionStore((s) => s.setActivePort);
  const disconnect = useConnectionStore((s) => s.disconnect);
  const addPacket = useDataStore((s) => s.addPacket);

  const handleDataReceived = useCallback(
    (payload: DataEventPayload) => {
      addPacket(payload.port_id, "rx", payload.data, payload.timestamp);
    },
    [addPacket],
  );

  const handleDataSent = useCallback(
    (payload: DataEventPayload) => {
      addPacket(payload.port_id, "tx", payload.data, payload.timestamp);
    },
    [addPacket],
  );

  useTauriEvent<DataEventPayload>("data-received", handleDataReceived);
  useTauriEvent<DataEventPayload>("data-sent", handleDataSent);

  const connectedPorts = connections.filter((c) => c.status === "connected");

  return (
    <div className="flex flex-col h-full">
      <ConnectionBar />

      {/* Port tabs */}
      {connectedPorts.length > 0 && (
        <div className="flex items-center gap-1 px-2 py-1 border-b border-border bg-base-deep overflow-x-auto">
          {connectedPorts.map((entry) => (
            <button
              key={entry.portId}
              onClick={() => setActivePort(entry.portId)}
              className={`flex items-center gap-1 px-2 py-0.5 rounded text-xs whitespace-nowrap transition-colors ${
                entry.portId === activePortId
                  ? "bg-accent/20 text-accent"
                  : "text-text-muted hover:text-text"
              }`}
            >
              <span className="font-mono">{entry.portName}</span>
              <span className="text-[10px] opacity-60">
                {entry.portStatus
                  ? ` RX:${entry.portStatus.bytes_received}`
                  : ""}
              </span>
              <X
                size={12}
                className="opacity-40 hover:opacity-100 cursor-pointer"
                onClick={(e) => {
                  e.stopPropagation();
                  disconnect(entry.portId);
                }}
              />
            </button>
          ))}
        </div>
      )}

      <Group orientation="horizontal" className="flex-1">
        <Panel defaultSize={65} minSize={50}>
          <Group orientation="vertical">
            <Panel defaultSize={65} minSize={30}>
              <RxViewer portId={activePortId ?? undefined} />
            </Panel>
            <Separator className="h-px bg-border hover:bg-accent transition-colors" />
            <Panel defaultSize={35} minSize={15}>
              <TxSender portId={activePortId ?? undefined} />
            </Panel>
          </Group>
        </Panel>

        {!rightPanelCollapsed && (
          <>
            <Separator className="w-1.5 bg-border hover:bg-accent cursor-col-resize transition-colors" />
            <Panel defaultSize={35} minSize={25}>
              <RightPanel />
            </Panel>
          </>
        )}
      </Group>

      <button
        onClick={toggleRightPanel}
        className="fixed top-2 right-2 z-50 p-1 rounded bg-surface/80 text-text-muted hover:text-text"
        title="Toggle right panel (Cmd+\\)"
      >
        {rightPanelCollapsed ? (
          <PanelRight size={14} />
        ) : (
          <PanelRightClose size={14} />
        )}
      </button>
    </div>
  );
}
