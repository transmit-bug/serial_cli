import { create } from "zustand";
import { tauriApi } from "@/lib/tauri-api";
import type { DataPacket, DisplayFormat } from "@/types";

const MAX_PACKETS = 10000;

export interface SearchOptions {
  caseSensitive: boolean;
  useRegex: boolean;
}

export type ExportFormat = "txt" | "csv" | "json";

export interface ExportFields {
  id: boolean;
  timestamp: boolean;
  direction: boolean;
  hex: boolean;
  ascii: boolean;
  decoded: boolean;
}

export interface ExportOptions {
  format: ExportFormat;
  fields: Partial<ExportFields>;
  filteredOnly: boolean;
}

interface DataStore {
  packets: DataPacket[];
  displayFormat: DisplayFormat;
  showTimestamp: boolean;
  autoScroll: boolean;
  maxPackets: number;
  searchQuery: string;
  searchOptions: SearchOptions;
  nextId: number;
  exportOptions: ExportOptions;
  exportProgress: number | null;

  addPacket: (
    portId: string,
    direction: "rx" | "tx",
    data: number[],
    timestamp: number,
    decoded?: string,
  ) => void;
  clearBuffer: (portId?: string) => void;
  setDisplayFormat: (format: DisplayFormat) => void;
  toggleAutoScroll: () => void;
  setSearchQuery: (query: string) => void;
  setSearchOptions: (options: Partial<SearchOptions>) => void;
  setExportOptions: (opts: Partial<ExportOptions>) => void;
  setExportProgress: (progress: number | null) => void;
  getFilteredPackets: (portId?: string) => DataPacket[];
  exportData: (searchQuery: string, searchOpts: SearchOptions) => void;
  applyConfig: (cfg: {
    maxPackets?: number;
    format?: string;
    showTimestamp?: boolean;
  }) => void;
}

/** Check if a string matches a query with given search options. */
export function matchesSearch(
  text: string,
  query: string,
  options: SearchOptions,
): boolean {
  if (!query) return true;
  try {
    if (options.useRegex) {
      const flags = options.caseSensitive ? "" : "i";
      return new RegExp(query, flags).test(text);
    }
    const target = options.caseSensitive ? text : text.toLowerCase();
    const q = options.caseSensitive ? query : query.toLowerCase();
    return target.includes(q);
  } catch {
    // Invalid regex — fallback to plain includes
    return text.toLowerCase().includes(query.toLowerCase());
  }
}

export const useDataStore = create<DataStore>()((set, get) => ({
  packets: [],
  displayFormat: "mixed",
  showTimestamp: true,
  autoScroll: true,
  maxPackets: MAX_PACKETS,
  searchQuery: "",
  searchOptions: { caseSensitive: false, useRegex: false },
  nextId: 1,
  exportOptions: {
    format: "txt",
    fields: {
      id: true,
      timestamp: true,
      direction: true,
      hex: true,
      ascii: false,
      decoded: false,
    },
    filteredOnly: true,
  },
  exportProgress: null,

  addPacket: (portId, direction, data, timestamp, decoded) => {
    const { maxPackets, nextId } = get();
    const packet: DataPacket = {
      id: nextId,
      portId,
      direction,
      timestamp,
      data,
      decoded,
    };
    set((s) => ({
      packets:
        s.packets.length >= maxPackets
          ? [...s.packets.slice(1), packet]
          : [...s.packets, packet],
      nextId: nextId + 1,
    }));
  },

  clearBuffer: (portId) =>
    set((s) => ({
      packets: portId ? s.packets.filter((p) => p.portId !== portId) : [],
      nextId: portId ? s.nextId : 1,
    })),
  setDisplayFormat: (format) => set({ displayFormat: format }),
  toggleAutoScroll: () => set((s) => ({ autoScroll: !s.autoScroll })),
  setSearchQuery: (query) => set({ searchQuery: query }),
  setSearchOptions: (opts) =>
    set((s) => ({ searchOptions: { ...s.searchOptions, ...opts } })),
  setExportOptions: (opts) =>
    set((s) => {
      const newFields = opts.fields
        ? { ...s.exportOptions.fields, ...opts.fields }
        : s.exportOptions.fields;
      return {
        exportOptions: { ...s.exportOptions, ...opts, fields: newFields },
      };
    }),
  setExportProgress: (progress) => set({ exportProgress: progress }),

  getFilteredPackets: (portId) => {
    const { packets, searchQuery, searchOptions, exportOptions } = get();
    const base = portId ? packets.filter((p) => p.portId === portId) : packets;
    if (!searchQuery) return base;
    return base.filter((p) => {
      const hex = p.data.map((b) => b.toString(16).padStart(2, "0")).join(" ");
      const ascii = p.data
        .map((b) => (b >= 32 && b <= 126 ? String.fromCharCode(b) : "."))
        .join("");
      return (
        matchesSearch(hex, searchQuery, searchOptions) ||
        matchesSearch(ascii, searchQuery, searchOptions)
      );
    });
  },

  exportData: async (_searchQuery, _searchOpts) => {
    const { exportOptions, setExportProgress: setProgress } = get();
    const packets = get().getFilteredPackets();
    if (packets.length === 0) return;

    setProgress(0);

    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const ext = exportOptions.format === "txt" ? "txt" : exportOptions.format;
      const defaultPath = `serial-data-${Date.now()}.${ext}`;

      const path = await save({
        filters: [
          {
            name: `${ext.toUpperCase()} File`,
            extensions: [ext],
          },
        ],
        defaultPath,
      });

      if (!path) {
        setProgress(null);
        return;
      }

      const exportData = packets.map((p) => ({
        direction: p.direction,
        data: p.data,
        timestamp_millis: p.timestamp,
      }));

      await tauriApi.exportData(path, exportOptions.format, exportData);
      setProgress(100);
      setTimeout(() => setProgress(null), 500);
    } catch (e) {
      setProgress(null);
      throw e;
    }
  },

  applyConfig: (cfg) => {
    const updates: Partial<DataStore> = {};
    if (cfg.maxPackets && cfg.maxPackets > 0)
      updates.maxPackets = cfg.maxPackets;
    if (
      cfg.format === "hex" ||
      cfg.format === "ascii" ||
      cfg.format === "mixed"
    )
      updates.displayFormat = cfg.format;
    if (cfg.showTimestamp !== undefined)
      updates.showTimestamp = cfg.showTimestamp;
    if (Object.keys(updates).length > 0) set(updates);
  },
}));
