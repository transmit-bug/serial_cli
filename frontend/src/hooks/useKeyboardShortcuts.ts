import { useCallback, useEffect, useState } from "react";
import { useUIStore } from "@/stores/ui";
import { useDataStore } from "@/stores/data";
import { useConnectionStore } from "@/stores/connection";

export interface ShortcutDef {
  keys: string;
  descriptionKey: string;
  action: () => void;
}

export function useKeyboardShortcuts() {
  const navigateTo = useUIStore((s) => s.navigateTo);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);
  const toggleRightPanel = useUIStore((s) => s.toggleRightPanel);
  const clearBuffer = useDataStore((s) => s.clearBuffer);
  const activePortId = useConnectionStore((s) => s.activePortId);
  const [showHelp, setShowHelp] = useState(false);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      // Ignore when typing in inputs
      const target = e.target as HTMLElement;
      if (
        target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.tagName === "SELECT" ||
        target.isContentEditable
      ) {
        // Only intercept Escape from inputs
        if (e.key === "Escape") {
          target.blur();
        }
        return;
      }

      const mod = e.metaKey || e.ctrlKey;

      // Ctrl/Cmd + Shift shortcuts
      if (mod && e.shiftKey) {
        const shiftKeyMap: Record<string, () => void> = {
          "/": () => setShowHelp(true),
          "?": () => setShowHelp(true),
        };
        const action = shiftKeyMap[e.key];
        if (action) {
          e.preventDefault();
          action();
          return;
        }
      }

      // Ctrl/Cmd shortcuts
      if (mod) {
        const keyMap: Record<string, () => void> = {
          "1": () => navigateTo("terminal"),
          "2": () => navigateTo("virtual"),
          "3": () => navigateTo("editor"),
          "4": () => navigateTo("settings"),
          b: toggleSidebar,
          e: toggleRightPanel,
          l: () => clearBuffer(activePortId ?? undefined),
          "/": () => setShowHelp(true),
        };

        const action = keyMap[e.key];
        if (action) {
          e.preventDefault();
          action();
          return;
        }
      }

      // Escape to close help
      if (e.key === "Escape" && showHelp) {
        setShowHelp(false);
      }
    },
    [navigateTo, toggleSidebar, toggleRightPanel, clearBuffer, activePortId, showHelp],
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  return { showHelp, setShowHelp };
}
