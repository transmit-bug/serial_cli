import { ArrowDown, ArrowUp, Plus, Trash2, X } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useCommandStore } from "@/stores/commands";
import type { CommandSequence, SequenceStep } from "@/types";

interface SequenceEditorProps {
  sequence?: CommandSequence | null;
  onSave: () => void;
  onCancel: () => void;
}

const emptyStep = (): SequenceStep => ({
  label: "",
  data: "",
  format: "ascii",
  delay: 0,
  waitTimeout: 5000,
  loopCount: 1,
});

export function SequenceEditor({
  sequence,
  onSave,
  onCancel,
}: SequenceEditorProps) {
  const { t } = useTranslation();
  const addSequence = useCommandStore((s) => s.addSequence);
  const updateSequence = useCommandStore((s) => s.updateSequence);

  const [name, setName] = useState(sequence?.name ?? "");
  const [steps, setSteps] = useState<SequenceStep[]>(
    sequence?.steps.length ? structuredClone(sequence.steps) : [emptyStep()],
  );
  const nameInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    nameInputRef.current?.focus();
  }, []);

  const updateStep = useCallback(
    (idx: number, patch: Partial<SequenceStep>) => {
      setSteps((prev) => {
        const next = [...prev];
        next[idx] = { ...next[idx], ...patch };
        return next;
      });
    },
    [],
  );

  const addStep = useCallback(() => {
    setSteps((prev) => [...prev, emptyStep()]);
  }, []);

  const removeStep = useCallback((idx: number) => {
    setSteps((prev) => prev.filter((_, i) => i !== idx));
  }, []);

  const moveStep = useCallback((from: number, to: number) => {
    setSteps((prev) => {
      if (to < 0 || to >= prev.length) return prev;
      const next = [...prev];
      const [item] = next.splice(from, 1);
      next.splice(to, 0, item);
      return next;
    });
  }, []);

  const handleSave = () => {
    if (!name.trim() || steps.length === 0) return;
    const cleaned = steps.filter((s) => s.data.trim());
    if (cleaned.length === 0) return;

    if (sequence) {
      updateSequence(sequence.id, {
        ...sequence,
        name: name.trim(),
        steps: cleaned,
      });
    } else {
      addSequence({
        id: crypto.randomUUID(),
        name: name.trim(),
        steps: cleaned,
      });
    }
    onSave();
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 p-2 border-b border-border">
        <input
          className="flex-1 h-7 text-xs rounded border border-border bg-transparent px-2 font-medium"
          placeholder={t("sequences.sequenceName")}
          value={name}
          onChange={(e) => setName(e.target.value)}
          ref={nameInputRef}
        />
        <button
          className="px-2 py-1 text-xs rounded bg-accent/20 text-accent hover:bg-accent/30"
          onClick={handleSave}
          disabled={!name.trim()}
        >
          {t("common.save")}
        </button>
        <button
          className="px-1 py-1 text-xs rounded text-text-muted hover:text-text"
          onClick={onCancel}
        >
          <X size={14} />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-2 space-y-2">
        {steps.map((step, idx) => (
          <div
            key={idx}
            className="rounded border border-border/60 bg-surface/30 p-2 space-y-1.5"
          >
            <div className="flex items-center gap-1">
              <span className="text-[10px] text-text-muted font-mono w-5">
                {idx + 1}
              </span>
              <input
                className="flex-1 h-6 text-xs rounded border border-border bg-transparent px-2"
                placeholder={t("sequences.stepLabel")}
                value={step.label}
                onChange={(e) => updateStep(idx, { label: e.target.value })}
              />
              <div className="flex flex-col">
                <button
                  className="text-[8px] text-text-muted leading-none hover:text-text"
                  onClick={() => moveStep(idx, idx - 1)}
                  disabled={idx === 0}
                >
                  <ArrowUp size={8} />
                </button>
                <button
                  className="text-[8px] text-text-muted leading-none hover:text-text"
                  onClick={() => moveStep(idx, idx + 1)}
                  disabled={idx === steps.length - 1}
                >
                  <ArrowDown size={8} />
                </button>
              </div>
              <button
                className="text-text-muted hover:text-danger px-1"
                onClick={() => removeStep(idx)}
                disabled={steps.length === 1}
              >
                <Trash2 size={12} />
              </button>
            </div>

            <div className="flex gap-1.5">
              <input
                className="flex-1 h-6 text-xs rounded border border-border bg-transparent px-2 font-mono"
                placeholder={t("sequences.stepData")}
                value={step.data}
                onChange={(e) => updateStep(idx, { data: e.target.value })}
              />
              <select
                className="h-6 text-xs rounded border border-border bg-transparent px-1"
                value={step.format}
                onChange={(e) =>
                  updateStep(idx, { format: e.target.value as "hex" | "ascii" })
                }
              >
                <option value="ascii">ASCII</option>
                <option value="hex">HEX</option>
              </select>
            </div>

            <div className="flex items-center gap-3 flex-wrap">
              <label className="flex items-center gap-1 text-[11px] text-text-muted">
                {t("sequences.delay")}
                <input
                  type="number"
                  className="w-16 h-5 text-xs rounded border border-border bg-transparent px-1"
                  value={step.delay}
                  min={0}
                  max={60000}
                  onChange={(e) =>
                    updateStep(idx, { delay: Number(e.target.value) || 0 })
                  }
                />
                ms
              </label>
              <label className="flex items-center gap-1 text-[11px] text-text-muted">
                {t("sequences.waitFor")}
                <input
                  className="w-20 h-5 text-xs rounded border border-border bg-transparent px-1 font-mono"
                  placeholder="RX string"
                  value={step.waitFor ?? ""}
                  onChange={(e) =>
                    updateStep(idx, {
                      waitFor: e.target.value || undefined,
                    })
                  }
                />
              </label>
              {step.waitFor && (
                <label className="flex items-center gap-1 text-[11px] text-text-muted">
                  {t("sequences.timeout")}
                  <input
                    type="number"
                    className="w-16 h-5 text-xs rounded border border-border bg-transparent px-1"
                    value={step.waitTimeout ?? 5000}
                    min={100}
                    max={60000}
                    step={500}
                    onChange={(e) =>
                      updateStep(idx, {
                        waitTimeout: Number(e.target.value) || 5000,
                      })
                    }
                  />
                  ms
                </label>
              )}
              <label className="flex items-center gap-1 text-[11px] text-text-muted">
                {t("sequences.loop")}
                <input
                  type="number"
                  className="w-12 h-5 text-xs rounded border border-border bg-transparent px-1"
                  value={step.loopCount ?? 1}
                  min={1}
                  max={9999}
                  onChange={(e) =>
                    updateStep(idx, {
                      loopCount: Math.max(1, Number(e.target.value) || 1),
                    })
                  }
                />
              </label>
            </div>
          </div>
        ))}
      </div>

      <div className="p-2 border-t border-border">
        <button
          className="w-full rounded px-2 py-1.5 text-xs text-text-muted hover:bg-surface/50 hover:text-text flex items-center justify-center gap-1"
          onClick={addStep}
        >
          <Plus size={12} />
          {t("sequences.addStep")}
        </button>
      </div>
    </div>
  );
}
