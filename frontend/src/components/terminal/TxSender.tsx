import { useCallback, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { tauriApi } from "@/lib/tauri-api";
import { hexToBytes } from "@/lib/utils";
import { useConnectionStore } from "@/stores/connection";
import { useDataStore } from "@/stores/data";

export function TxSender() {
  const { t } = useTranslation();
  const isConnected = useConnectionStore((s) => s.status === "connected");
  const portId = useConnectionStore((s) => s.portId);
  const addPacket = useDataStore((s) => s.addPacket);

  const [input, setInput] = useState("");
  const [hexMode, setHexMode] = useState(false);
  const [loopActive, setLoopActive] = useState(false);
  const [loopInterval, setLoopInterval] = useState(1000);
  const loopRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const historyRef = useRef<string[]>([]);
  const historyIdx = useRef(-1);

  const send = useCallback(async () => {
    if (!portId || !input.trim()) return;

    try {
      let data: number[];
      if (hexMode) {
        data = hexToBytes(input);
      } else {
        data = Array.from(new TextEncoder().encode(input));
      }

      await tauriApi.sendData(portId, data);
      addPacket("tx", data, Date.now());

      // Add to history
      if (historyRef.current[historyRef.current.length - 1] !== input) {
        historyRef.current.push(input);
        if (historyRef.current.length > 100) historyRef.current.shift();
      }
      historyIdx.current = historyRef.current.length;
    } catch (e) {
      toast.error(String(e));
    }
  }, [portId, input, hexMode, addPacket]);

  const toggleLoop = useCallback(() => {
    if (loopActive) {
      if (loopRef.current) clearInterval(loopRef.current);
      loopRef.current = null;
      setLoopActive(false);
    } else {
      send();
      loopRef.current = setInterval(send, loopInterval);
      setLoopActive(true);
    }
  }, [loopActive, send, loopInterval]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        send();
      } else if (e.key === "ArrowUp" && e.metaKey) {
        e.preventDefault();
        if (historyIdx.current > 0) {
          historyIdx.current--;
          setInput(historyRef.current[historyIdx.current] ?? "");
        }
      } else if (e.key === "ArrowDown" && e.metaKey) {
        e.preventDefault();
        if (historyIdx.current < historyRef.current.length - 1) {
          historyIdx.current++;
          setInput(historyRef.current[historyIdx.current] ?? "");
        } else {
          historyIdx.current = historyRef.current.length;
          setInput("");
        }
      }
    },
    [send],
  );

  return (
    <div className="flex flex-col h-full">
      <textarea
        value={input}
        onChange={(e) => setInput(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={t("terminal.inputPlaceholder")}
        disabled={!isConnected}
        className="flex-1 resize-none p-2 font-mono text-xs border-b border-border rounded-none"
        spellCheck={false}
      />
      <div className="flex items-center gap-2 px-3 py-1.5 shrink-0">
        <button
          onClick={() => setHexMode(!hexMode)}
          className={`px-2 py-0.5 rounded text-xs ${
            hexMode ? "bg-warning/20 text-warning" : "text-text-muted"
          }`}
        >
          {t("terminal.hexMode")}
        </button>

        <button
          onClick={send}
          disabled={!isConnected || !input.trim()}
          className="px-3 py-1 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30 disabled:opacity-50"
        >
          {t("common.send")} ▶
        </button>

        {loopActive && (
          <input
            type="number"
            value={loopInterval}
            onChange={(e) => setLoopInterval(Number(e.target.value))}
            className="w-20 h-6 text-xs"
            min={100}
          />
        )}
        <button
          onClick={toggleLoop}
          disabled={!isConnected}
          className={`px-2 py-0.5 rounded text-xs ${
            loopActive
              ? "bg-danger/20 text-danger"
              : "text-text-muted hover:text-text"
          }`}
        >
          {t("terminal.sendLoop")} {loopActive ? "■" : "↻"}
        </button>
      </div>
    </div>
  );
}
