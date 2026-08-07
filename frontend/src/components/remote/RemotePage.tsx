import {
  Download,
  Pencil,
  Plug,
  Plus,
  Radio,
  RefreshCw,
  Send,
  Trash2,
  Unplug,
  Wifi,
  WifiOff,
} from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type { RemoteDevice } from "@/types";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { setupRemoteDataListener, useRemoteStore } from "@/stores/remote";

const BAUD_RATES = [9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600];

function hexToBytes(hex: string): number[] {
  const clean = hex.replace(/\s+/g, "");
  if (!/^[0-9a-fA-F]*$/.test(clean)) return [];
  const bytes: number[] = [];
  for (let i = 0; i + 1 < clean.length; i += 2) {
    bytes.push(parseInt(clean.slice(i, i + 2), 16));
  }
  return bytes;
}

export function RemotePage() {
  const { t } = useTranslation();
  const {
    devices,
    loading,
    error,
    activeDeviceId,
    ports,
    connections,
    workbenchLoading,
    workbenchError,
    loadDevices,
    addDevice,
    updateDevice,
    deleteDevice,
    testDevice,
    selectDevice,
    refreshPorts,
    openRemotePort,
    closeRemoteConnection,
    sendRemoteData,
    recvRemoteData,
    startStream,
    stopStream,
    appendStreamData,
    clearRx,
    streaming,
    rxBuffers,
  } = useRemoteStore();

  // Workbench state
  const [selectedPort, setSelectedPort] = useState("");
  const [baudrate, setBaudrate] = useState("115200");
  const [activeConnectionId, setActiveConnectionId] = useState("");
  const [txText, setTxText] = useState("");
  const [testStates, setTestStates] = useState<Record<string, boolean>>({});

  useEffect(() => {
    loadDevices();
  }, [loadDevices]);

  // Setup data-stream event listeners; stop streams on unmount
  useEffect(() => {
    const unlisten = setupRemoteDataListener();
    return () => {
      unlisten.then((un) => un());
      const activeDeviceId = useRemoteStore.getState().activeDeviceId;
      if (activeDeviceId) {
        useRemoteStore.getState().stopDeviceStreams(activeDeviceId);
      }
    };
  }, []);

  const rxOutput = activeConnectionId
    ? (rxBuffers[activeConnectionId] ?? "")
    : "";

  // Device form dialog
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [name, setName] = useState("");
  const [host, setHost] = useState("");
  const [port, setPort] = useState("23333");

  const openAddDialog = () => {
    setEditingId(null);
    setName("");
    setHost("");
    setPort("23333");
    setDialogOpen(true);
  };

  const openEditDialog = (id: string) => {
    const device = devices.find((d) => d.id === id);
    if (!device) return;
    setEditingId(id);
    setName(device.name);
    setHost(device.host);
    setPort(String(device.port));
    setDialogOpen(true);
  };

  const handleSaveDevice = async () => {
    const portNum = parseInt(port, 10);
    if (editingId) {
      await updateDevice(editingId, name, host, portNum);
    } else {
      await addDevice(name, host, portNum);
    }
    setDialogOpen(false);
  };

  const handleTest = async (id: string) => {
    const ok = await testDevice(id);
    setTestStates((s) => ({ ...s, [id]: ok }));
  };

  // Delete confirmation
  const [deleteTarget, setDeleteTarget] = useState<RemoteDevice | null>(null);

  const confirmDelete = async () => {
    if (!deleteTarget) return;
    const id = deleteTarget.id;
    setDeleteTarget(null);
    await deleteDevice(id);
  };

  const handleSelectDevice = async (id: string) => {
    if (activeDeviceId === id) {
      await selectDevice(null);
    } else {
      await selectDevice(id);
    }
  };

  const handleOpenPort = async () => {
    if (!selectedPort) return;
    const result = await openRemotePort(selectedPort, parseInt(baudrate, 10));
    if (result) {
      setActiveConnectionId(result.connection_id);
      clearRx(result.connection_id);
    }
  };

  const handleSend = async () => {
    if (!activeConnectionId) return;
    const bytes = hexToBytes(txText);
    if (bytes.length === 0) return;
    await sendRemoteData(activeConnectionId, bytes);
  };

  const handleRecv = async () => {
    if (!activeConnectionId || !activeDevice) return;
    const result = await recvRemoteData(activeConnectionId, 1000);
    if (result && result.bytes_read > 0) {
      appendStreamData({
        device_id: activeDevice.id,
        connection_id: activeConnectionId,
        data: result.data,
        bytes_read: result.bytes_read,
        timestamp: Math.floor(Date.now() / 1000),
      });
    }
  };

  const toggleStream = async () => {
    if (!activeConnectionId) return;
    if (streaming[activeConnectionId]) {
      await stopStream(activeConnectionId);
    } else {
      await startStream(activeConnectionId);
    }
  };

  const activeDevice = devices.find((d) => d.id === activeDeviceId);

  return (
    <div className="flex flex-col h-full bg-base">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-border">
        <h1 className="text-lg font-semibold text-text">{t("remote.title")}</h1>
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={loadDevices}
            disabled={loading}
          >
            <RefreshCw className="w-4 h-4 mr-1" />
            {t("remote.refresh")}
          </Button>
          <Button size="sm" onClick={openAddDialog}>
            <Plus className="w-4 h-4 mr-1" />
            {t("remote.addDevice")}
          </Button>
        </div>
      </div>

      {error && (
        <div className="px-4 py-2 text-sm text-danger bg-surface border-b border-border">
          {error}
        </div>
      )}

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {/* Device list */}
        <Card>
          <CardHeader>
            <CardTitle className="text-sm font-medium">
              {t("remote.deviceList")}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {devices.length === 0 && (
              <p className="text-sm text-text-secondary">
                {t("remote.noDevices")}
              </p>
            )}
            {devices.map((device) => (
              <div
                key={device.id}
                className={`flex items-center justify-between px-3 py-2 rounded border ${
                  activeDeviceId === device.id
                    ? "border-accent bg-surface"
                    : "border-border bg-surface/50"
                }`}
              >
                <button
                  className="flex items-center gap-3 text-left flex-1 min-w-0"
                  onClick={() => handleSelectDevice(device.id)}
                  title={t("remote.selectToOperate")}
                >
                  <div className="min-w-0">
                    <div className="text-sm font-medium text-text truncate">
                      {device.name}
                    </div>
                    <div className="text-xs text-text-secondary font-mono">
                      {device.host}:{device.port}
                    </div>
                  </div>
                  <Badge
                    variant={testStates[device.id] ? "default" : "secondary"}
                    className="ml-1 shrink-0"
                  >
                    {testStates[device.id] ? (
                      <Wifi className="w-3 h-3 mr-1" />
                    ) : (
                      <WifiOff className="w-3 h-3 mr-1" />
                    )}
                    {testStates[device.id]
                      ? t("remote.online")
                      : t("remote.unknown")}
                  </Badge>
                </button>

                <div className="flex items-center gap-1 shrink-0">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleTest(device.id)}
                    title={t("remote.testConnection")}
                  >
                    <Plug className="w-4 h-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => openEditDialog(device.id)}
                    title={t("common.edit")}
                  >
                    <Pencil className="w-4 h-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setDeleteTarget(device)}
                    title={t("common.delete")}
                  >
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </div>
              </div>
            ))}
          </CardContent>
        </Card>

        {/* Workbench */}
        {activeDevice && (
          <div className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle className="text-sm font-medium">
                  {t("remote.workbench")} — {activeDevice.name}
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                {workbenchError && (
                  <div className="px-3 py-2 text-sm text-danger bg-surface rounded border border-border">
                    {workbenchError}
                  </div>
                )}

                {/* Port open */}
                <div className="space-y-2">
                  <Label className="text-xs text-text-secondary">
                    {t("remote.openPort")}
                  </Label>
                  <div className="flex gap-2">
                    <Select
                      value={selectedPort}
                      onValueChange={setSelectedPort}
                    >
                      <SelectTrigger className="flex-1">
                        <SelectValue placeholder={t("remote.selectPort")} />
                      </SelectTrigger>
                      <SelectContent>
                        {ports.map((p) => (
                          <SelectItem key={p.port_name} value={p.port_name}>
                            {p.port_name}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    <Select value={baudrate} onValueChange={setBaudrate}>
                      <SelectTrigger className="w-28">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {BAUD_RATES.map((b) => (
                          <SelectItem key={b} value={String(b)}>
                            {b}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    <Button
                      size="sm"
                      onClick={handleOpenPort}
                      disabled={workbenchLoading || !selectedPort}
                    >
                      <Plug className="w-4 h-4 mr-1" />
                      {t("remote.open")}
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={refreshPorts}
                      disabled={workbenchLoading}
                      title={t("remote.refresh")}
                    >
                      <RefreshCw className="w-4 h-4" />
                    </Button>
                  </div>
                </div>

                {/* Connections */}
                <div className="space-y-2">
                  <Label className="text-xs text-text-secondary">
                    {t("remote.connections")}
                  </Label>
                  {connections.length === 0 && (
                    <p className="text-sm text-text-secondary">
                      {t("remote.noConnections")}
                    </p>
                  )}
                  {connections.map((conn) => (
                    <div
                      key={conn.connection_id}
                      className={`flex items-center justify-between px-3 py-2 rounded border ${
                        activeConnectionId === conn.connection_id
                          ? "border-accent bg-surface"
                          : "border-border bg-surface/50"
                      }`}
                    >
                      <div className="min-w-0">
                        <div className="text-sm text-text font-mono truncate">
                          {conn.port_id ?? conn.connection_id}
                        </div>
                        <div className="text-xs text-text-secondary font-mono truncate">
                          {conn.connection_id}
                        </div>
                      </div>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() =>
                          setActiveConnectionId(conn.connection_id)
                        }
                        disabled={activeConnectionId === conn.connection_id}
                      >
                        {t("remote.use")}
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() =>
                          closeRemoteConnection(conn.connection_id)
                        }
                        title={t("remote.close")}
                      >
                        <Unplug className="w-4 h-4" />
                      </Button>
                    </div>
                  ))}
                </div>

                {/* Tx / Rx */}
                <div className="space-y-2">
                  <Label className="text-xs text-text-secondary">
                    {t("remote.sendData")}
                  </Label>
                  <div className="flex gap-2">
                    <Input
                      value={txText}
                      onChange={(e) => setTxText(e.target.value)}
                      placeholder={t("remote.txPlaceholder")}
                      className="font-mono"
                      onKeyDown={(e) => {
                        if (e.key === "Enter") handleSend();
                      }}
                    />
                    <Button
                      size="sm"
                      onClick={handleSend}
                      disabled={!activeConnectionId}
                    >
                      <Send className="w-4 h-4 mr-1" />
                      {t("remote.send")}
                    </Button>
                  </div>

                  <div className="flex items-center justify-between">
                    <Label className="text-xs text-text-secondary">
                      {t("remote.receiveData")}
                    </Label>
                    <div className="flex items-center gap-1">
                      {activeConnectionId && (
                        <Button
                          variant={
                            streaming[activeConnectionId]
                              ? "default"
                              : "outline"
                          }
                          size="sm"
                          onClick={toggleStream}
                        >
                          <Radio className="w-4 h-4 mr-1" />
                          {streaming[activeConnectionId]
                            ? t("remote.stopStream")
                            : t("remote.startStream")}
                        </Button>
                      )}
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={handleRecv}
                        disabled={!activeConnectionId}
                      >
                        <Download className="w-4 h-4 mr-1" />
                        {t("remote.receive")}
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() =>
                          activeConnectionId && clearRx(activeConnectionId)
                        }
                        disabled={!activeConnectionId}
                      >
                        <Trash2 className="w-4 h-4" />
                      </Button>
                    </div>
                  </div>
                  <pre className="min-h-24 max-h-48 overflow-y-auto p-3 rounded bg-surface border border-border text-xs font-mono text-text whitespace-pre-wrap">
                    {rxOutput || t("remote.rxPlaceholder")}
                  </pre>
                </div>
              </CardContent>
            </Card>
          </div>
        )}
      </div>

      {/* Delete confirmation dialog */}
      <Dialog
        open={deleteTarget !== null}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("remote.confirmDeleteTitle")}</DialogTitle>
          </DialogHeader>
          <p className="text-sm text-text-secondary">
            {deleteTarget
              ? t("remote.confirmDeleteBody", { name: deleteTarget.name })
              : ""}
          </p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteTarget(null)}>
              {t("common.cancel")}
            </Button>
            <Button variant="destructive" onClick={confirmDelete}>
              {t("common.delete")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Add/Edit dialog */}
      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {editingId ? t("remote.editDevice") : t("remote.addDevice")}
            </DialogTitle>
          </DialogHeader>
          <div className="space-y-3">
            <div className="space-y-1">
              <Label>{t("remote.deviceName")}</Label>
              <Input
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="RPi Lab Board"
              />
            </div>
            <div className="space-y-1">
              <Label>{t("remote.host")}</Label>
              <Input
                value={host}
                onChange={(e) => setHost(e.target.value)}
                placeholder="192.168.1.50"
              />
            </div>
            <div className="space-y-1">
              <Label>{t("remote.port")}</Label>
              <Input
                value={port}
                onChange={(e) => setPort(e.target.value)}
                placeholder="23333"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDialogOpen(false)}>
              {t("common.cancel")}
            </Button>
            <Button onClick={handleSaveDevice} disabled={!name || !host}>
              {t("common.save")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
