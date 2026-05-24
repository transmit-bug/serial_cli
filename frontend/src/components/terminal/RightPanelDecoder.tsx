import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";

type EncodingMode = "hex" | "ascii" | "base64" | "binary" | "decimal";

const MODES: EncodingMode[] = ["hex", "ascii", "base64", "binary", "decimal"];

export function RightPanelDecoder() {
  const { t } = useTranslation();
  const [input, setInput] = useState("");
  const [inputMode, setInputMode] = useState<EncodingMode>("hex");

  const decodeInput = useCallback(
    (mode: EncodingMode, raw: string): number[] => {
      try {
        switch (mode) {
          case "hex":
            return raw
              .trim()
              .split(/\s+/)
              .filter(Boolean)
              .map((b) => Number.parseInt(b, 16));
          case "ascii":
            return Array.from(new TextEncoder().encode(raw));
          case "base64": {
            const decoded = atob(raw.trim());
            return Array.from(new TextEncoder().encode(decoded));
          }
          case "binary":
            return raw
              .trim()
              .split(/\s+/)
              .filter(Boolean)
              .map((b) => Number.parseInt(b, 2));
          case "decimal":
            return raw
              .trim()
              .split(/\s+/)
              .filter(Boolean)
              .map((b) => Number.parseInt(b, 10));
        }
      } catch {
        return [];
      }
    },
    [],
  );

  const encodeOutput = useCallback(
    (mode: EncodingMode, bytes: number[]): string => {
      switch (mode) {
        case "hex":
          return bytes
            .map((b) => b.toString(16).padStart(2, "0"))
            .join(" ")
            .toUpperCase();
        case "ascii":
          return bytes
            .map((b) => (b >= 32 && b <= 126 ? String.fromCharCode(b) : "."))
            .join("");
        case "base64":
          return btoa(String.fromCharCode(...bytes));
        case "binary":
          return bytes.map((b) => b.toString(2).padStart(8, "0")).join(" ");
        case "decimal":
          return bytes.map((b) => b.toString(10)).join(" ");
      }
    },
    [],
  );

  const bytes = useMemo(
    () => decodeInput(inputMode, input),
    [input, inputMode, decodeInput],
  );

  const results = useMemo(() => {
    const map: Record<EncodingMode, string> = {} as Record<
      EncodingMode,
      string
    >;
    for (const mode of MODES) {
      try {
        map[mode] = encodeOutput(mode, bytes);
      } catch {
        map[mode] = "—";
      }
    }
    return map;
  }, [bytes, encodeOutput]);

  const handleCopy = useCallback(
    (mode: EncodingMode) => {
      navigator.clipboard.writeText(results[mode]);
      toast.success(t("decoder.copied"));
    },
    [results, t],
  );

  return (
    <div className="flex flex-col h-full">
      {/* Input */}
      <div className="p-2 border-b border-border">
        <div className="flex items-center gap-2 mb-1.5">
          <div className="flex rounded border border-border overflow-hidden text-xs">
            {MODES.map((m) => (
              <button
                key={m}
                className={`px-1.5 py-0.5 capitalize ${
                  inputMode === m
                    ? "bg-accent/20 text-accent"
                    : "text-text-muted hover:bg-surface"
                }`}
                onClick={() => setInputMode(m)}
              >
                {m === "hex" ? "HEX" : m}
              </button>
            ))}
          </div>
        </div>
        <textarea
          className="w-full h-16 resize-none rounded border border-border bg-transparent p-1.5 font-mono text-xs"
          placeholder={t("decoder.inputPlaceholder")}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          spellCheck={false}
        />
        <div className="text-[10px] text-text-muted mt-0.5">
          {bytes.length} bytes
        </div>
      </div>

      {/* Output */}
      <div className="flex-1 overflow-y-auto p-2 space-y-2">
        {MODES.map((mode) => (
          <div key={mode} className="group">
            <div className="flex items-center justify-between mb-0.5">
              <span className="text-[10px] text-text-muted uppercase font-medium">
                {mode === "hex" ? "HEX" : mode}
              </span>
              <button
                className="text-[9px] text-text-muted opacity-0 group-hover:opacity-100 hover:text-text transition-opacity"
                onClick={() => handleCopy(mode)}
              >
                {t("common.copy")}
              </button>
            </div>
            <div className="text-xs font-mono break-all p-1.5 rounded bg-surface/50 text-text">
              {results[mode]}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
