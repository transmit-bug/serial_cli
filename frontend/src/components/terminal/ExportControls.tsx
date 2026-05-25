import { Download } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import type { ExportFields, ExportFormat } from "@/stores/data";
import { useDataStore } from "@/stores/data";

const FIELD_LABELS: Record<keyof ExportFields, string> = {
  id: "export.fields.id",
  timestamp: "export.fields.timestamp",
  direction: "export.fields.direction",
  hex: "export.fields.hex",
  ascii: "export.fields.ascii",
  decoded: "export.fields.decoded",
};

export function ExportControls() {
  const { t } = useTranslation();
  const packets = useDataStore((s) => s.packets);
  const searchQuery = useDataStore((s) => s.searchQuery);
  const exportOptions = useDataStore((s) => s.exportOptions);
  const exportProgress = useDataStore((s) => s.exportProgress);
  const setExportOptions = useDataStore((s) => s.setExportOptions);
  const exportData = useDataStore((s) => s.exportData);
  const searchOptions = useDataStore((s) => s.searchOptions);

  const [showPanel, setShowPanel] = useState(false);

  const filteredCount = searchQuery
    ? packets.filter((p) => {
        const hex = p.data
          .map((b) => b.toString(16).padStart(2, "0"))
          .join(" ");
        const ascii = p.data
          .map((b) => (b >= 32 && b <= 126 ? String.fromCharCode(b) : ""))
          .join("");
        const q = searchQuery.toLowerCase();
        return hex.toLowerCase().includes(q) || ascii.toLowerCase().includes(q);
      }).length
    : packets.length;

  if (packets.length === 0) return null;

  if (exportProgress !== null) {
    return (
      <div className="p-2 border-t border-border">
        <div className="flex items-center gap-2">
          <div className="flex-1 h-1.5 bg-surface rounded-full overflow-hidden">
            <div
              className="h-full bg-accent transition-all duration-150"
              style={{ width: `${exportProgress}%` }}
            />
          </div>
          <span className="text-[10px] text-text-muted tabular-nums w-10 text-right">
            {exportProgress}%
          </span>
        </div>
      </div>
    );
  }

  if (!showPanel) {
    return (
      <div className="p-2 border-t border-border">
        <button
          onClick={() => setShowPanel(true)}
          className="w-full flex items-center justify-center gap-1 rounded px-2 py-1 text-xs bg-surface text-text-muted hover:bg-surface-hover hover:text-text transition"
        >
          <Download size={12} />
          {t("export.title")}
        </button>
      </div>
    );
  }

  const setFormat = (fmt: ExportFormat) => setExportOptions({ format: fmt });
  const toggleField = (field: keyof ExportFields) =>
    setExportOptions({ fields: { [field]: !exportOptions.fields[field] } });
  const toggleFiltered = () =>
    setExportOptions({ filteredOnly: !exportOptions.filteredOnly });

  const handleExport = async () => {
    try {
      await exportData(searchQuery, searchOptions);
      toast.success("Data exported successfully");
    } catch (e) {
      // User cancelled save dialog — not an error
      if (e && typeof e === "object" && "message" in e) {
        toast.error(String(e));
      }
    }
  };

  const formats: { key: ExportFormat; label: string }[] = [
    { key: "txt", label: "TXT" },
    { key: "csv", label: "CSV" },
    { key: "json", label: "JSON" },
  ];

  return (
    <div className="p-2 border-t border-border space-y-2">
      {/* Format selector */}
      <div className="flex items-center gap-1">
        <span className="text-[10px] text-text-muted mr-1">
          {t("export.format")}
        </span>
        <div className="flex rounded border border-border overflow-hidden text-xs">
          {formats.map(({ key, label }) => (
            <button
              key={key}
              onClick={() => setFormat(key)}
              className={`px-2 py-0.5 ${
                exportOptions.format === key
                  ? "bg-accent/20 text-accent"
                  : "text-text-muted hover:bg-surface"
              }`}
            >
              {label}
            </button>
          ))}
        </div>
      </div>

      {/* Field toggles */}
      <div className="flex flex-wrap gap-x-3 gap-y-1">
        {(Object.keys(FIELD_LABELS) as Array<keyof ExportFields>).map(
          (field) => (
            <label
              key={field}
              className="flex items-center gap-1 text-[10px] text-text-muted cursor-pointer"
            >
              <input
                type="checkbox"
                checked={exportOptions.fields[field]}
                onChange={() => toggleField(field)}
                className="accent-accent"
              />
              {t(FIELD_LABELS[field])}
            </label>
          ),
        )}
      </div>

      {/* Export filtered toggle */}
      <label className="flex items-center gap-1 text-[10px] text-text-muted cursor-pointer">
        <input
          type="checkbox"
          checked={exportOptions.filteredOnly}
          onChange={toggleFiltered}
          className="accent-accent"
        />
        {t("export.filteredOnly")}{" "}
        {searchQuery && (
          <span className="text-accent">
            ({filteredCount}/{packets.length})
          </span>
        )}
      </label>

      {/* Export button */}
      <button
        onClick={handleExport}
        className="w-full flex items-center justify-center gap-1 rounded px-2 py-1 text-xs bg-accent/15 text-accent hover:bg-accent/25 transition"
      >
        <Download size={12} />
        {t("export.download")}
      </button>

      {/* Close */}
      <button
        onClick={() => setShowPanel(false)}
        className="w-full text-[10px] text-text-muted hover:text-text transition"
      >
        {t("common.close")}
      </button>
    </div>
  );
}
