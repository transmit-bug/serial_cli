import { useTranslation } from "react-i18next";
import { X } from "lucide-react";

const SHORTCUTS = [
  { keys: "⌘/Ctrl + 1", key: "shortcuts.navTerminal" },
  { keys: "⌘/Ctrl + 2", key: "shortcuts.navVirtual" },
  { keys: "⌘/Ctrl + 3", key: "shortcuts.navEditor" },
  { keys: "⌘/Ctrl + 4", key: "shortcuts.navSettings" },
  { keys: "⌘/Ctrl + B", key: "shortcuts.toggleSidebar" },
  { keys: "⌘/Ctrl + E", key: "shortcuts.toggleRightPanel" },
  { keys: "⌘/Ctrl + L", key: "shortcuts.clearBuffer" },
  { keys: "⌘/Ctrl + /", key: "shortcuts.showHelp" },
  { keys: "F1 – F12", key: "shortcuts.quickSend" },
  { keys: "Ctrl + Enter", key: "shortcuts.sendData" },
];

export function ShortcutsHelp({
  onClose,
}: {
  onClose: () => void;
}) {
  const { t } = useTranslation();

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      onClick={onClose}
    >
      <div
        className="bg-base-deep border border-border rounded-lg shadow-2xl w-80 max-h-[80vh] overflow-y-auto"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-4 py-3 border-b border-border">
          <span className="text-sm font-medium">{t("shortcuts.title")}</span>
          <button
            onClick={onClose}
            className="text-text-muted hover:text-text transition-colors"
          >
            <X size={16} />
          </button>
        </div>
        <div className="p-3 flex flex-col gap-1">
          {SHORTCUTS.map(({ keys, key }) => (
            <div
              key={key}
              className="flex items-center justify-between py-1.5 px-1 text-xs"
            >
              <span className="text-text-muted">{t(key)}</span>
              <kbd className="px-1.5 py-0.5 rounded bg-surface text-text font-mono text-[11px]">
                {keys}
              </kbd>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
