import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { tauriApi } from "@/lib/tauri-api";
import { hexToBytes } from "@/lib/utils";
import { useProtocolStore } from "@/stores/protocol";

export function ProtocolTester() {
  const { t } = useTranslation();
  const { protocols } = useProtocolStore();
  const [selectedProtocol, setSelectedProtocol] = useState("");
  const [mode, setMode] = useState<"encode" | "decode">("encode");
  const [input, setInput] = useState("");
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleRun = useCallback(async () => {
    if (!selectedProtocol) {
      toast.error("请选择协议");
      return;
    }
    if (!input.trim()) {
      toast.error("请输入数据");
      return;
    }

    setError(null);
    setResult(null);

    try {
      const bytes = mode === "encode" ? hexToBytes(input) : hexToBytes(input);

      if (mode === "encode") {
        const encoded = await tauriApi.protocolEncode(selectedProtocol, bytes);
        setResult(JSON.stringify(encoded, null, 2));
      } else {
        const decoded = await tauriApi.protocolDecode(selectedProtocol, bytes);
        setResult(JSON.stringify(decoded, null, 2));
      }
    } catch (e) {
      setError(String(e));
    }
  }, [selectedProtocol, mode, input]);

  const customProtocols = protocols.filter(
    (p) =>
      !["ModbusRTU", "Modbus ASCII", "AT Commands", "Line"].includes(p.name),
  );

  return (
    <div className="flex flex-col h-full p-3 gap-3">
      {/* Header */}
      <div className="flex items-center gap-2">
        <select
          className="flex-1 h-7 text-xs rounded border border-border bg-transparent px-2"
          value={selectedProtocol}
          onChange={(e) => setSelectedProtocol(e.target.value)}
        >
          <option value="">{t("protocolTester.selectProtocol")}</option>
          {customProtocols.map((p) => (
            <option key={p.name} value={p.name}>
              {p.name}
            </option>
          ))}
        </select>

        <div className="flex rounded border border-border overflow-hidden text-xs">
          <button
            className={`px-3 py-1 ${mode === "encode" ? "bg-accent/20 text-accent" : "text-text-muted hover:bg-surface"}`}
            onClick={() => setMode("encode")}
          >
            {t("protocolTester.encode")}
          </button>
          <button
            className={`px-3 py-1 ${mode === "decode" ? "bg-accent/20 text-accent" : "text-text-muted hover:bg-surface"}`}
            onClick={() => setMode("decode")}
          >
            {t("protocolTester.decode")}
          </button>
        </div>
      </div>

      {/* Input */}
      <div className="flex flex-col gap-1">
        <label className="text-[10px] text-text-muted uppercase">
          {t("protocolTester.input")}
        </label>
        <textarea
          className="h-20 resize-none rounded border border-border bg-transparent p-2 font-mono text-xs"
          placeholder={mode === "encode" ? "01 03 00 00 00 01" : "raw bytes"}
          value={input}
          onChange={(e) => setInput(e.target.value)}
        />
      </div>

      {/* Run button */}
      <button
        className="h-7 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30 disabled:opacity-50"
        disabled={!selectedProtocol || !input.trim()}
        onClick={handleRun}
      >
        {t("common.run")} ▶
      </button>

      {/* Result */}
      {(result || error) && (
        <div className="flex flex-col gap-1">
          <label className="text-[10px] text-text-muted uppercase">
            {t("protocolTester.result")}
          </label>
          <pre
            className={`max-h-40 overflow-auto rounded border p-2 font-mono text-xs ${
              error
                ? "border-danger/30 bg-danger/5 text-danger"
                : "border-border bg-surface text-text"
            }`}
          >
            {error || result}
          </pre>
        </div>
      )}
    </div>
  );
}
