import { useCallback, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Toggle } from "@/components/ui/toggle";
import { tauriApi } from "@/lib/tauri-api";
import { hexToBytes } from "@/lib/utils";
import { useConnectionStore } from "@/stores/connection";
import { useDataStore } from "@/stores/data";
import { QuickSendPanel } from "./QuickSendPanel";
import { SequenceList } from "./SequenceList";

export function TxSender({ portId }: { portId?: string }) {
  const { t } = useTranslation();
  const isConnected = useConnectionStore((s) =>
    s.connections.some((c) => c.portId === portId && c.status === "connected"),
  );
  const addPacket = useDataStore((s) => s.addPacket);

  const [tab, setTab] = useState<"free" | "quick" | "sequences">("free");
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
      addPacket(portId, "tx", data, Date.now());

      if (historyRef.current[historyRef.current.length - 1] !== input) {
        historyRef.current.push(input);
        if (historyRef.current.length > 100) historyRef.current.shift();
      }
      historyIdx.current = historyRef.current.length;
    } catch (e) {
      toast.error(String(e));
    }
  }, [portId, input, hexMode, addPacket]);

  const handleQuickSend = useCallback(
    (data: string, _format: "hex" | "ascii") => {
      if (historyRef.current[historyRef.current.length - 1] !== data) {
        historyRef.current.push(data);
        if (historyRef.current.length > 100) historyRef.current.shift();
      }
      historyIdx.current = historyRef.current.length;
    },
    [],
  );

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
    <Tabs
      value={tab}
      onValueChange={(value) => setTab(value as "free" | "quick" | "sequences")}
      className="flex flex-col h-full"
    >
      <TabsList variant="line" className="shrink-0">
        <TabsTrigger value="free">{t("txSender.freeSend")}</TabsTrigger>
        <TabsTrigger value="quick">{t("txSender.quickSend")}</TabsTrigger>
        <TabsTrigger value="sequences">{t("txSender.sequences")}</TabsTrigger>
      </TabsList>

      <TabsContent value="free" className="flex-1 flex flex-col mt-0">
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
          <Toggle
            pressed={hexMode}
            onPressedChange={setHexMode}
            variant="outline"
            size="sm"
          >
            {t("terminal.hexMode")}
          </Toggle>

          <Button
            variant="default"
            onClick={send}
            disabled={!isConnected || !input.trim()}
          >
            {t("common.send")} ▶
          </Button>

          {loopActive && (
            <Input
              type="number"
              value={loopInterval}
              onChange={(e) => setLoopInterval(Number(e.target.value))}
              className="w-20"
              min={100}
            />
          )}
          <Toggle
            pressed={loopActive}
            onPressedChange={toggleLoop}
            disabled={!isConnected}
            variant={loopActive ? "outline" : "default"}
            size="sm"
          >
            {t("terminal.sendLoop")} {loopActive ? "■" : "↻"}
          </Toggle>
        </div>
      </TabsContent>

      <TabsContent value="quick" className="flex-1 mt-0">
        <QuickSendPanel onSent={handleQuickSend} />
      </TabsContent>

      <TabsContent value="sequences" className="flex-1 mt-0">
        <SequenceList />
      </TabsContent>
    </Tabs>
  );
}
