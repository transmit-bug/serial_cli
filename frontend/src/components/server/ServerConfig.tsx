import { useServerStore } from "@/stores/server";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useTranslation } from "react-i18next";
import { Copy } from "lucide-react";

export function ServerConfig() {
  const { t } = useTranslation();
  const { status } = useServerStore();

  const copySocketPath = () => {
    if (status.socket_path) {
      navigator.clipboard.writeText(status.socket_path);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm font-medium">
          {t("server.configuration")}
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex items-center justify-between">
          <span className="text-sm text-text-secondary">
            {t("server.socketPath")}
          </span>
          <div className="flex items-center gap-2">
            <code className="text-xs bg-base-deep px-2 py-1 rounded text-text font-mono">
              {status.socket_path || t("server.notSet")}
            </code>
            {status.socket_path && (
              <button
                onClick={copySocketPath}
                className="text-text-secondary hover:text-text transition-colors"
                title={t("server.copyPath")}
              >
                <Copy className="w-3.5 h-3.5" />
              </button>
            )}
          </div>
        </div>

        {status.running && (
          <div className="flex items-center justify-between">
            <span className="text-sm text-text-secondary">
              {t("server.uptime")}
            </span>
            <span className="text-sm text-text font-mono">
              {Math.floor((Date.now() / 1000 - status.started_at) / 60)}m
            </span>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
