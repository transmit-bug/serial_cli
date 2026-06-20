import { useServerStore } from "@/stores/server";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useTranslation } from "react-i18next";
import { Activity, AlertCircle, CheckCircle } from "lucide-react";

export function ServerStats() {
  const { t } = useTranslation();
  const { status } = useServerStore();

  const uptime = status.running
    ? Math.floor((Date.now() / 1000 - status.started_at) / 60)
    : 0;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm font-medium">
          {t("server.statistics")}
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid grid-cols-3 gap-4">
          <div className="flex flex-col items-center p-3 bg-base-deep rounded-lg">
            <Activity className="w-5 h-5 text-primary mb-2" />
            <div className="text-2xl font-bold text-text">
              {status.total_requests}
            </div>
            <div className="text-xs text-text-secondary">
              {t("server.requests")}
            </div>
          </div>

          <div className="flex flex-col items-center p-3 bg-base-deep rounded-lg">
            <CheckCircle className="w-5 h-5 text-green-500 mb-2" />
            <div className="text-2xl font-bold text-text">
              {status.active_connections}
            </div>
            <div className="text-xs text-text-secondary">
              {t("server.connections")}
            </div>
          </div>

          <div className="flex flex-col items-center p-3 bg-base-deep rounded-lg">
            <AlertCircle className="w-5 h-5 text-red-500 mb-2" />
            <div className="text-2xl font-bold text-text">
              {status.total_errors}
            </div>
            <div className="text-xs text-text-secondary">
              {t("server.errors")}
            </div>
          </div>
        </div>

        {status.running && (
          <div className="pt-2 border-t border-border">
            <div className="flex items-center justify-between text-sm">
              <span className="text-text-secondary">{t("server.uptime")}</span>
              <span className="text-text font-mono">{uptime}m</span>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
