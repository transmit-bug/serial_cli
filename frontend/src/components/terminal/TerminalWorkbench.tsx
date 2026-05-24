import { PanelRight, PanelRightClose } from "lucide-react";
import { useCallback } from "react";
import { Group, Panel, Separator } from "react-resizable-panels";
import { useTauriEvent } from "@/hooks/useTauriEvent";
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
  const addPacket = useDataStore((s) => s.addPacket);

  // Listen for data events from Tauri backend
  const handleDataReceived = useCallback(
    (payload: DataEventPayload) => {
      addPacket("rx", payload.data, payload.timestamp);
    },
    [addPacket],
  );

  const handleDataSent = useCallback(
    (payload: DataEventPayload) => {
      addPacket("tx", payload.data, payload.timestamp);
    },
    [addPacket],
  );

  useTauriEvent<DataEventPayload>("data-received", handleDataReceived);
  useTauriEvent<DataEventPayload>("data-sent", handleDataSent);

  return (
    <div className="flex flex-col h-full">
      <ConnectionBar />

      <Group orientation="horizontal" className="flex-1">
        <Panel defaultSize={65} minSize={50}>
          <Group orientation="vertical">
            <Panel defaultSize={65} minSize={30}>
              <RxViewer />
            </Panel>
            <Separator className="h-px bg-border hover:bg-accent transition-colors" />
            <Panel defaultSize={35} minSize={15}>
              <TxSender />
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

      {/* Toggle right panel button */}
      <button
        onClick={toggleRightPanel}
        className="fixed top-2 right-2 z-50 p-1 rounded bg-surface/80 text-text-muted hover:text-text"
        title="Toggle right panel (Cmd+\)"
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
