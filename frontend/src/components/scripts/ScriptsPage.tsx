import MonacoEditor from "@monaco-editor/react";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Group, Panel, Separator } from "react-resizable-panels";
import { toast } from "sonner";
import { useConnectionStore } from "@/stores/connection";
import { useScriptStore } from "@/stores/script";
import { ScriptTemplates } from "./ScriptTemplates";

export function ScriptsPage() {
  const { t } = useTranslation();
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

  const { status, availablePorts } = useConnectionStore();
  const [scriptNameInput, setScriptNameInput] = useState("");
  const [attachedPort, setAttachedPort] = useState<string | null>(null);
  const [attachDropdown, setAttachDropdown] = useState(false);

  useEffect(() => {
    loadScriptList();
  }, [loadScriptList]);

  const handleSave = useCallback(async () => {
    if (!currentScript) return;
    const name = currentScript.name || scriptNameInput;
    if (!name.trim()) {
      toast.error("Script name is required");
      return;
    }
    try {
      await saveScript(name, currentScript.content);
      toast.success(t("scripts.saveSuccess"));
    } catch (e) {
      toast.error(String(e));
    }
  }, [currentScript, scriptNameInput, saveScript, t]);

  const handleValidate = useCallback(async () => {
    if (!currentScript) return;
    try {
      const errors = await validateScript(currentScript.content);
      if (errors.length === 0) {
        toast.success(t("scripts.validateSuccess"));
      } else {
        errors.forEach((e) => toast.error(`Line ${e.line}: ${e.message}`));
      }
    } catch (e) {
      toast.error(String(e));
    }
  }, [currentScript, validateScript, t]);

  const handleRun = useCallback(async () => {
    if (!currentScript) return;
    try {
      await executeScript(currentScript.content);
      toast.success(t("scripts.executeSuccess"));
    } catch (e) {
      toast.error(String(e));
    }
  }, [currentScript, executeScript, t]);

  const handleDelete = useCallback(
    async (name: string) => {
      try {
        await deleteScript(name);
        toast.success(t("scripts.deleteSuccess"));
      } catch (e) {
        toast.error(String(e));
      }
    },
    [deleteScript, t],
  );

  const handleLoadTemplate = useCallback(
    (content: string) => {
      newScript();
      updateContent(content);
    },
    [newScript, updateContent],
  );

  const handleAttachToPort = useCallback(
    async (targetPortId: string) => {
      if (!currentScript) return;
      try {
        await executeScript(currentScript.content);
        setAttachedPort(targetPortId);
        toast.success(`Script attached to port`);
      } catch (e) {
        toast.error(String(e));
      }
      setAttachDropdown(false);
    },
    [currentScript, executeScript],
  );

  const handleDetach = useCallback(() => {
    setAttachedPort(null);
    toast.success("Script detached");
  }, []);

  // Empty state
  if (!currentScript) {
    return (
      <div className="flex flex-col h-full">
        <div className="flex items-center gap-2 px-3 py-2 border-b border-border shrink-0">
          <h1 className="text-sm font-semibold mr-4">{t("scripts.title")}</h1>
          <button
            onClick={newScript}
            className="px-2 py-1 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30"
          >
            + {t("scripts.newScript")}
          </button>
        </div>
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center space-y-4">
            <div className="text-text-muted text-sm">
              {t("scripts.emptyState")}
            </div>
            <button
              onClick={newScript}
              className="px-4 py-2 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30"
            >
              + {t("scripts.newScript")}
            </button>
            <div className="text-xs text-text-muted">
              {t("scripts.templateTip")}
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="flex items-center gap-2 px-3 py-2 border-b border-border shrink-0">
        <h1 className="text-sm font-semibold mr-4">{t("scripts.title")}</h1>
        <button
          onClick={() => {
            newScript();
            setScriptNameInput("");
          }}
          className="px-2 py-1 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30"
        >
          + {t("scripts.newScript")}
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
          disabled={!currentScript}
          className="px-2 py-1 rounded text-xs text-text-muted hover:text-text disabled:opacity-50"
        >
          💾 {t("common.save")}
        </button>

        {/* Attach to Port */}
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
              disabled={!currentScript || status !== "connected"}
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

        {currentScript?.name && (
          <button
            onClick={() => handleDelete(currentScript.name ?? "")}
            className="px-2 py-1 rounded text-xs text-danger hover:bg-danger/20"
          >
            🗑 {t("common.delete")}
          </button>
        )}
        {isDirty && (
          <span className="text-warning text-xs ml-2">
            {t("scripts.unsavedChanges")}
          </span>
        )}
      </div>

      <Group orientation="horizontal" className="flex-1">
        {/* Script list */}
        <Panel defaultSize={18} minSize={12} maxSize={25}>
          <div className="h-full overflow-y-auto">
            <ScriptTemplates onLoadTemplate={handleLoadTemplate} />

            {scripts.length === 0 ? (
              <div className="p-3 text-text-muted text-xs">
                {t("scripts.noScripts")}
              </div>
            ) : (
              scripts.map((s) => (
                <button
                  key={s.name}
                  onClick={() => openScript(s.name)}
                  className={`w-full text-left px-3 py-1.5 text-xs hover:bg-surface/50 border-b border-border/30 ${
                    currentScript?.name === s.name
                      ? "bg-accent/10 text-accent"
                      : ""
                  }`}
                >
                  {s.name}.lua
                </button>
              ))
            )}
          </div>
        </Panel>

        <Separator className="w-px bg-border" />

        {/* Editor + Output */}
        <Panel defaultSize={82}>
          <Group orientation="vertical">
            {/* Name input for new script */}
            {!currentScript?.name && currentScript && (
              <div className="px-3 py-1.5 border-b border-border">
                <input
                  value={scriptNameInput}
                  onChange={(e) => setScriptNameInput(e.target.value)}
                  placeholder={t("scripts.scriptName")}
                  className="w-48 h-6 text-xs"
                />
              </div>
            )}

            {/* Monaco Editor */}
            <Panel defaultSize={70} minSize={30}>
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
            </Panel>

            <Separator className="h-px bg-border" />

            {/* Output console */}
            <Panel defaultSize={30} minSize={10}>
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
                      className={`py-0.5 ${line.type === "error" ? "text-danger" : line.type === "success" ? "text-green-500" : "text-text"}`}
                    >
                      {line.text}
                    </div>
                  ))}
                </div>
              </div>
            </Panel>
          </Group>
        </Panel>
      </Group>
    </div>
  );
}
