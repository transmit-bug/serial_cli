import MonacoEditor from "@monaco-editor/react";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Group, Panel, Separator } from "react-resizable-panels";
import { toast } from "sonner";
import { tauriApi } from "@/lib/tauri-api";
import { hexToBytes } from "@/lib/utils";
import { useConnectionStore } from "@/stores/connection";
import { useProtocolStore } from "@/stores/protocol";
import { useScriptStore } from "@/stores/script";
import { useStandaloneScriptStore } from "@/stores/serialScript";
import { ProtocolList } from "./ProtocolList";
import { StandaloneActions } from "./StandaloneActions";
import { TemplateList } from "./TemplateList";

const BUILT_IN_PROTOCOLS = ["ModbusRTU", "Modbus ASCII", "AT Commands", "Line"];

export type FileType = "script" | "protocol";

export function EditorPage() {
  const { t } = useTranslation();
  const activeEntry = useConnectionStore((s) =>
    s.connections.find((c) => c.portId === s.activePortId),
  );
  const { availablePorts } = useConnectionStore();
  const isConnected = activeEntry?.status === "connected";
  const {
    scripts,
    currentScript,
    isDirty,
    output,
    loadScriptList,
    openScript,
    saveScript,
    deleteScript,
    executeScript,
    validateScript,
    newScript,
    updateContent,
    clearOutput,
  } = useScriptStore();
  const {
    actions: standaloneActions,
    loadActions: loadStandaloneActions,
    callAction: callStandaloneAction,
  } = useStandaloneScriptStore();
  const {
    protocols,
    loading: protocolsLoading,
    loadProtocols,
    loadCustomProtocol,
    unloadProtocol,
    reloadProtocol,
  } = useProtocolStore();

  const [fileType, setFileType] = useState<FileType | null>(null);
  const [scriptNameInput, setScriptNameInput] = useState("");
  const [protocolNameInput, setProtocolNameInput] = useState("");
  const [attachedPort, setAttachedPort] = useState<string | null>(null);
  const [attachDropdown, setAttachDropdown] = useState(false);

  // Protocol tester state
  const [testerProtocol, setTesterProtocol] = useState("");
  const [testerMode, setTesterMode] = useState<"encode" | "decode">("encode");
  const [testerInput, setTesterInput] = useState("");
  const [testerResult, setTesterResult] = useState<string | null>(null);
  const [testerError, setTesterError] = useState<string | null>(null);

  const [isDragging, setIsDragging] = useState(false);

  // ─── Drag & Drop ───

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.currentTarget.contains(e.relatedTarget as Node)) return;
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback(
    async (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragging(false);

      const file = e.dataTransfer.files[0];
      if (!file) return;

      const name = file.name;
      const content = await file.text();
      const baseName = name.replace(/\.(lua|json)$/, "");

      if (name.endsWith(".lua")) {
        newScript();
        setScriptNameInput(baseName);
        updateContent(content);
        setFileType("script");
        toast.success(t("editor.imported"));
      } else if (name.endsWith(".json")) {
        setProtocolNameInput(baseName);
        updateContent(content);
        setFileType("protocol");
        toast.success(t("editor.imported"));
      } else {
        toast.error(t("editor.dropUnsupported"));
      }
    },
    [newScript, updateContent, t],
  );

  useEffect(() => {
    loadScriptList();
    loadProtocols();
  }, [loadScriptList, loadProtocols]);

  const customProtocols = protocols.filter(
    (p) => !BUILT_IN_PROTOCOLS.includes(p.name),
  );

  // ─── Create / Open ───

  const handleNew = useCallback(
    (type: FileType = "script") => {
      if (type === "script") {
        newScript();
        setScriptNameInput("");
      } else {
        setProtocolNameInput("my_protocol");
        updateContent(
          "-- Lua protocol definition\nfunction on_frame(data)\n  return data\nend\n\nfunction on_encode(data)\n  return data\nend\n",
        );
      }
      setFileType(type);
    },
    [newScript, updateContent],
  );

  const handleLoadTemplate = useCallback(
    (content: string, type: FileType) => {
      handleNew(type);
      updateContent(content);
    },
    [handleNew, updateContent],
  );

  const handleOpenScript = useCallback(
    async (name: string) => {
      await openScript(name);
      setFileType("script");
    },
    [openScript],
  );

  const handleOpenProtocol = useCallback(
    async (name: string) => {
      const proto = protocols.find((p) => p.name === name);
      if (!proto) return;
      setProtocolNameInput(name);
      setFileType("protocol");
      updateContent(
        "function on_frame(data)\n  return data\nend\n\nfunction on_encode(data)\n  return data\nend\n",
      );
    },
    [protocols, updateContent],
  );

  // Auto-open first script when list loads and no file is open
  const autoOpenedRef = useRef(false);
  useEffect(() => {
    if (autoOpenedRef.current) return;
    if (scripts.length === 0 || currentScript) return;
    autoOpenedRef.current = true;
    openScript(scripts[0].name);
    setFileType("script");
  }, [scripts, currentScript, openScript]);

  // ─── Actions ───

  const handleSave = useCallback(async () => {
    if (fileType === "script" && currentScript) {
      const name = currentScript.name || scriptNameInput;
      if (!name.trim()) {
        toast.error(t("scripts.saveError"));
        return;
      }
      try {
        await saveScript(name, currentScript.content);
        toast.success(t("scripts.saveSuccess"));
      } catch (e) {
        toast.error(String(e));
      }
    } else if (fileType === "protocol") {
      const name = protocolNameInput;
      if (!name.trim()) {
        toast.error(t("protocols.saveError"));
        return;
      }
      try {
        const path = await tauriApi.saveProtocolFile(
          name,
          currentScript?.content ?? "",
        );
        await loadCustomProtocol(path);
        await tauriApi.validateProtocol(path);
        toast.success(t("protocols.saveSuccess"));
        await loadProtocols();
      } catch (e) {
        toast.error(String(e));
      }
    }
  }, [
    fileType,
    currentScript,
    scriptNameInput,
    protocolNameInput,
    saveScript,
    loadCustomProtocol,
    loadProtocols,
    t,
  ]);

  const handleValidate = useCallback(async () => {
    if (!currentScript) return;
    try {
      if (fileType === "script") {
        const errors = await validateScript(currentScript.content);
        if (errors.length === 0) {
          toast.success(t("scripts.validateSuccess"));
        } else {
          errors.forEach((e) => toast.error(`Line ${e.line}: ${e.message}`));
        }
      } else {
        const path = await tauriApi.saveProtocolFile(
          protocolNameInput,
          currentScript.content,
        );
        await tauriApi.validateProtocol(path);
        toast.success(t("scripts.validateSuccess"));
      }
    } catch (e) {
      toast.error(String(e));
    }
  }, [currentScript, fileType, protocolNameInput, validateScript, t]);

  const handleRun = useCallback(async () => {
    if (!currentScript) return;
    try {
      await executeScript(currentScript.content);
      toast.success(t("scripts.executeSuccess"));
      // Load standalone UI actions for this script
      await loadStandaloneActions(currentScript.content);
    } catch (e) {
      toast.error(String(e));
    }
  }, [currentScript, executeScript, loadStandaloneActions, t]);

  const handleDelete = useCallback(
    async (name: string) => {
      try {
        if (fileType === "script") {
          await deleteScript(name);
        } else {
          await unloadProtocol(name);
        }
        toast.success(t("scripts.deleteSuccess"));
        setFileType(null);
      } catch (e) {
        toast.error(String(e));
      }
    },
    [fileType, deleteScript, unloadProtocol, t],
  );

  const handleAttachToPort = useCallback(
    async (targetPortId: string) => {
      if (!currentScript) return;
      try {
        await executeScript(currentScript.content);
        setAttachedPort(targetPortId);
        toast.success(t("scripts.attached"));
      } catch (e) {
        toast.error(String(e));
      }
      setAttachDropdown(false);
    },
    [currentScript, executeScript, t],
  );

  const handleDetach = useCallback(() => {
    setAttachedPort(null);
    toast.success(t("scripts.detached"));
  }, [t]);

  // Protocol tester
  const handleTesterRun = useCallback(async () => {
    if (!testerProtocol) {
      toast.error(t("protocolTester.noProtocol"));
      return;
    }
    if (!testerInput.trim()) {
      toast.error(t("protocolTester.noInput"));
      return;
    }

    setTesterError(null);
    setTesterResult(null);

    try {
      const bytes = hexToBytes(testerInput);
      if (testerMode === "encode") {
        const encoded = await tauriApi.protocolEncode(testerProtocol, bytes);
        setTesterResult(JSON.stringify(encoded, null, 2));
      } else {
        const decoded = await tauriApi.protocolDecode(testerProtocol, bytes);
        setTesterResult(JSON.stringify(decoded, null, 2));
      }
    } catch (e) {
      setTesterError(String(e));
    }
  }, [testerProtocol, testerMode, testerInput, t]);

  const canDelete =
    (fileType === "script" && currentScript?.name) ||
    (fileType === "protocol" && protocolNameInput);

  // ─── Main layout (always visible — editor area shows empty state when no file) ───

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="flex items-center gap-2 px-3 py-2 border-b border-border shrink-0">
        <h1 className="text-sm font-semibold mr-4">{t("editor.title")}</h1>

        <button
          onClick={() => handleNew("script")}
          className="px-2 py-1 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30"
        >
          + {t("common.new")}
        </button>

        <div className="mx-2 w-px h-5 bg-border" />

        <button
          onClick={handleValidate}
          disabled={!currentScript}
          className="px-2 py-1 rounded text-xs text-text-muted hover:text-text disabled:opacity-50"
        >
          ✓ {t("common.validate")}
        </button>
        <button
          onClick={handleRun}
          disabled={!currentScript}
          className="px-2 py-1 rounded text-xs text-text-muted hover:text-text disabled:opacity-50"
        >
          ▶ {t("common.run")}
        </button>
        <button
          onClick={handleSave}
          disabled={!currentScript && fileType !== "protocol"}
          className="px-2 py-1 rounded text-xs text-text-muted hover:text-text disabled:opacity-50"
        >
          💾 {t("common.save")}
        </button>

        {/* Attach to Port */}
        {fileType === "script" && (
          <div className="relative">
            {attachedPort ? (
              <div className="flex items-center gap-1">
                <span className="text-xs text-success">
                  ● {t("scripts.attached")}
                </span>
                <button
                  onClick={handleDetach}
                  className="text-xs text-danger hover:bg-danger/20 px-1 rounded"
                >
                  {t("common.cancel")}
                </button>
              </div>
            ) : (
              <button
                onClick={() => setAttachDropdown(!attachDropdown)}
                disabled={!currentScript || !isConnected}
                className="px-2 py-1 rounded text-xs text-text-muted hover:text-text disabled:opacity-50"
              >
                {t("scripts.attachToPort")}
              </button>
            )}
            {attachDropdown && (
              <div className="absolute top-8 left-0 z-50 rounded border border-border bg-base shadow-lg py-1 min-w-40">
                {availablePorts
                  .filter((p) => !p.is_virtual)
                  .map((p) => (
                    <button
                      key={p.port_name}
                      className="w-full text-left px-3 py-1 text-xs hover:bg-surface"
                      onClick={() => handleAttachToPort(p.port_name)}
                    >
                      {p.port_name}
                    </button>
                  ))}
              </div>
            )}
          </div>
        )}

        {canDelete && (
          <button
            onClick={() =>
              handleDelete(
                fileType === "script"
                  ? (currentScript?.name ?? "")
                  : protocolNameInput,
              )
            }
            className="px-2 py-1 rounded text-xs text-danger hover:bg-danger/20"
          >
            {t("common.delete")}
          </button>
        )}

        {isDirty && (
          <span className="text-warning text-xs ml-2">
            {t("scripts.unsavedChanges")}
          </span>
        )}
      </div>

      <Group orientation="horizontal" className="flex-1">
        {/* Left panel: file list + templates */}
        <Panel defaultSize="25%" minSize="15%" maxSize="45%">
          <div className="h-full overflow-y-auto">
            {/* Protocol list */}
            <ProtocolList
              protocols={protocols}
              customProtocols={customProtocols}
              loading={protocolsLoading}
              onOpenProtocol={handleOpenProtocol}
              onReloadProtocol={async (name) => {
                try {
                  await reloadProtocol(name);
                  toast.success(t("protocols.reload"));
                } catch (e) {
                  toast.error(String(e));
                }
              }}
              onUnloadProtocol={async (name) => {
                try {
                  await unloadProtocol(name);
                  toast.success(t("protocols.deleteSuccess"));
                  if (protocolNameInput === name) setFileType(null);
                } catch (e) {
                  toast.error(String(e));
                }
              }}
              onImportProtocol={async () => {
                try {
                  const { open } = await import("@tauri-apps/plugin-dialog");
                  const selected = await open({
                    filters: [{ name: "Lua", extensions: ["lua"] }],
                    multiple: false,
                  });
                  if (selected) {
                    await loadCustomProtocol(selected);
                    toast.success(t("protocols.importSuccess"));
                    await loadProtocols();
                  }
                } catch (e) {
                  toast.error(String(e));
                }
              }}
            />

            {/* Script list */}
            <div className="px-3 py-1.5">
              <span className="text-[10px] uppercase tracking-wider text-text-muted">
                {t("scripts.title")}
              </span>
            </div>
            {scripts.length === 0 ? (
              <div className="px-3 py-2 text-text-muted text-xs">
                {t("scripts.noScripts")}
              </div>
            ) : (
              scripts.map((s) => (
                <button
                  key={s.name}
                  onClick={() => handleOpenScript(s.name)}
                  className={`w-full text-left px-3 py-1.5 text-xs hover:bg-surface/50 border-b border-border/30 ${
                    fileType === "script" && currentScript?.name === s.name
                      ? "bg-accent/10 text-accent"
                      : ""
                  }`}
                >
                  {s.name}.lua
                </button>
              ))
            )}

            {/* Unified templates */}
            <TemplateList onLoadTemplate={handleLoadTemplate} />
          </div>
        </Panel>

        <Separator className="w-1.5 bg-border hover:bg-accent cursor-col-resize transition-colors" />

        {/* Editor + Bottom panel */}
        <Panel defaultSize="75%" minSize="35%">
          <Group orientation="vertical">
            {/* Name input for new file */}
            {fileType === "script" && !currentScript?.name && currentScript && (
              <div className="px-3 py-1.5 border-b border-border">
                <input
                  value={scriptNameInput}
                  onChange={(e) => setScriptNameInput(e.target.value)}
                  placeholder={t("scripts.scriptName")}
                  className="w-48 h-6 text-xs"
                />
              </div>
            )}
            {fileType === "protocol" && (
              <div className="px-3 py-1.5 border-b border-border flex items-center gap-2">
                <input
                  value={protocolNameInput}
                  onChange={(e) => setProtocolNameInput(e.target.value)}
                  placeholder={t("protocols.protocolName")}
                  className="w-48 h-6 text-xs"
                />
                <button
                  onClick={handleSave}
                  className="px-3 py-1 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30"
                >
                  {t("protocols.saveAndLoad")}
                </button>
              </div>
            )}

            {/* Monaco Editor / Welcome placeholder */}
            <Panel defaultSize={65} minSize={30}>
              {/* biome-ignore lint/a11y/noStaticElementInteractions: drop zone for file drag-and-drop */}
              <section
                className="relative h-full"
                onDragOver={handleDragOver}
                onDragEnter={handleDragEnter}
                onDragLeave={handleDragLeave}
                onDrop={handleDrop}
              >
                {isDragging && (
                  <div className="absolute inset-0 z-50 flex flex-col items-center justify-center bg-base/80 border-2 border-dashed border-accent rounded-lg gap-2">
                    <span className="text-2xl">📄</span>
                    <div className="text-accent text-sm">
                      {t("editor.dropZone")}
                    </div>
                  </div>
                )}
                {!fileType ? (
                  <div className="flex flex-col items-center justify-center h-full text-center space-y-4">
                    <div className="text-text-muted text-sm">
                      {t("editor.emptyState")}
                    </div>
                    <button
                      onClick={() => handleNew("script")}
                      className="px-4 py-2 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30"
                    >
                      + {t("common.new")}
                    </button>
                    <div className="text-xs text-text-muted">
                      {t("editor.templateTip")}
                    </div>
                    <div className="text-[10px] text-text-muted/60">
                      {t("editor.dropZone")}
                    </div>
                  </div>
                ) : (
                  <MonacoEditor
                    height="100%"
                    language="lua"
                    theme="vs-dark"
                    value={currentScript?.content ?? ""}
                    onChange={(value) =>
                      value !== undefined && updateContent(value)
                    }
                    options={{
                      fontSize: 13,
                      fontFamily: '"JetBrains Mono", "Fira Code", monospace',
                      minimap: { enabled: false },
                      scrollBeyondLastLine: false,
                      lineNumbers: "on",
                      renderWhitespace: "selection",
                      tabSize: 2,
                      wordWrap: "on",
                      padding: { top: 8 },
                    }}
                  />
                )}
              </section>
            </Panel>

            <Separator className="h-1 bg-border hover:bg-accent cursor-row-resize transition-colors" />

            {/* Bottom panel: output console or protocol tester */}
            <Panel defaultSize={35} minSize={10}>
              {!fileType ? (
                <div className="flex items-center justify-center h-full text-xs text-text-muted">
                  {t("editor.selectOrCreateFile")}
                </div>
              ) : fileType === "script" ? (
                <div className="flex flex-col h-full">
                  <div className="flex items-center justify-between px-3 py-1 border-b border-border">
                    <span className="text-xs text-text-secondary">
                      {t("scripts.output")}
                    </span>
                    <button
                      onClick={clearOutput}
                      className="text-xs text-text-muted hover:text-text"
                    >
                      {t("common.clear")}
                    </button>
                  </div>
                  <div className="flex-1 overflow-y-auto p-2 font-mono text-xs">
                    {output.map((line, i) => (
                      <div
                        key={i}
                        className={`py-0.5 ${
                          line.type === "error"
                            ? "text-danger"
                            : line.type === "success"
                              ? "text-green-500"
                              : "text-text"
                        }`}
                      >
                        {line.text}
                      </div>
                    ))}
                  </div>
                  {standaloneActions.length > 0 && (
                    <StandaloneActions
                      actions={standaloneActions}
                      onCall={async (fn) => {
                        try {
                          const result = await callStandaloneAction(fn);
                          toast.success(result);
                        } catch (e) {
                          toast.error(String(e));
                        }
                      }}
                    />
                  )}
                </div>
              ) : (
                <ProtocolTesterPanel
                  protocols={customProtocols}
                  selectedProtocol={testerProtocol}
                  onSelectProtocol={setTesterProtocol}
                  mode={testerMode}
                  onModeChange={setTesterMode}
                  input={testerInput}
                  onInputChange={setTesterInput}
                  onRun={handleTesterRun}
                  result={testerResult}
                  error={testerError}
                />
              )}
            </Panel>
          </Group>
        </Panel>
      </Group>
    </div>
  );
}

/* ─── Protocol Tester Panel ─── */

function ProtocolTesterPanel({
  protocols,
  selectedProtocol,
  onSelectProtocol,
  mode,
  onModeChange,
  input,
  onInputChange,
  onRun,
  result,
  error,
}: {
  protocols: { name: string; description: string }[];
  selectedProtocol: string;
  onSelectProtocol: (name: string) => void;
  mode: "encode" | "decode";
  onModeChange: (mode: "encode" | "decode") => void;
  input: string;
  onInputChange: (val: string) => void;
  onRun: () => void;
  result: string | null;
  error: string | null;
}) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col h-full p-3 gap-3">
      <div className="flex items-center gap-2">
        <select
          className="flex-1 h-7 text-xs rounded border border-border bg-transparent px-2"
          value={selectedProtocol}
          onChange={(e) => onSelectProtocol(e.target.value)}
        >
          <option value="">{t("protocolTester.selectProtocol")}</option>
          {protocols.map((p) => (
            <option key={p.name} value={p.name}>
              {p.name}
            </option>
          ))}
        </select>

        <div className="flex rounded border border-border overflow-hidden text-xs">
          <button
            className={`px-3 py-1 ${mode === "encode" ? "bg-accent/20 text-accent" : "text-text-muted hover:bg-surface"}`}
            onClick={() => onModeChange("encode")}
          >
            {t("protocolTester.encode")}
          </button>
          <button
            className={`px-3 py-1 ${mode === "decode" ? "bg-accent/20 text-accent" : "text-text-muted hover:bg-surface"}`}
            onClick={() => onModeChange("decode")}
          >
            {t("protocolTester.decode")}
          </button>
        </div>
      </div>

      <div className="flex flex-col gap-1">
        <label className="text-[10px] text-text-muted uppercase">
          {t("protocolTester.input")}
        </label>
        <textarea
          className="h-20 resize-none rounded border border-border bg-transparent p-2 font-mono text-xs"
          placeholder={mode === "encode" ? "01 03 00 00 00 01" : "raw bytes"}
          value={input}
          onChange={(e) => onInputChange(e.target.value)}
        />
      </div>

      <button
        className="h-7 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30 disabled:opacity-50"
        disabled={!selectedProtocol || !input.trim()}
        onClick={onRun}
      >
        {t("common.run")} ▶
      </button>

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
