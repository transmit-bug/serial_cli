import { Code, Layers, Settings, Split, Terminal } from "lucide-react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import { useUIStore } from "@/stores/ui";
import type { PageName } from "@/types";

const NAV_ITEMS: { page: PageName; icon: typeof Terminal; labelKey: string }[] =
  [
    { page: "terminal", icon: Terminal, labelKey: "nav.terminal" },
    { page: "virtual", icon: Split, labelKey: "nav.virtual" },
    { page: "scripts", icon: Code, labelKey: "nav.scripts" },
    { page: "protocols", icon: Layers, labelKey: "nav.protocols" },
    { page: "settings", icon: Settings, labelKey: "nav.settings" },
  ];

export function Sidebar() {
  const { t } = useTranslation();
  const currentPage = useUIStore((s) => s.currentPage);
  const navigateTo = useUIStore((s) => s.navigateTo);

  return (
    <nav className="flex flex-col items-center w-12 bg-base-deep border-r border-border py-2">
      {NAV_ITEMS.map(({ page, icon: Icon, labelKey }) => (
        <button
          key={page}
          onClick={() => navigateTo(page)}
          title={t(labelKey)}
          className={cn(
            "flex items-center justify-center w-9 h-9 rounded-md mb-1 transition-colors",
            currentPage === page
              ? "bg-accent/20 text-accent"
              : "text-text-muted hover:bg-surface hover:text-text",
          )}
        >
          <Icon size={18} />
        </button>
      ))}
    </nav>
  );
}
