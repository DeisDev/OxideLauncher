import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, Download, AlertCircle, Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { Slider } from "@/components/ui/slider";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { cn } from "@/lib/utils";
import type { InstanceInfo, InstanceSettings, JavaInfo, JavaDownloadInfo } from "../types";

interface SettingsTabProps {
  instanceId: string;
  instance: InstanceInfo;
}

export function SettingsTab({ instanceId, instance }: SettingsTabProps) {
  const [settings, setSettings] = useState<InstanceSettings>({
    java_path: null,
    memory_min_mb: 512,
    memory_max_mb: 2048,
    java_args: "",
    game_args: "",
    window_width: 854,
    window_height: 480,
    start_maximized: false,
    console_mode: "on_error",
    pre_launch_hook: null,
    post_exit_hook: null,
    enable_analytics: false,
    enable_logging: true,
    game_dir_override: null,
  });

  const [detectedJavas, setDetectedJavas] = useState<JavaInfo[]>([]);
  const [downloadableJavas, setDownloadableJavas] = useState<JavaDownloadInfo[]>([]);
  const [detectingJava, setDetectingJava] = useState(false);
  const [downloadingJava, setDownloadingJava] = useState<string | null>(null);
  const [loadingDownloadable, setLoadingDownloadable] = useState(false);
  const [saving, setSaving] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);
  const [originalSettings, setOriginalSettings] = useState<InstanceSettings | null>(null);

  useEffect(() => {
    loadSettings();
    detectJavas();
  }, [instanceId]);

  const loadSettings = async () => {
    try {
      const loadedSettings = await invoke<InstanceSettings>("get_instance_settings", { instanceId });
      setSettings(loadedSettings);
      setOriginalSettings(loadedSettings);
      setHasChanges(false);
    } catch (error) {
      console.error("Failed to load instance settings:", error);
    }
  };

  const saveSettings = async () => {
    setSaving(true);
    try {
      await invoke("update_instance_settings", {
        instanceId,
        settings,
      });
      setOriginalSettings(settings);
      setHasChanges(false);
    } catch (error) {
      console.error("Failed to save settings:", error);
      alert("Failed to save settings: " + error);
    } finally {
      setSaving(false);
    }
  };

  const updateSetting = useCallback(<K extends keyof InstanceSettings>(
    key: K,
    value: InstanceSettings[K]
  ) => {
    setSettings(prev => {
      const updated = { ...prev, [key]: value };
      setHasChanges(JSON.stringify(updated) !== JSON.stringify(originalSettings));
      return updated;
    });
  }, [originalSettings]);

  const detectJavas = async () => {
    setDetectingJava(true);
    try {
      const javas = await invoke<JavaInfo[]>("detect_java_installations");
      setDetectedJavas(javas);
    } catch (error) {
      console.error("Failed to detect Java:", error);
    } finally {
      setDetectingJava(false);
    }
  };

  const loadDownloadableJavas = async () => {
    setLoadingDownloadable(true);
    try {
      const javas = await invoke<JavaDownloadInfo[]>("get_downloadable_javas");
      setDownloadableJavas(javas);
    } catch (error) {
      console.error("Failed to load downloadable javas:", error);
    } finally {
      setLoadingDownloadable(false);
    }
  };

  const downloadJava = async (java: JavaDownloadInfo) => {
    setDownloadingJava(java.version);
    try {
      const javaPath = await invoke<string>("download_java", {
        vendor: java.vendor,
        version: java.version,
      });
      updateSetting("java_path", javaPath);
      await detectJavas();
    } catch (error) {
      console.error("Failed to download Java:", error);
      alert("Failed to download Java: " + error);
    } finally {
      setDownloadingJava(null);
    }
  };

  const getRecommendedJava = (): number => {
    const mcVersion = instance.minecraft_version;
    const parts = mcVersion.split('.').map(Number);
    if (parts[1] >= 21) return 21;
    if (parts[1] >= 18) return 17;
    if (parts[1] >= 17) return 16;
    return 8;
  };

  const recommendedJava = getRecommendedJava();

  return (
    <ScrollArea className="h-full">
      <div className="space-y-6 pr-4 pb-4">
        <Tabs defaultValue="general" className="w-full">
          <TabsList className="grid w-full grid-cols-5 mb-4">
            <TabsTrigger value="general">General</TabsTrigger>
            <TabsTrigger value="java">Java</TabsTrigger>
            <TabsTrigger value="memory">Memory</TabsTrigger>
            <TabsTrigger value="game">Game</TabsTrigger>
            <TabsTrigger value="launch">Launch</TabsTrigger>
          </TabsList>

          {/* General Tab */}
          <TabsContent value="general" className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle>Instance Information</CardTitle>
                <CardDescription>Basic settings for this instance</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid gap-4">
                  <div className="grid gap-2">
                    <Label>Instance Name</Label>
                    <p className="text-sm text-muted-foreground">{instance.name}</p>
                  </div>
                  <div className="grid gap-2">
                    <Label>Minecraft Version</Label>
                    <p className="text-sm text-muted-foreground">{instance.minecraft_version}</p>
                  </div>
                  <div className="grid gap-2">
                    <Label>Mod Loader</Label>
                    <p className="text-sm text-muted-foreground">
                      {instance.mod_loader}
                      {instance.mod_loader_version && ` (${instance.mod_loader_version})`}
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Data Collection</CardTitle>
                <CardDescription>Privacy settings</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                  <div className="space-y-0.5">
                    <Label>Enable Analytics</Label>
                    <p className="text-sm text-muted-foreground">
                      Help improve the launcher by sending anonymous usage data
                    </p>
                  </div>
                  <Switch
                    checked={settings.enable_analytics}
                    onCheckedChange={(v) => updateSetting("enable_analytics", v)}
                  />
                </div>
                <div className="flex items-center justify-between">
                  <div className="space-y-0.5">
                    <Label>Enable Logging</Label>
                    <p className="text-sm text-muted-foreground">
                      Save debug logs for troubleshooting
                    </p>
                  </div>
                  <Switch
                    checked={settings.enable_logging}
                    onCheckedChange={(v) => updateSetting("enable_logging", v)}
                  />
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Java Tab */}
          <TabsContent value="java" className="space-y-4">
            <Alert>
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>
                Minecraft {instance.minecraft_version} requires Java {recommendedJava} or newer.
              </AlertDescription>
            </Alert>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between pb-2">
                <div>
                  <CardTitle>Detected Java Installations</CardTitle>
                  <CardDescription>Select a Java installation to use</CardDescription>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={detectJavas}
                  disabled={detectingJava}
                >
                  <RefreshCw className={cn("h-4 w-4 mr-2", detectingJava && "animate-spin")} />
                  Detect
                </Button>
              </CardHeader>
              <CardContent>
                {detectingJava ? (
                  <div className="flex items-center justify-center py-8">
                    <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
                    <span className="ml-2 text-muted-foreground">Detecting Java...</span>
                  </div>
                ) : detectedJavas.length === 0 ? (
                  <p className="text-sm text-muted-foreground text-center py-4">
                    No Java installations detected.
                  </p>
                ) : (
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead className="w-10"></TableHead>
                        <TableHead>Version</TableHead>
                        <TableHead>Vendor</TableHead>
                        <TableHead>Architecture</TableHead>
                        <TableHead>Path</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {detectedJavas.map((java) => (
                        <TableRow
                          key={java.path}
                          className={cn(
                            "cursor-pointer",
                            settings.java_path === java.path && "bg-muted"
                          )}
                          onClick={() => updateSetting("java_path", java.path)}
                        >
                          <TableCell>
                            {settings.java_path === java.path && (
                              <Check className="h-4 w-4 text-primary" />
                            )}
                          </TableCell>
                          <TableCell>
                            <div className="flex items-center gap-2">
                              <span>Java {java.major_version}</span>
                              {java.major_version >= recommendedJava && (
                                <Badge variant="secondary" className="text-xs">Compatible</Badge>
                              )}
                            </div>
                          </TableCell>
                          <TableCell>{java.vendor}</TableCell>
                          <TableCell>{java.architecture}</TableCell>
                          <TableCell className="text-xs text-muted-foreground max-w-xs truncate">
                            {java.path}
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                )}
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between pb-2">
                <div>
                  <CardTitle>Download Java</CardTitle>
                  <CardDescription>Download and install Java automatically</CardDescription>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={loadDownloadableJavas}
                  disabled={loadingDownloadable}
                >
                  <RefreshCw className={cn("h-4 w-4 mr-2", loadingDownloadable && "animate-spin")} />
                  Load Options
                </Button>
              </CardHeader>
              <CardContent>
                {downloadableJavas.length === 0 ? (
                  <p className="text-sm text-muted-foreground text-center py-4">
                    Click "Load Options" to see available Java downloads.
                  </p>
                ) : (
                  <div className="grid gap-2">
                    {downloadableJavas.map((java) => (
                      <div
                        key={`${java.vendor}-${java.version}`}
                        className="flex items-center justify-between p-3 rounded-md border"
                      >
                        <div>
                          <p className="font-medium">Java {java.version}</p>
                          <p className="text-sm text-muted-foreground">
                            {java.vendor} • {java.architecture}
                          </p>
                        </div>
                        <Button
                          size="sm"
                          onClick={() => downloadJava(java)}
                          disabled={downloadingJava === java.version}
                        >
                          {downloadingJava === java.version ? (
                            <>
                              <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                              Downloading...
                            </>
                          ) : (
                            <>
                              <Download className="h-4 w-4 mr-2" />
                              Download
                            </>
                          )}
                        </Button>
                      </div>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Custom Java Path</CardTitle>
                <CardDescription>Manually specify a Java executable path</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid gap-2">
                  <Label>Java Executable Path</Label>
                  <Input
                    value={settings.java_path || ""}
                    onChange={(e) => updateSetting("java_path", e.target.value || null)}
                    placeholder="Auto-detect (recommended)"
                  />
                  <p className="text-xs text-muted-foreground">
                    Leave empty to use auto-detected Java
                  </p>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Memory Tab */}
          <TabsContent value="memory" className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle>Memory Allocation</CardTitle>
                <CardDescription>
                  Configure how much RAM Minecraft can use
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-4">
                  <div className="grid gap-2">
                    <div className="flex items-center justify-between">
                      <Label>Minimum Memory</Label>
                      <span className="text-sm font-mono">{settings.memory_min_mb} MB</span>
                    </div>
                    <Slider
                      value={[settings.memory_min_mb]}
                      onValueChange={([v]) => {
                        updateSetting("memory_min_mb", v);
                        if (v > settings.memory_max_mb) {
                          updateSetting("memory_max_mb", v);
                        }
                      }}
                      min={256}
                      max={16384}
                      step={256}
                    />
                  </div>

                  <div className="grid gap-2">
                    <div className="flex items-center justify-between">
                      <Label>Maximum Memory</Label>
                      <span className="text-sm font-mono">{settings.memory_max_mb} MB</span>
                    </div>
                    <Slider
                      value={[settings.memory_max_mb]}
                      onValueChange={([v]) => {
                        updateSetting("memory_max_mb", v);
                        if (v < settings.memory_min_mb) {
                          updateSetting("memory_min_mb", v);
                        }
                      }}
                      min={256}
                      max={16384}
                      step={256}
                    />
                  </div>
                </div>

                <Separator />

                <div className="text-sm text-muted-foreground space-y-1">
                  <p>
                    <strong>Recommendations:</strong>
                  </p>
                  <ul className="list-disc list-inside space-y-1 ml-2">
                    <li>Vanilla: 2-4 GB</li>
                    <li>Light modpacks: 4-6 GB</li>
                    <li>Heavy modpacks: 6-10 GB</li>
                  </ul>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Game Tab */}
          <TabsContent value="game" className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle>Window Settings</CardTitle>
                <CardDescription>Configure the game window</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div className="grid gap-2">
                    <Label>Window Width</Label>
                    <Input
                      type="number"
                      value={settings.window_width}
                      onChange={(e) => updateSetting("window_width", parseInt(e.target.value) || 854)}
                      min={640}
                      max={7680}
                    />
                  </div>
                  <div className="grid gap-2">
                    <Label>Window Height</Label>
                    <Input
                      type="number"
                      value={settings.window_height}
                      onChange={(e) => updateSetting("window_height", parseInt(e.target.value) || 480)}
                      min={480}
                      max={4320}
                    />
                  </div>
                </div>
                <div className="flex items-center justify-between">
                  <div className="space-y-0.5">
                    <Label>Start Maximized</Label>
                    <p className="text-sm text-muted-foreground">
                      Launch the game in a maximized window
                    </p>
                  </div>
                  <Switch
                    checked={settings.start_maximized}
                    onCheckedChange={(v) => updateSetting("start_maximized", v)}
                  />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Game Arguments</CardTitle>
                <CardDescription>Custom JVM and game arguments</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid gap-2">
                  <Label>JVM Arguments</Label>
                  <Textarea
                    value={settings.java_args}
                    onChange={(e) => updateSetting("java_args", e.target.value)}
                    placeholder="-XX:+UseG1GC -XX:+UnlockExperimentalVMOptions"
                    rows={3}
                    className="font-mono text-sm"
                  />
                  <p className="text-xs text-muted-foreground">
                    Additional arguments passed to the Java virtual machine
                  </p>
                </div>

                <div className="grid gap-2">
                  <Label>Game Arguments</Label>
                  <Textarea
                    value={settings.game_args}
                    onChange={(e) => updateSetting("game_args", e.target.value)}
                    placeholder="--fullscreen"
                    rows={2}
                    className="font-mono text-sm"
                  />
                  <p className="text-xs text-muted-foreground">
                    Additional arguments passed to Minecraft
                  </p>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Game Directory</CardTitle>
                <CardDescription>Override the default game directory</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid gap-2">
                  <Label>Custom Game Directory</Label>
                  <Input
                    value={settings.game_dir_override || ""}
                    onChange={(e) => updateSetting("game_dir_override", e.target.value || null)}
                    placeholder="Default (instance directory)"
                  />
                  <p className="text-xs text-muted-foreground">
                    Leave empty to use the instance's directory
                  </p>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Launch Tab */}
          <TabsContent value="launch" className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle>Console Behavior</CardTitle>
                <CardDescription>Control when the console window is shown</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid gap-2">
                  <Label>Show Console</Label>
                  <Select
                    value={settings.console_mode}
                    onValueChange={(v: "always" | "on_error" | "never") => updateSetting("console_mode", v)}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="always">Always</SelectItem>
                      <SelectItem value="on_error">On Error</SelectItem>
                      <SelectItem value="never">Never</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    When to show the game console output window
                  </p>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Launch Hooks</CardTitle>
                <CardDescription>Run commands before launching or after closing</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid gap-2">
                  <Label>Pre-Launch Command</Label>
                  <Input
                    value={settings.pre_launch_hook || ""}
                    onChange={(e) => updateSetting("pre_launch_hook", e.target.value || null)}
                    placeholder="Optional command to run before launch"
                    className="font-mono text-sm"
                  />
                </div>
                <div className="grid gap-2">
                  <Label>Post-Exit Command</Label>
                  <Input
                    value={settings.post_exit_hook || ""}
                    onChange={(e) => updateSetting("post_exit_hook", e.target.value || null)}
                    placeholder="Optional command to run after game closes"
                    className="font-mono text-sm"
                  />
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>

        {/* Save Button */}
        {hasChanges && (
          <div className="sticky bottom-0 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 border-t pt-4 -mx-4 px-4">
            <div className="flex items-center justify-end gap-4">
              <span className="text-sm text-muted-foreground">You have unsaved changes</span>
              <Button
                variant="outline"
                onClick={loadSettings}
                disabled={saving}
              >
                Discard
              </Button>
              <Button onClick={saveSettings} disabled={saving}>
                {saving ? (
                  <>
                    <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                    Saving...
                  </>
                ) : (
                  "Save Changes"
                )}
              </Button>
            </div>
          </div>
        )}
      </div>
    </ScrollArea>
  );
}
