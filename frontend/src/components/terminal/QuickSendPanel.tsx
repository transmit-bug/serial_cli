import { CommandSender } from "@/components/shared/CommandSender";
import { tauriApi } from "@/lib/tauri-api";
import { useConnectionStore } from "@/stores/connection";
import { useDataStore } from "@/stores/data";

export function QuickSendPanel() {
  const activePortId = useConnectionStore((s) => s.activePortId);
  const connections = useConnectionStore((s) => s.connections);
  const portId = activePortId;
  const isConnected =
    !!activePortId &&
    connections.find((c) => c.portId === activePortId)?.status === "connected";
  const addPacket = useDataStore((s) => s.addPacket);

  const sendFn = async (data: string, format: "hex" | "ascii") => {
    if (!portId || !isConnected) return;
    const bytes =
      format === "hex"
        ? data
            .trim()
            .split(/\s+/)
            .map((b) => parseInt(b, 16))
            .filter((n) => !Number.isNaN(n))
        : data.split("").map((c) => c.charCodeAt(0));
    await tauriApi.sendData(portId, bytes);
    addPacket(portId, "tx", bytes, Date.now());
  };

  return <CommandSender enabled={isConnected} sendFn={sendFn} />;
}
