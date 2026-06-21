import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { useConnectionStore } from "@/stores/connection";
import { useSerialScriptStore } from "@/stores/serialScript";
import type { UiAction } from "@/types";
import { ActionParamsModal } from "./ActionParamsModal";

/** Group actions by their `group` field. */
function groupActions(actions: UiAction[]): Map<string, UiAction[]> {
  const groups = new Map<string, UiAction[]>();
  for (const action of actions) {
    const key = action.group ?? "";
    const arr = groups.get(key) ?? [];
    arr.push(action);
    groups.set(key, arr);
  }
  return groups;
}

export function RightPanelActions() {
  const { t } = useTranslation();
  const portId = useConnectionStore((s) => s.activePortId);
  const isConnected = useConnectionStore(
    (s) =>
      s.activePortId != null &&
      s.connections.some(
        (c) => c.portId === s.activePortId && c.status === "connected",
      ),
  );
  const actions = useSerialScriptStore((s) => s.actions);
  const callAction = useSerialScriptStore((s) => s.callAction);

  // Modal state: which action is pending param input
  const [pendingAction, setPendingAction] = useState<UiAction | null>(null);

  const handleCallAction = useCallback(
    async (action: UiAction, argsJson?: string) => {
      if (!portId) return;

      // If action requires confirmation, show browser confirm
      if (action.confirm) {
        const ok = window.confirm(
          `确认执行 "${action.label}" ?`,
        );
        if (!ok) return;
      }

      // Check if params need user input (no default)
      if (!argsJson && action.params.length > 0) {
        const hasRequired = action.params.some(
          (p) => p.default === undefined,
        );
        if (hasRequired) {
          // Open modal to collect params
          setPendingAction(action);
          return;
        }
        // All params have defaults — use them
        const defaultArgs = action.params.map(
          (p) => Number(p.default) ?? p.default,
        );
        argsJson = JSON.stringify(defaultArgs);
      }

      try {
        const result = await callAction(portId, action.function_name, argsJson);
        toast.success(result || "ok");
      } catch (e) {
        toast.error(String(e));
      }
    },
    [portId, callAction],
  );

  const handleModalConfirm = useCallback(
    async (argsJson: string) => {
      if (!pendingAction || !portId) return;
      setPendingAction(null);
      try {
        const result = await callAction(
          portId,
          pendingAction.function_name,
          argsJson,
        );
        toast.success(result || "ok");
      } catch (e) {
        toast.error(String(e));
      }
    },
    [pendingAction, portId, callAction],
  );

  if (!isConnected || actions.length === 0) return null;

  const grouped = groupActions(actions);
  const hasGroups = [...grouped.keys()].some((k) => k !== "");

  return (
    <div className="p-3">
      <h3 className="text-xs font-medium text-text-secondary mb-2">
        {t("terminal.actions", "脚本动作")}
      </h3>

      {hasGroups ? (
        // Grouped layout
        <div className="space-y-3">
          {[...grouped.entries()].map(([group, groupActions]) => (
            <div key={group || "_ungrouped"}>
              {group && (
                <div className="text-[10px] font-medium text-text-tertiary uppercase tracking-wider mb-1">
                  {group}
                </div>
              )}
              <div className="flex flex-col gap-1">
                {groupActions.map((action) => (
                  <ActionButton
                    key={action.function_name}
                    action={action}
                    onClick={() => handleCallAction(action)}
                  />
                ))}
              </div>
            </div>
          ))}
        </div>
      ) : (
        // Flat layout (no groups)
        <div className="flex flex-col gap-1">
          {actions.map((action) => (
            <ActionButton
              key={action.function_name}
              action={action}
              onClick={() => handleCallAction(action)}
            />
          ))}
        </div>
      )}

      {/* Param input modal */}
      {pendingAction && (
        <ActionParamsModal
          label={pendingAction.label}
          params={pendingAction.params}
          onConfirm={handleModalConfirm}
          onCancel={() => setPendingAction(null)}
        />
      )}
    </div>
  );
}

/** Single action button with icon and param indicator. */
function ActionButton({
  action,
  onClick,
}: {
  action: UiAction;
  onClick: () => void;
}) {
  const requiredParams = action.params.filter((p) => p.default === undefined);
  const hasRequired = requiredParams.length > 0;

  return (
    <button
      type="button"
      onClick={onClick}
      className="group flex items-center gap-2 px-2 py-1.5 rounded text-xs
                 bg-surface hover:bg-surface-hover text-text text-left
                 transition-colors"
      title={
        hasRequired
          ? `需要参数: ${requiredParams.map((p) => p.label || p.name).join(", ")}`
          : undefined
      }
    >
      <span className="flex-1 truncate">{action.label}</span>
      {hasRequired && (
        <span className="text-[10px] text-text-tertiary opacity-0 group-hover:opacity-100 transition-opacity">
          {requiredParams.length} 参数
        </span>
      )}
    </button>
  );
}
