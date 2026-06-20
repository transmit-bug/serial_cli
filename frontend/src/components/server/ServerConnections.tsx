import { useServerStore } from "@/stores/server";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useTranslation } from "react-i18next";
import { Users, Clock, Wifi } from "lucide-react";

export function ServerConnections() {
  const { t } = useTranslation();
  const { status } = useServerStore();

  const formatConnectionTime = (startedAt: number) => {
    const now = Math.floor(Date.now() / 1000);
    const diff = now - startedAt;
    const minutes = Math.floor(diff / 60);
    const seconds = diff % 60;
    return `${minutes}m ${seconds}s`;
  };

  return (
    <Card className="h-full">
      <CardHeader>
        <CardTitle className="text-sm font-medium flex items-center gap-2">
          <Users className="w-4 h-4" />
          {t("server.activeConnections")}
          <span className="ml-auto text-xs text-text-secondary">
            {status.connections.length}
          </span>
        </CardTitle>
      </CardHeader>
      <CardContent>
        {status.connections.length === 0 ? (
          <div className="text-center py-8 text-text-secondary text-sm">
            {t("server.noConnections")}
          </div>
        ) : (
          <div className="space-y-3">
            {status.connections.map((conn) => (
              <div
                key={conn.connection_id}
                className="p-3 bg-base-deep rounded-lg space-y-2"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Wifi className="w-4 h-4 text-primary" />
                    <span className="text-sm font-medium text-text">
                      {conn.connection_id.slice(0, 8)}...
                    </span>
                  </div>
                  {conn.subscribed && (
                    <span className="text-xs px-2 py-0.5 bg-primary/10 text-primary rounded">
                      {t("server.subscribed")}
                    </span>
                  )}
                </div>

                {conn.port_id && (
                  <div className="flex items-center gap-2 text-xs text-text-secondary">
                    <span className="font-medium">{t("server.port")}:</span>
                    <span>{conn.port_id}</span>
                  </div>
                )}

                {conn.protocol && (
                  <div className="flex items-center gap-2 text-xs text-text-secondary">
                    <span className="font-medium">{t("server.protocol")}:</span>
                    <span>{conn.protocol}</span>
                  </div>
                )}

                <div className="flex items-center gap-2 text-xs text-text-secondary">
                  <Clock className="w-3 h-3" />
                  <span>{formatConnectionTime(conn.created_at)}</span>
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
