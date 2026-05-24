import { useCallback, useEffect } from "react";
import { useUIStore } from "@/stores/ui";

export function useKeyboardShortcuts() {
  const navigateTo = useUIStore((s) => s.navigateTo);
  const toggleRightPanel = useUIStore((s) => s.toggleRightPanel);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey;
      if (!mod) return;

      const keyMap: Record<string, () => void> = {
        "1": () => navigateTo("terminal"),
        "2": () => navigateTo("virtual"),
        "3": () => navigateTo("scripts"),
        "4": () => navigateTo("protocols"),
        "5": () => navigateTo("settings"),
        "\\": toggleRightPanel,
        ",": () => navigateTo("settings"),
      };

      const action = keyMap[e.key];
      if (action) {
        e.preventDefault();
        action();
      }
    },
    [navigateTo, toggleRightPanel],
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);
}
