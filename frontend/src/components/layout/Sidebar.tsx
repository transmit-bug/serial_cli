import {
  ChevronLeft,
  ChevronRight,
  Code2,
  Settings,
  Split,
  Terminal,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import { useUIStore } from "@/stores/ui";
import type { PageName } from "@/types";

const NAV_ITEMS: { page: PageName; icon: typeof Terminal; labelKey: string }[] =
  [
    { page: "terminal", icon: Terminal, labelKey: "nav.terminal" },
    { page: "virtual", icon: Split, labelKey: "nav.virtual" },
    { page: "editor", icon: Code2, labelKey: "nav.editor" },
    { page: "settings", icon: Settings, labelKey: "nav.settings" },
  ];

export function Sidebar() {
  const { t } = useTranslation();
  const currentPage = useUIStore((s) => s.currentPage);
  const navigateTo = useUIStore((s) => s.navigateTo);
  const collapsed = useUIStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);

  return (
    <aside
      className={cn(
        "flex flex-col bg-base-deep border-r border-border transition-all duration-200",
        collapsed ? "w-12" : "w-48",
      )}
    >
      {/* Nav items */}
      <nav className="flex flex-col gap-0.5 p-1.5 overflow-y-auto flex-1">
        {NAV_ITEMS.map(({ page, icon: Icon, labelKey }) => (
          <button
            key={page}
            onClick={() => navigateTo(page)}
            title={t(labelKey)}
            className={cn(
              "flex items-center rounded-md transition-colors shrink-0",
              collapsed
                ? "justify-center w-9 h-9 mx-auto"
                : "gap-2.5 px-2.5 h-9",
              currentPage === page
                ? "bg-accent/20 text-accent"
                : "text-text-muted hover:bg-surface hover:text-text",
            )}
          >
            <Icon size={18} className="shrink-0" />
            {!collapsed && (
              <span className="text-xs truncate">{t(labelKey)}</span>
            )}
          </button>
        ))}
      </nav>

      {/* Toggle button */}
      <div className="border-t border-border p-1.5">
        <button
          onClick={toggleSidebar}
          title={collapsed ? "展开侧边栏" : "收起侧边栏"}
          className={cn(
            "flex items-center rounded-md transition-colors shrink-0",
            collapsed
              ? "justify-center w-9 h-9 mx-auto"
              : "gap-2.5 px-2.5 h-9 w-full",
            "text-text-muted hover:bg-surface hover:text-text",
          )}
        >
          {collapsed ? <ChevronRight size={16} /> : <ChevronLeft size={16} />}
          {!collapsed && <span className="text-xs truncate">收起</span>}
        </button>
      </div>
    </aside>
  );
}
