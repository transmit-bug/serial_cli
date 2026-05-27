import { useEffect } from "react";
import { useSettingsStore } from "@/stores/settings";

/** Returns the current theme name derived from the loaded config. */
export function useTheme(): string {
  const config = useSettingsStore((s) => s.config);
  const theme = config?.display?.theme ?? "dark";

  useEffect(() => {
    const effective = theme === "light" ? "light" : "dark";
    document.documentElement.setAttribute("data-theme", effective);
    // Also store in localStorage for instant re-application on next load
    localStorage.setItem("theme", effective);
  }, [theme]);

  return theme;
}

/** Apply stored theme synchronously before React renders (call in main.tsx). */
export function applyStoredTheme() {
  const stored = localStorage.getItem("theme");
  if (stored === "light") {
    document.documentElement.setAttribute("data-theme", "light");
  } else {
    document.documentElement.removeAttribute("data-theme");
  }
}
