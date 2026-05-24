import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { useConnectionStore } from "@/stores/connection";
import { useSerialScriptStore } from "@/stores/serialScript";

export function RightPanelActions() {
  const { t } = useTranslation();
  const portId = useConnectionStore((s) => s.portId);
  const isConnected = useConnectionStore((s) => s.status === "connected");
  const actions = useSerialScriptStore((s) => s.actions);
  const callAction = useSerialScriptStore((s) => s.callAction);

  const handleCallAction = useCallback(
    async (fn: string) => {
      if (!portId) return;
      try {
        const result = await callAction(portId, fn);
        toast.success(result);
      } catch (e) {
        toast.error(String(e));
      }
    },
    [portId, callAction],
  );

  if (!isConnected || actions.length === 0) return null;

  return (
    <div className="p-3">
      <h3 className="text-xs font-medium text-text-secondary mb-2">
        {t("terminal.actions")}
      </h3>
      <div className="flex flex-col gap-1">
        {actions.map((action) => (
          <button
            key={action.function_name}
            onClick={() => handleCallAction(action.function_name)}
            className="px-2 py-1 rounded text-xs bg-surface hover:bg-surface-hover text-text text-left"
          >
            {action.label}
          </button>
        ))}
      </div>
    </div>
  );
}
