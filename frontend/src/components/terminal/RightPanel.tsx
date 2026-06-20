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
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
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

  return (
    <Tabs
      value={activeTab}
      onValueChange={(value) => setActiveTab(value as RightPanelTab)}
      className="flex flex-col h-full"
    >
      <TabsList variant="line" className="shrink-0 overflow-x-auto pb-1">
        {TABS.map(({ id, icon: Icon, labelKey, compactKey }) => (
          <TabsTrigger key={id} value={id} title={t(labelKey)}>
            <Icon size={12} />
            <span className="hidden lg:inline">{t(labelKey)}</span>
            <span className="lg:hidden">{compactKey}</span>
          </TabsTrigger>
        ))}
      </TabsList>

      {TABS.map(({ id }) => (
        <TabsContent
          key={id}
          value={id}
          className="flex-1 overflow-hidden mt-0"
        >
          {TAB_COMPONENTS[id]()}
        </TabsContent>
      ))}
    </Tabs>
  );
}
