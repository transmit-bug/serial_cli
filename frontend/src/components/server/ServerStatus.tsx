import { useServerStore } from "@/stores/server";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { useTranslation } from "react-i18next";
import { Power, PowerOff } from "lucide-react";

export function ServerStatus() {
  const { t } = useTranslation();
  const { status, loading, startServer, stopServer } = useServerStore();

  const handleToggle = async () => {
    if (status.running) {
      await stopServer();
    } else {
      await startServer();
    }
  };

  return (
    <Card>
      <CardContent className="pt-6">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2">
              <div
                className={`w-3 h-3 rounded-full ${
                  status.running ? "bg-green-500 animate-pulse" : "bg-gray-400"
                }`}
              />
              <span className="text-sm font-medium text-text">
                {status.running ? t("server.running") : t("server.stopped")}
              </span>
            </div>
            {status.running && (
              <Badge variant="secondary" className="text-xs">
                {t("server.activeConnections", {
                  count: status.active_connections,
                })}
              </Badge>
            )}
          </div>

          <Button
            onClick={handleToggle}
            disabled={loading}
            variant={status.running ? "destructive" : "default"}
            size="sm"
          >
            {status.running ? (
              <>
                <PowerOff className="w-4 h-4 mr-2" />
                {t("server.stop")}
              </>
            ) : (
              <>
                <Power className="w-4 h-4 mr-2" />
                {t("server.start")}
              </>
            )}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
