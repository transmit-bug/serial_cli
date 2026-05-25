import {
  Activity,
  BarChart3,
  FileClock,
  ListTodo,
  Puzzle,
  ScrollText,
  Zap,
} from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import { RightPanelActions } from "./RightPanelActions";
import { RightPanelCommands } from "./RightPanelCommands";
import { RightPanelDecoder } from "./RightPanelDecoder";
import { RightPanelHistory } from "./RightPanelHistory";
import { RightPanelLog } from "./RightPanelLog";
import { RightPanelMonitor } from "./RightPanelMonitor";
import { RightPanelStats } from "./RightPanelStats";

type RightPanelTab =
  | "stats"
  | "commands"
  | "monitor"
  | "history"
  | "decoder"
  | "actions"
  | "log";

interface TabDef {
  id: RightPanelTab;
  icon: typeof Activity;
  labelKey: string;
  compactKey: string;
}

const TABS: TabDef[] = [
  {
    id: "stats",
    icon: Activity,
    labelKey: "rightPanel.stats",
    compactKey: "Stats",
  },
  {
    id: "commands",
    icon: ListTodo,
    labelKey: "rightPanel.commands",
    compactKey: "Cmds",
  },
  {
    id: "monitor",
    icon: BarChart3,
    labelKey: "rightPanel.monitor",
    compactKey: "Mon",
  },
  {
    id: "history",
    icon: FileClock,
    labelKey: "rightPanel.history",
    compactKey: "Hist",
  },
  {
    id: "decoder",
    icon: Puzzle,
    labelKey: "rightPanel.decoder",
    compactKey: "Dec",
  },
  {
    id: "actions",
    icon: Zap,
    labelKey: "rightPanel.actions",
    compactKey: "Act",
  },
  {
    id: "log",
    icon: ScrollText,
    labelKey: "rightPanel.log",
    compactKey: "Log",
  },
];

const TAB_COMPONENTS: Record<RightPanelTab, () => React.ReactNode> = {
  stats: () => <RightPanelStats />,
  commands: () => <RightPanelCommands />,
  monitor: () => <RightPanelMonitor />,
  history: () => <RightPanelHistory />,
  decoder: () => <RightPanelDecoder />,
  actions: () => <RightPanelActions />,
  log: () => <RightPanelLog />,
};

export function RightPanel() {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<RightPanelTab>("stats");
  const RenderComponent = TAB_COMPONENTS[activeTab];

  return (
    <div className="flex flex-col h-full">
      {/* Tab bar */}
      <div className="flex items-center border-b border-border shrink-0 overflow-x-auto">
        {TABS.map(({ id, icon: Icon, labelKey, compactKey }) => (
          <button
            key={id}
            className={cn(
              "flex items-center gap-1 px-2 py-1.5 text-xs whitespace-nowrap border-b-2 transition-colors",
              activeTab === id
                ? "border-accent text-accent"
                : "border-transparent text-text-muted hover:text-text hover:bg-surface/30",
            )}
            onClick={() => setActiveTab(id)}
            title={t(labelKey)}
          >
            <Icon size={12} />
            <span className="hidden lg:inline">{t(labelKey)}</span>
            <span className="lg:hidden">{compactKey}</span>
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-hidden">
        <RenderComponent />
      </div>
    </div>
  );
}
