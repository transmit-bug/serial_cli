import { useCallback, useMemo } from "react";
import { Command } from "cmdk";
import {
  Terminal,
  Split,
  Code2,
  Settings,
  Plug,
  PlugZap2,
  Eraser,
  RefreshCw,
  PanelLeftClose,
  PanelRightClose,
  Download,
  Eye,
  EyeOff,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { useUIStore } from "@/stores/ui";
import { useConnectionStore } from "@/stores/connection";
import { useDataStore } from "@/stores/data";
import { usePresetsStore } from "@/stores/presets";
import type { PageName } from "@/types";

interface CommandPaletteProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

interface PaletteItem {
  id: string;
  label: string;
  icon: React.ComponentType<{ size?: number; className?: string }>;
  shortcut?: string;
  group?: string;
  action: () => void | Promise<void>;
}

export function CommandPalette({ open, onOpenChange }: CommandPaletteProps) {
  const { t } = useTranslation();
  const navigateTo = useUIStore((s) => s.navigateTo);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);
  const toggleRightPanel = useUIStore((s) => s.toggleRightPanel);
  const sidebarCollapsed = useUIStore((s) => s.sidebarCollapsed);
  const rightPanelCollapsed = useUIStore((s) => s.rightPanelCollapsed);
  const activeEntry = useConnectionStore((s) =>
    s.connections.find((c) => c.portId === s.activePortId),
  );
  const isConnected = activeEntry?.status === "connected";
  const activePortId = useConnectionStore((s) => s.activePortId);
  const disconnect = useConnectionStore((s) => s.disconnect);
  const refreshPorts = useConnectionStore((s) => s.refreshPorts);
  const clearBuffer = useDataStore((s) => s.clearBuffer);
  const displayFormat = useDataStore((s) => s.displayFormat);
  const setDisplayFormat = useDataStore((s) => s.setDisplayFormat);
  const autoScroll = useDataStore((s) => s.autoScroll);
  const toggleAutoScroll = useDataStore((s) => s.toggleAutoScroll);
  const exportData = useDataStore((s) => s.exportData);
  const presets = usePresetsStore((s) => s.presets);
  const applyPreset = usePresetsStore((s) => s.applyPreset);

  // Navigate helper
  const nav = useCallback(
    (page: PageName) => {
      navigateTo(page);
      onOpenChange(false);
    },
    [navigateTo, onOpenChange],
  );

  // Export helper
  const handleExport = useCallback(async () => {
    try {
      await exportData("", { caseSensitive: false, useRegex: false });
    } catch {
      // User cancelled or error
    } finally {
      onOpenChange(false);
    }
  }, [exportData, onOpenChange]);

  // Build commands list — memoized so it updates when stores change
  const commands: PaletteItem[] = useMemo(() => {
    const items: PaletteItem[] = [
      // ─── Navigation ───
      {
        id: "nav-terminal",
        label: t("nav.terminal"),
        icon: Terminal,
        shortcut: "⌘1",
        group: t("commandPalette.navigation"),
        action: () => nav("terminal"),
      },
      {
        id: "nav-virtual",
        label: t("nav.virtual"),
        icon: Split,
        shortcut: "⌘2",
        group: t("commandPalette.navigation"),
        action: () => nav("virtual"),
      },
      {
        id: "nav-editor",
        label: t("nav.editor"),
        icon: Code2,
        shortcut: "⌘3",
        group: t("commandPalette.navigation"),
        action: () => nav("editor"),
      },
      {
        id: "nav-settings",
        label: t("nav.settings"),
        icon: Settings,
        shortcut: "⌘4",
        group: t("commandPalette.navigation"),
        action: () => nav("settings"),
      },

      // ─── Connection ───
      ...(isConnected
        ? [
            {
              id: "disconnect",
              label: t("common.disconnect"),
              icon: PlugZap2,
              group: t("commandPalette.connection"),
              action: () => {
                if (activePortId) disconnect(activePortId);
                onOpenChange(false);
              },
            } satisfies PaletteItem,
          ]
        : [
            {
              id: "connect-pending",
              label: t("common.connect"),
              icon: Plug,
              group: t("commandPalette.connection"),
              action: () => {
                nav("terminal");
              },
            } satisfies PaletteItem,
          ]),

      {
        id: "refresh-ports",
        label: t("common.refresh"),
        icon: RefreshCw,
        group: t("commandPalette.connection"),
        action: async () => {
          await refreshPorts();
          toast.success(t("commandPalette.portsRefreshed"));
          onOpenChange(false);
        },
      },

      // ─── Data View ───
      {
        id: "clear-buffer",
        label: t("terminal.clearBuffer"),
        icon: Eraser,
        shortcut: "⌘L",
        group: t("commandPalette.dataView"),
        action: () => {
          clearBuffer(activePortId ?? undefined);
          onOpenChange(false);
        },
      },
      {
        id: "format-hex",
        label: t("terminal.hexMode"),
        icon: Eye,
        shortcut: displayFormat === "hex" ? "✓" : undefined,
        group: t("commandPalette.dataView"),
        action: () => {
          setDisplayFormat("hex");
          onOpenChange(false);
        },
      },
      {
        id: "format-ascii",
        label: t("terminal.asciiMode"),
        icon: Eye,
        shortcut: displayFormat === "ascii" ? "✓" : undefined,
        group: t("commandPalette.dataView"),
        action: () => {
          setDisplayFormat("ascii");
          onOpenChange(false);
        },
      },
      {
        id: "format-mixed",
        label: t("terminal.mixedMode"),
        icon: Eye,
        shortcut: displayFormat === "mixed" ? "✓" : undefined,
        group: t("commandPalette.dataView"),
        action: () => {
          setDisplayFormat("mixed");
          onOpenChange(false);
        },
      },
      {
        id: "toggle-auto-scroll",
        label: autoScroll
          ? t("commandPalette.disableAutoScroll")
          : t("commandPalette.enableAutoScroll"),
        icon: autoScroll ? EyeOff : Eye,
        group: t("commandPalette.dataView"),
        action: () => {
          toggleAutoScroll();
          onOpenChange(false);
        },
      },
      {
        id: "export-data",
        label: t("export.title"),
        icon: Download,
        group: t("commandPalette.dataView"),
        action: handleExport,
      },

      // ─── Layout ───
      {
        id: "toggle-sidebar",
        label: sidebarCollapsed
          ? t("commandPalette.expandSidebar")
          : t("commandPalette.collapseSidebar"),
        icon: PanelLeftClose,
        shortcut: "⌘B",
        group: t("commandPalette.layout"),
        action: () => {
          toggleSidebar();
          onOpenChange(false);
        },
      },
      {
        id: "toggle-right-panel",
        label: rightPanelCollapsed
          ? t("commandPalette.showRightPanel")
          : t("commandPalette.hideRightPanel"),
        icon: PanelRightClose,
        shortcut: "⌘E",
        group: t("commandPalette.layout"),
        action: () => {
          toggleRightPanel();
          onOpenChange(false);
        },
      },

      // ─── Connection Presets ───
      ...presets.map(
        (p) =>
          ({
            id: `preset-${p.name}`,
            label: `${t("presets.apply")} — ${p.name}`,
            icon: Plug,
            group: t("commandPalette.presets"),
            action: () => {
              applyPreset(p);
              toast.success(t("presets.apply"));
              onOpenChange(false);
            },
          }) as PaletteItem,
      ),
    ];

    return items;
  }, [
    t,
    isConnected,
    activePortId,
    displayFormat,
    autoScroll,
    sidebarCollapsed,
    rightPanelCollapsed,
    presets,
    nav,
    disconnect,
    refreshPorts,
    clearBuffer,
    setDisplayFormat,
    toggleAutoScroll,
    handleExport,
    toggleSidebar,
    toggleRightPanel,
    onOpenChange,
    applyPreset,
  ]);

  // Group commands for rendering
  const groupedCommands = useMemo(() => {
    const groups = new Map<string, PaletteItem[]>();
    for (const cmd of commands) {
      const group = cmd.group || t("commandPalette.other");
      if (!groups.has(group)) groups.set(group, []);
      groups.get(group)!.push(cmd);
    }
    return groups;
  }, [commands, t]);

  // Handle selection
  const handleSelect = useCallback(
    (action: () => void | Promise<void>) => {
      onOpenChange(false);
      action();
    },
    [onOpenChange],
  );

  return (
    <Command.Dialog
      open={open}
      onOpenChange={onOpenChange}
      label={t("commandPalette.title")}
      className="fixed inset-0 z-[100] flex items-start justify-center pt-[20vh]"
    >
      {/* Backdrop */}
      <button
        type="button"
        className="absolute inset-0 bg-black/60 backdrop-blur-sm border-0 cursor-default"
        onClick={() => onOpenChange(false)}
        onKeyDown={(e) => {
          if (e.key === "Escape") onOpenChange(false);
        }}
        aria-label="Close command palette"
      />

      {/* Dialog */}
      <div className="relative w-full max-w-lg bg-base-deep border border-border rounded-xl shadow-2xl overflow-hidden">
        {/* Search input */}
        <Command.Input
          placeholder={t("commandPalette.search")}
          className="w-full border-none bg-transparent px-4 py-3 text-sm text-text placeholder:text-text-muted outline-none"
        />
        <Command.List className="max-h-80 overflow-y-auto px-2 pb-2">
          <Command.Empty className="py-8 text-center text-xs text-text-muted">
            {t("commandPalette.noResults")}
          </Command.Empty>

          {Array.from(groupedCommands.entries()).map(
            ([group, items]) =>
              items.length > 0 && (
                <Command.Group
                  key={group}
                  heading={group}
                  className="px-2 pt-2 pb-1 text-[10px] font-semibold uppercase tracking-wider text-text-muted"
                >
                  {items.map((item) => (
                    <Command.Item
                      key={item.id}
                      value={item.label}
                      onSelect={() => handleSelect(item.action)}
                      className="flex items-center gap-3 rounded-md px-3 py-2 text-sm cursor-pointer select-none data-[selected=true]:bg-surface data-[selected=true]:text-accent"
                    >
                      <item.icon
                        size={16}
                        className="shrink-0 text-text-muted"
                      />
                      <span className="flex-1 truncate">{item.label}</span>
                      {item.shortcut && (
                        <kbd className="shrink-0 px-1.5 py-0.5 rounded bg-base text-[10px] font-mono text-text-muted/70">
                          {item.shortcut}
                        </kbd>
                      )}
                    </Command.Item>
                  ))}
                </Command.Group>
              ),
          )}
        </Command.List>
      </div>
    </Command.Dialog>
  );
}
