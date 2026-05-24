import { create } from "zustand";
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
  fields: ExportFields;
  filteredOnly: boolean;
}

interface DataStore {
  packets: DataPacket[];
  displayFormat: DisplayFormat;
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

  exportData: (_searchQuery, _searchOpts) => {
    const { exportOptions, setExportProgress: setProgress } = get();
    const packets = get().getFilteredPackets();
    if (packets.length === 0) return;

    const { format, fields } = exportOptions;
    const CHUNK = 500;
    let progress = 0;

    const buildContent = (): {
      content: string;
      mimeType: string;
      ext: string;
    } => {
      if (format === "json") {
        const items = packets.map((p) => {
          const item: Record<string, unknown> = {};
          if (fields.id) item.id = p.id;
          if (fields.timestamp)
            item.timestamp = new Date(p.timestamp).toISOString();
          if (fields.direction) item.direction = p.direction;
          if (fields.hex)
            item.hex = p.data
              .map((b) => b.toString(16).padStart(2, "0"))
              .join(" ");
          if (fields.ascii)
            item.ascii = p.data
              .map((b) => (b >= 32 && b <= 126 ? String.fromCharCode(b) : "."))
              .join("");
          if (fields.decoded && p.decoded) item.decoded = p.decoded;
          return item;
        });
        return {
          content: JSON.stringify(items, null, 2),
          mimeType: "application/json",
          ext: "json",
        };
      }

      if (format === "csv") {
        const colNames: string[] = [];
        if (fields.id) colNames.push("id");
        if (fields.timestamp) colNames.push("timestamp");
        if (fields.direction) colNames.push("direction");
        if (fields.hex) colNames.push("hex");
        if (fields.ascii) colNames.push("ascii");
        if (fields.decoded) colNames.push("decoded");
        const header = `${colNames.join(",")}\n`;
        const rows = packets.map((p) => {
          const vals: string[] = [];
          if (fields.id) vals.push(String(p.id));
          if (fields.timestamp) vals.push(new Date(p.timestamp).toISOString());
          if (fields.direction) vals.push(p.direction);
          if (fields.hex)
            vals.push(
              `"${p.data.map((b) => b.toString(16).padStart(2, "0")).join(" ")}"`,
            );
          if (fields.ascii)
            vals.push(
              `"${p.data.map((b) => (b >= 32 && b <= 126 ? String.fromCharCode(b) : ".")).join("")}"`,
            );
          if (fields.decoded)
            vals.push(`"${(p.decoded ?? "").replace(/"/g, '""')}"`);
          return vals.join(",");
        });
        return {
          content: header + rows.join("\n"),
          mimeType: "text/csv",
          ext: "csv",
        };
      }

      // TXT
      const lines = packets.map((p) => {
        const parts: string[] = [];
        if (fields.timestamp)
          parts.push(`[${new Date(p.timestamp).toISOString()}]`);
        if (fields.id) parts.push(`#${p.id}`);
        if (fields.direction) parts.push(p.direction.toUpperCase());
        if (fields.hex)
          parts.push(
            p.data.map((b) => b.toString(16).padStart(2, "0")).join(" "),
          );
        if (fields.ascii)
          parts.push(
            p.data
              .map((b) => (b >= 32 && b <= 126 ? String.fromCharCode(b) : "."))
              .join(""),
          );
        if (fields.decoded && p.decoded) parts.push(`(decoded: ${p.decoded})`);
        return parts.join(" ");
      });
      return { content: lines.join("\n"), mimeType: "text/plain", ext: "txt" };
    };

    if (packets.length > 10000) {
      const chunks = Math.ceil(packets.length / CHUNK);

      let i = 0;
      const processChunk = () => {
        if (i >= chunks) {
          const { content, mimeType, ext } = buildContent();
          downloadFile(content, mimeType, ext);
          setProgress(null);
          return;
        }
        i++;
        progress = Math.round((i / chunks) * 100);
        setProgress(progress);
        setTimeout(processChunk, 0);
      };
      processChunk();
    } else {
      const { content, mimeType, ext } = buildContent();
      downloadFile(content, mimeType, ext);
    }
  },
}));

function downloadFile(content: string, mimeType: string, ext: string) {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `serial-data-${Date.now()}.${ext}`;
  a.click();
  URL.revokeObjectURL(url);
}
