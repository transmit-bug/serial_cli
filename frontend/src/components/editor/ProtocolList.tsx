import { useTranslation } from "react-i18next";
import type { ProtocolInfo } from "@/types";

const BUILT_IN_PROTOCOLS = ["ModbusRTU", "Modbus ASCII", "AT Commands", "Line"];

interface ProtocolListProps {
  protocols: ProtocolInfo[];
  customProtocols: ProtocolInfo[];
  loading: boolean;
  onOpenProtocol: (name: string) => void;
  onReloadProtocol: (name: string) => void;
  onUnloadProtocol: (name: string) => void;
  onImportProtocol: () => void;
}

export function ProtocolList({
  protocols,
  customProtocols,
  loading,
  onOpenProtocol,
  onReloadProtocol,
  onUnloadProtocol,
  onImportProtocol,
}: ProtocolListProps) {
  const { t } = useTranslation();
  const builtIn = protocols.filter((p) => BUILT_IN_PROTOCOLS.includes(p.name));

  return (
    <div>
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-1.5">
        <span className="text-[10px] uppercase tracking-wider text-text-muted">
          {t("protocols.title")}
        </span>
        <button
          onClick={onImportProtocol}
          className="text-[10px] text-text-muted hover:text-accent transition"
        >
          {t("protocols.importFile")}
        </button>
      </div>

      {loading && (
        <div className="px-3 py-1 text-text-muted text-xs">
          {t("common.loading")}
        </div>
      )}

      {/* Built-in protocols */}
      {builtIn.length > 0 && (
        <div className="px-3 py-1 space-y-0.5">
          {builtIn.map((p) => (
            <div
              key={p.name}
              className="flex items-center gap-2 py-0.5 text-xs text-text-muted"
            >
              <span className="w-1.5 h-1.5 rounded-full bg-accent shrink-0" />
              <span className="truncate">{p.name}</span>
            </div>
          ))}
        </div>
      )}

      {/* Custom protocols */}
      {customProtocols.length > 0 ? (
        <div className="px-3 py-1 space-y-0.5">
          {customProtocols.map((p) => (
            <div key={p.name} className="flex items-center gap-1.5 group">
              <span className="w-1.5 h-1.5 rounded-full bg-success shrink-0" />
              <button
                onClick={() => onOpenProtocol(p.name)}
                className="flex-1 text-left text-xs truncate hover:text-accent transition"
              >
                {p.name}
              </button>
              <button
                onClick={() => onReloadProtocol(p.name)}
                className="text-[10px] text-text-muted hover:text-text opacity-0 group-hover:opacity-100 transition"
              >
                ↻
              </button>
              <button
                onClick={() => onUnloadProtocol(p.name)}
                className="text-[10px] text-danger hover:text-danger opacity-0 group-hover:opacity-100 transition"
              >
                ×
              </button>
            </div>
          ))}
        </div>
      ) : (
        <div className="px-3 py-1 text-text-muted text-[10px]">
          {t("protocols.noCustomProtocols")}
        </div>
      )}
    </div>
  );
}
