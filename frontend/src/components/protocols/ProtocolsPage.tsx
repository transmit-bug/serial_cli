import MonacoEditor from "@monaco-editor/react";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { tauriApi } from "@/lib/tauri-api";
import { useProtocolStore } from "@/stores/protocol";

export function ProtocolsPage() {
  const { t } = useTranslation();
  const {
    protocols,
    loading,
    loadProtocols,
    loadCustomProtocol,
    unloadProtocol,
    reloadProtocol,
  } = useProtocolStore();

  const [editorContent, setEditorContent] = useState(
    "-- Lua protocol definition\nfunction encode(data)\n  return data\nend\n\nfunction parse(data)\n  return data\nend\n",
  );
  const [editorName, setEditorName] = useState("my_protocol");

  useEffect(() => {
    loadProtocols();
  }, [loadProtocols]);

  const handleImportFile = useCallback(async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        filters: [{ name: "Lua", extensions: ["lua"] }],
        multiple: false,
      });
      if (selected) {
        await loadCustomProtocol(selected);
        toast.success("Protocol loaded");
        await loadProtocols();
      }
    } catch (e) {
      toast.error(String(e));
    }
  }, [loadCustomProtocol, loadProtocols]);

  const handleSaveAndLoad = useCallback(async () => {
    try {
      const path = await tauriApi.saveProtocolFile(editorName, editorContent);
      await loadCustomProtocol(path);
      toast.success("Protocol saved and loaded");
      await loadProtocols();
    } catch (e) {
      toast.error(String(e));
    }
  }, [editorName, editorContent, loadCustomProtocol, loadProtocols]);

  const handleReload = useCallback(
    async (name: string) => {
      try {
        await reloadProtocol(name);
        toast.success("Protocol reloaded");
      } catch (e) {
        toast.error(String(e));
      }
    },
    [reloadProtocol],
  );

  const handleUnload = useCallback(
    async (name: string) => {
      try {
        await unloadProtocol(name);
        toast.success("Protocol unloaded");
      } catch (e) {
        toast.error(String(e));
      }
    },
    [unloadProtocol],
  );

  // Separate built-in from custom (built-in are: Modbus RTU, Modbus ASCII, AT Commands, Line)
  const builtIn = protocols.filter((p) =>
    ["ModbusRTU", "Modbus ASCII", "AT Commands", "Line"].includes(p.name),
  );
  const custom = protocols.filter(
    (p) => !builtIn.some((b) => b.name === p.name),
  );

  return (
    <div className="flex flex-col h-full p-4 overflow-y-auto">
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-lg font-semibold">{t("protocols.title")}</h1>
        <div className="flex gap-2">
          <button
            onClick={handleImportFile}
            className="px-3 py-1.5 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30"
          >
            {t("protocols.importFile")}
          </button>
        </div>
      </div>

      {loading && (
        <div className="text-text-muted text-sm">{t("common.loading")}</div>
      )}

      {/* Built-in protocols */}
      {builtIn.length > 0 && (
        <section className="mb-6">
          <h2 className="text-sm font-medium text-text-secondary mb-2">
            {t("protocols.builtIn")}
          </h2>
          <div className="space-y-1">
            {builtIn.map((p) => (
              <div
                key={p.name}
                className="flex items-center gap-3 px-3 py-2 bg-base-deep rounded border border-border"
              >
                <span className="w-2 h-2 rounded-full bg-accent" />
                <span className="text-sm font-medium">{p.name}</span>
                <span className="text-xs text-text-muted">{p.description}</span>
              </div>
            ))}
          </div>
        </section>
      )}

      {/* Custom protocols */}
      {custom.length > 0 && (
        <section className="mb-6">
          <h2 className="text-sm font-medium text-text-secondary mb-2">
            {t("protocols.custom")}
          </h2>
          <div className="space-y-1">
            {custom.map((p) => (
              <div
                key={p.name}
                className="flex items-center gap-3 px-3 py-2 bg-base-deep rounded border border-border"
              >
                <span className="w-2 h-2 rounded-full bg-success" />
                <span className="text-sm font-medium flex-1">{p.name}</span>
                <span className="text-xs text-text-muted">{p.description}</span>
                <button
                  onClick={() => handleReload(p.name)}
                  className="px-2 py-0.5 rounded text-xs text-text-muted hover:text-text"
                >
                  {t("protocols.reload")}
                </button>
                <button
                  onClick={() => handleUnload(p.name)}
                  className="px-2 py-0.5 rounded text-xs text-danger hover:bg-danger/20"
                >
                  {t("protocols.unload")}
                </button>
              </div>
            ))}
          </div>
        </section>
      )}

      {/* Protocol Editor */}
      <section>
        <h2 className="text-sm font-medium text-text-secondary mb-2">
          {t("protocols.protocolEditor")}
        </h2>
        <div className="mb-2">
          <input
            value={editorName}
            onChange={(e) => setEditorName(e.target.value)}
            placeholder="Protocol name"
            className="w-48 h-7 text-xs"
          />
        </div>
        <div
          className="border border-border rounded overflow-hidden"
          style={{ height: 300 }}
        >
          <MonacoEditor
            height="100%"
            language="lua"
            theme="vs-dark"
            value={editorContent}
            onChange={(v) => v !== undefined && setEditorContent(v)}
            options={{
              fontSize: 13,
              fontFamily: '"JetBrains Mono", "Fira Code", monospace',
              minimap: { enabled: false },
              scrollBeyondLastLine: false,
              lineNumbers: "on",
              tabSize: 2,
              padding: { top: 8 },
            }}
          />
        </div>
        <div className="flex gap-2 mt-2">
          <button
            onClick={handleSaveAndLoad}
            className="px-3 py-1.5 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30"
          >
            {t("protocols.saveAndLoad")}
          </button>
        </div>
      </section>
    </div>
  );
}
