import {
  AlertCircle,
  CheckCircle,
  Edit3,
  Loader2,
  Play,
  Square,
  Trash2,
} from "lucide-react";
import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { useCommandStore } from "@/stores/commands";
import { useConnectionStore } from "@/stores/connection";
import type { CommandSequence } from "@/types";
import { SequenceEditor } from "./SequenceEditor";

export function SequenceList() {
  const { t } = useTranslation();
  const portId = useConnectionStore((s) => s.activePortId);
  const isConnected = useConnectionStore(
    (s) =>
      s.activePortId != null &&
      s.connections.some(
        (c) => c.portId === s.activePortId && c.status === "connected",
      ),
  );
  const sequences = useCommandStore((s) => s.sequences);
  const executionState = useCommandStore((s) => s.executionState);
  const executeSequence = useCommandStore((s) => s.executeSequence);
  const stopSequence = useCommandStore((s) => s.stopSequence);
  const deleteSequence = useCommandStore((s) => s.deleteSequence);

  const [editingSequence, setEditingSequence] =
    useState<CommandSequence | null>(null);
  const [showEditor, setShowEditor] = useState(false);

  const handleExecute = useCallback(
    async (id: string) => {
      if (!portId) return;
      await executeSequence(id, portId);
    },
    [portId, executeSequence],
  );

  const isExecutingThis = executionState.sequenceId;

  if (showEditor) {
    return (
      <SequenceEditor
        sequence={editingSequence}
        onSave={() => {
          setShowEditor(false);
          setEditingSequence(null);
        }}
        onCancel={() => {
          setShowEditor(false);
          setEditingSequence(null);
        }}
      />
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-2 py-1.5 border-b border-border">
        <span className="text-xs font-medium text-text-muted">
          {t("sequences.title")}
        </span>
        <button
          className="px-2 py-0.5 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30"
          onClick={() => {
            setEditingSequence(null);
            setShowEditor(true);
          }}
        >
          + {t("sequences.newSequence")}
        </button>
      </div>

      <div className="flex-1 overflow-y-auto">
        {sequences.length === 0 ? (
          <div className="p-4 text-center text-xs text-text-muted">
            {t("sequences.emptyState")}
          </div>
        ) : (
          <div className="divide-y divide-border/50">
            {sequences.map((seq) => {
              const isActive = isExecutingThis === seq.id;
              const execStatus = isActive ? executionState.status : null;

              return (
                <div
                  key={seq.id}
                  className="group flex items-center gap-2 px-2 py-2 hover:bg-surface/30"
                >
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-1.5">
                      <span className="text-xs font-medium truncate">
                        {seq.name}
                      </span>
                      {isActive && execStatus === "running" && (
                        <Loader2
                          size={12}
                          className="animate-spin text-accent"
                        />
                      )}
                      {isActive && execStatus === "completed" && (
                        <CheckCircle size={12} className="text-green-500" />
                      )}
                      {isActive && execStatus === "error" && (
                        <AlertCircle size={12} className="text-danger" />
                      )}
                    </div>
                    <div className="text-[10px] text-text-muted">
                      {seq.steps.length} {t("sequences.steps")}
                    </div>
                    {isActive && execStatus === "running" && (
                      <div className="text-[10px] text-accent">
                        {t("sequences.executingStep", {
                          current: executionState.stepIndex + 1,
                          total: seq.steps.length,
                        })}
                        {executionState.loopIteration >= 0 &&
                          ` (${executionState.loopIteration + 1})`}
                      </div>
                    )}
                    {isActive &&
                      execStatus === "error" &&
                      executionState.error && (
                        <div className="text-[10px] text-danger truncate">
                          {executionState.error}
                        </div>
                      )}
                  </div>

                  <div className="hidden group-hover:flex items-center gap-0.5 shrink-0">
                    <button
                      className="text-[10px] text-text-muted hover:text-text px-1"
                      onClick={() => {
                        setEditingSequence(seq);
                        setShowEditor(true);
                      }}
                    >
                      <Edit3 size={12} />
                    </button>
                    <button
                      className="text-[10px] text-text-muted hover:text-danger px-1"
                      onClick={() => deleteSequence(seq.id)}
                    >
                      <Trash2 size={12} />
                    </button>
                  </div>

                  {isActive && execStatus === "running" ? (
                    <button
                      className="px-2 py-0.5 rounded text-xs bg-danger/20 text-danger hover:bg-danger/30"
                      onClick={stopSequence}
                    >
                      <Square size={10} />
                    </button>
                  ) : (
                    <button
                      className="px-2 py-0.5 rounded text-xs bg-accent/20 text-accent hover:bg-accent/30 disabled:opacity-40"
                      disabled={!isConnected}
                      onClick={() => handleExecute(seq.id)}
                    >
                      <Play size={10} />
                    </button>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
