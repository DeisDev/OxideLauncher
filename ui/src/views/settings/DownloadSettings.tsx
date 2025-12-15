import { useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Slider } from "@/components/ui/slider";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { HelpCircle } from "lucide-react";
import { useSettings } from "./context";
import type { ProxyType } from "./types";

// Tooltip helper for settings
function SettingTooltip({ children }: { children: React.ReactNode }) {
  return (
    <TooltipProvider delayDuration={200}>
      <Tooltip>
        <TooltipTrigger asChild>
          <HelpCircle className="h-4 w-4 text-muted-foreground cursor-help inline-flex ml-1.5" />
        </TooltipTrigger>
        <TooltipContent side="right" className="max-w-xs">
          <p className="text-sm">{children}</p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

export function DownloadSettings() {
  const { config, setConfig } = useSettings();
  const [proxyEnabled, setProxyEnabled] = useState(!!config?.network.proxy);
  const [proxyAuthEnabled, setProxyAuthEnabled] = useState(
    !!(config?.network.proxy?.username || config?.network.proxy?.password)
  );

  if (!config) return null;

  const handleProxyToggle = (enabled: boolean) => {
    setProxyEnabled(enabled);
    if (!enabled) {
      setConfig({
        ...config,
        network: { ...config.network, proxy: null },
      });
    } else {
      setConfig({
        ...config,
        network: {
          ...config.network,
          proxy: {
            proxy_type: "Http",
            host: "127.0.0.1",
            port: 8080,
            username: null,
            password: null,
          },
        },
      });
    }
  };

  const updateProxy = (field: string, value: string | number | null) => {
    if (!config.network.proxy) return;
    setConfig({
      ...config,
      network: {
        ...config.network,
        proxy: { ...config.network.proxy, [field]: value },
      },
    });
  };

  return (
    <div className="space-y-6">
      {/* Concurrent Downloads */}
      <Card>
        <CardHeader>
          <CardTitle>Download Performance</CardTitle>
          <CardDescription>
            Configure how the launcher handles file downloads.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label className="inline-flex items-center">
                  Concurrent Downloads
                  <SettingTooltip>
                    Higher values download more files at once, speeding up installations. Lower values use less bandwidth and are more stable on slower connections.
                  </SettingTooltip>
                </Label>
                <p className="text-sm text-muted-foreground">
                  Number of files to download simultaneously.
                </p>
              </div>
              <span className="text-lg font-semibold w-12 text-right">
                {config.network.max_concurrent_downloads}
              </span>
            </div>
            <Slider
              value={[config.network.max_concurrent_downloads]}
              onValueChange={([value]) =>
                setConfig({
                  ...config,
                  network: { ...config.network, max_concurrent_downloads: value },
                })
              }
              min={1}
              max={50}
              step={1}
              className="w-full"
            />
            <div className="flex justify-between text-xs text-muted-foreground">
              <span>1 (Slowest)</span>
              <span>50 (Fastest)</span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Retry Settings */}
      <Card>
        <CardHeader>
          <CardTitle>Download Retry</CardTitle>
          <CardDescription>
            Configure automatic retry behavior for failed downloads.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label className="inline-flex items-center">
                  Auto-Retry Count
                  <SettingTooltip>
                    Failed downloads will automatically retry with exponential backoff. Set to 0 to disable auto-retry.
                  </SettingTooltip>
                </Label>
                <p className="text-sm text-muted-foreground">
                  Number of retry attempts before skipping a file.
                </p>
              </div>
              <span className="text-lg font-semibold w-12 text-right">
                {config.network.download_retries}
              </span>
            </div>
            <Slider
              value={[config.network.download_retries]}
              onValueChange={([value]) =>
                setConfig({
                  ...config,
                  network: { ...config.network, download_retries: value },
                })
              }
              min={0}
              max={10}
              step={1}
              className="w-full"
            />
            <div className="flex justify-between text-xs text-muted-foreground">
              <span>0 (No Retry)</span>
              <span>10 (Max)</span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Timeout Settings */}
      <Card>
        <CardHeader>
          <CardTitle>Request Timeout</CardTitle>
          <CardDescription>
            Configure HTTP request timeout settings.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label className="inline-flex items-center">
                  Timeout (seconds)
                  <SettingTooltip>
                    Increase this value if you have a slow connection or experience frequent timeouts. Decrease for faster failure detection.
                  </SettingTooltip>
                </Label>
                <p className="text-sm text-muted-foreground">
                  How long to wait for a server response before timing out.
                </p>
              </div>
              <span className="text-lg font-semibold w-16 text-right">
                {config.network.timeout_seconds}s
              </span>
            </div>
            <Slider
              value={[config.network.timeout_seconds]}
              onValueChange={([value]) =>
                setConfig({
                  ...config,
                  network: { ...config.network, timeout_seconds: value },
                })
              }
              min={5}
              max={300}
              step={5}
              className="w-full"
            />
            <div className="flex justify-between text-xs text-muted-foreground">
              <span>5s</span>
              <span>300s (5 min)</span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Proxy Settings */}
      <Card>
        <CardHeader>
          <CardTitle>Proxy Settings</CardTitle>
          <CardDescription>
            Configure a proxy server for network requests.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="proxyEnabled">Enable Proxy</Label>
              <p className="text-sm text-muted-foreground">
                Route all network traffic through a proxy server.
              </p>
            </div>
            <Switch
              id="proxyEnabled"
              checked={proxyEnabled}
              onCheckedChange={handleProxyToggle}
            />
          </div>

          {proxyEnabled && config.network.proxy && (
            <>
              <div className="grid grid-cols-2 gap-4 pt-4">
                <div className="space-y-2">
                  <Label htmlFor="proxyType">Proxy Type</Label>
                  <Select
                    value={config.network.proxy.proxy_type}
                    onValueChange={(value: ProxyType) => updateProxy("proxy_type", value)}
                  >
                    <SelectTrigger id="proxyType">
                      <SelectValue placeholder="Select type" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="Http">HTTP</SelectItem>
                      <SelectItem value="Socks5">SOCKS5</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label htmlFor="proxyHost">Host</Label>
                  <Input
                    id="proxyHost"
                    value={config.network.proxy.host}
                    onChange={(e) => updateProxy("host", e.target.value)}
                    placeholder="127.0.0.1"
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="proxyPort">Port</Label>
                  <Input
                    id="proxyPort"
                    type="number"
                    value={config.network.proxy.port}
                    onChange={(e) => updateProxy("port", parseInt(e.target.value) || 8080)}
                    placeholder="8080"
                  />
                </div>
              </div>

              <div className="flex items-center justify-between pt-2">
                <div className="space-y-0.5">
                  <Label htmlFor="proxyAuth">Proxy Authentication</Label>
                  <p className="text-sm text-muted-foreground">
                    Enable if your proxy requires a username and password.
                  </p>
                </div>
                <Switch
                  id="proxyAuth"
                  checked={proxyAuthEnabled}
                  onCheckedChange={(checked) => {
                    setProxyAuthEnabled(checked);
                    if (!checked) {
                      updateProxy("username", null);
                      updateProxy("password", null);
                    }
                  }}
                />
              </div>

              {proxyAuthEnabled && (
                <div className="grid grid-cols-2 gap-4 pt-2">
                  <div className="space-y-2">
                    <Label htmlFor="proxyUsername">Username</Label>
                    <Input
                      id="proxyUsername"
                      value={config.network.proxy.username || ""}
                      onChange={(e) => updateProxy("username", e.target.value || null)}
                      placeholder="Username"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="proxyPassword">Password</Label>
                    <Input
                      id="proxyPassword"
                      type="password"
                      value={config.network.proxy.password || ""}
                      onChange={(e) => updateProxy("password", e.target.value || null)}
                      placeholder="Password"
                    />
                  </div>
                </div>
              )}
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
