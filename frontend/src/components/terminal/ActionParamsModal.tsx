import { useCallback, useMemo, useState } from "react";
import type { ActionParam } from "@/types";

interface ActionParamsModalProps {
  /** Display label of the action */
  label: string;
  /** Parameter descriptors */
  params: ActionParam[];
  /** Called with JSON-encoded args array on confirm */
  onConfirm: (argsJson: string) => void;
  /** Called on cancel */
  onCancel: () => void;
}

/**
 * Modal dialog for collecting action parameters.
 *
 * - Params with `default` are pre-filled and considered optional.
 * - Params without `default` are required (validated before confirm).
 * - Supports number, string, and hex input types.
 */
export function ActionParamsModal({
  label,
  params,
  onConfirm,
  onCancel,
}: ActionParamsModalProps) {
  // Initialize values from defaults
  const initialValues = useMemo(
    () =>
      params.map((p) => ({
        name: p.name,
        value: p.default ?? "",
        param: p,
      })),
    [params],
  );

  const [values, setValues] = useState(initialValues);
  const [errors, setErrors] = useState<Record<string, string>>({});

  const handleChange = useCallback(
    (index: number, newValue: string) => {
      setValues((prev) =>
        prev.map((v, i) => (i === index ? { ...v, value: newValue } : v)),
      );
      // Clear error on change
      setErrors((prev) => {
        const next = { ...prev };
        delete next[params[index].name];
        return next;
      });
    },
    [params],
  );

  const handleConfirm = useCallback(() => {
    const newErrors: Record<string, string> = {};
    const args: (number | string)[] = [];

    for (const { name, value, param } of values) {
      if (!value && value !== "0" && param.default === undefined) {
        newErrors[name] = "必填";
        args.push(0);
        continue;
      }

      const effectiveValue = value || param.default || "";

      if (param.type === "number") {
        const num = Number(effectiveValue);
        if (Number.isNaN(num)) {
          newErrors[name] = "请输入数字";
        }
        args.push(num);
      } else {
        args.push(effectiveValue);
      }
    }

    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors);
      return;
    }

    onConfirm(JSON.stringify(args));
  }, [values, onConfirm]);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      onClick={(e) => {
        if (e.target === e.currentTarget) onCancel();
      }}
      onKeyDown={(e) => {
        if (e.key === "Escape") onCancel();
      }}
    >
      <div className="bg-surface border border-border rounded-lg shadow-xl w-96 max-h-[80vh] flex flex-col">
        {/* Header */}
        <div className="px-4 py-3 border-b border-border">
          <h3 className="text-sm font-medium text-text">{label}</h3>
        </div>

        {/* Params form */}
        <div className="px-4 py-3 flex-1 overflow-y-auto space-y-3">
          {values.map((item, index) => (
            <div key={item.name}>
              <label className="block text-xs text-text-secondary mb-1">
                {item.param.label || item.name}
                {item.param.default === undefined && (
                  <span className="text-red-400 ml-1">*</span>
                )}
              </label>
              <input
                type="text"
                value={item.value}
                onChange={(e) => handleChange(index, e.target.value)}
                placeholder={
                  item.param.default !== undefined
                    ? `默认: ${item.param.default}`
                    : item.param.type === "hex"
                      ? "01 02 03"
                      : ""
                }
                className="w-full px-2 py-1.5 text-xs bg-surface-alt border border-border rounded
                           text-text placeholder:text-text-tertiary
                           focus:outline-none focus:ring-1 focus:ring-accent"
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleConfirm();
                }}
              />
              {errors[item.name] && (
                <p className="text-xs text-red-400 mt-0.5">
                  {errors[item.name]}
                </p>
              )}
            </div>
          ))}
        </div>

        {/* Actions */}
        <div className="px-4 py-3 border-t border-border flex justify-end gap-2">
          <button
            type="button"
            onClick={onCancel}
            className="px-3 py-1.5 text-xs text-text-secondary hover:text-text rounded
                       hover:bg-surface-hover transition-colors"
          >
            取消
          </button>
          <button
            type="button"
            onClick={handleConfirm}
            className="px-3 py-1.5 text-xs bg-accent text-white rounded
                       hover:bg-accent/80 transition-colors"
          >
            执行
          </button>
        </div>
      </div>
    </div>
  );
}
