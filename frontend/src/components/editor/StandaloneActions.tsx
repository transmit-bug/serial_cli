import { useTranslation } from "react-i18next";
import type { UiAction } from "@/types";

export function StandaloneActions({
  actions,
  onCall,
}: {
  actions: UiAction[];
  onCall: (functionName: string) => Promise<void>;
}) {
  const { t } = useTranslation();

  if (actions.length === 0) return null;

  return (
    <div className="px-3 py-2 border-t border-border">
      <div className="text-[10px] uppercase tracking-wider text-text-muted mb-1.5">
        {t("scripts.actions")}
      </div>
      <div className="flex flex-wrap gap-1.5">
        {actions.map((action) => (
          <button
            key={action.function_name}
            onClick={() => onCall(action.function_name)}
            className="px-2.5 py-1 rounded text-xs bg-accent/15 text-accent hover:bg-accent/25 border border-accent/20"
            title={action.group ?? action.function_name}
          >
            {action.label || action.function_name}
          </button>
        ))}
      </div>
    </div>
  );
}
