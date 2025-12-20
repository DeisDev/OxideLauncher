// Instance settings tab for Java, memory, and per-instance configuration
//
// Oxide Launcher — A Rust-based Minecraft launcher
// Copyright (C) 2025 Oxide Launcher contributors
//
// This file is part of Oxide Launcher.
//
// Oxide Launcher is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Oxide Launcher is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, Download, AlertCircle, Check, AlertTriangle, Sparkles, Zap, HelpCircle } from "lucide-react";
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
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { cn } from "@/lib/utils";
import type { InstanceInfo, InstanceSettings, JavaInfo } from "../types";

// Tooltip helper component for settings
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

// Java compatibility check result from backend
interface JavaCompatibilityResult {
  compatible: boolean;
  java_major: number;
  required_major: number;
  min_compatible: number;
  max_compatible: number;
  message: string;
}

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
    fullscreen: false,
    console_mode: "on_error",
    pre_launch_hook: null,
    post_exit_hook: null,
    enable_analytics: false,
    enable_logging: true,
    game_dir_override: null,
    skip_java_compatibility_check: false,
    wrapper_command: null,
    // Debug settings
    use_java_console: false,
    disable_create_no_window: false,
    log_launch_command: false,
  });

  const [detectedJavas, setDetectedJavas] = useState<JavaInfo[]>([]);
  const [javaCompatibility, setJavaCompatibility] = useState<Map<string, JavaCompatibilityResult>>(new Map());
  const [detectingJava, setDetectingJava] = useState(false);
  const [downloadingJava, setDownloadingJava] = useState<number | null>(null);
  const [findingBestJava, setFindingBestJava] = useState(false);
  const [saveStatus, setSaveStatus] = useState<"idle" | "saving" | "saved">("idle");
  
  // Debounce timer ref
  const saveTimeoutRef = useRef<number | null>(null);

  // Load settings and detect javas on mount
  useEffect(() => {
    loadSettings();
    detectJavas();
  }, [instanceId]);

  // Check compatibility whenever detected javas or minecraft version changes
  useEffect(() => {
    if (detectedJavas.length > 0) {
      checkAllJavaCompatibility();
    }
  }, [detectedJavas, instance.minecraft_version]);

  // Debounced auto-save
  const saveSettings = useCallback(async (newSettings: InstanceSettings) => {
    if (saveTimeoutRef.current) {
      window.clearTimeout(saveTimeoutRef.current);
    }

    setSaveStatus("saving");
    
    saveTimeoutRef.current = window.setTimeout(async () => {
      try {
        await invoke("update_instance_settings", {
          instanceId,
          settings: newSettings,
        });
        setSaveStatus("saved");
        // Reset to idle after showing "saved" briefly
        setTimeout(() => setSaveStatus("idle"), 1500);
      } catch (error) {
        console.error("Failed to save settings:", error);
        setSaveStatus("idle");
      }
    }, 300); // 300ms debounce
  }, [instanceId]);

  const loadSettings = async () => {
    try {
      const loadedSettings = await invoke<InstanceSettings>("get_instance_settings", { instanceId });
      setSettings(loadedSettings);
    } catch (error) {
      console.error("Failed to load instance settings:", error);
    }
  };

  const updateSetting = useCallback(<K extends keyof InstanceSettings>(
    key: K,
    value: InstanceSettings[K]
  ) => {
    setSettings(prev => {
      const updated = { ...prev, [key]: value };
      saveSettings(updated);
      return updated;
    });
  }, [saveSettings]);

  const detectJavas = async () => {
    setDetectingJava(true);
    try {
      const javas = await invoke<JavaInfo[]>("detect_java");
      const mappedJavas = javas.map((j: any) => ({
        path: j.path,
        version: j.version,
        major_version: j.major_version,
        vendor: j.vendor,
        architecture: j.arch,
        is_64bit: j.is_64bit,
      }));
      setDetectedJavas(mappedJavas);
    } catch (error) {
      console.error("Failed to detect Java:", error);
    } finally {
      setDetectingJava(false);
    }
  };

  const checkAllJavaCompatibility = async () => {
    const compatMap = new Map<string, JavaCompatibilityResult>();
    
    for (const java of detectedJavas) {
      try {
        const result = await invoke<JavaCompatibilityResult>("check_java_compatibility", {
          javaMajorVersion: java.major_version,
          minecraftVersion: instance.minecraft_version,
        });
        compatMap.set(java.path, result);
      } catch (error) {
        console.error(`Failed to check compatibility for ${java.path}:`, error);
      }
    }
    
    setJavaCompatibility(compatMap);
  };

  const findBestJava = async (autoDownload: boolean = false) => {
    setFindingBestJava(true);
    try {
      const result = await invoke<{ path: string } | null>("find_best_java_for_instance", {
        minecraftVersion: instance.minecraft_version,
        autoDownload,
      });
      
      if (result) {
        updateSetting("java_path", result.path);
        await detectJavas();
      } else if (!autoDownload) {
        // No compatible Java found, prompt to download
        const requiredJava = getRequiredJavaVersion();
        await downloadJavaVersion(requiredJava);
      }
    } catch (error) {
      console.error("Failed to find best Java:", error);
    } finally {
      setFindingBestJava(false);
    }
  };

  const downloadJavaVersion = async (majorVersion: number) => {
    setDownloadingJava(majorVersion);
    try {
      const result = await invoke<{ path: string }>("download_java", {
        majorVersion,
      });
      updateSetting("java_path", result.path);
      await detectJavas();
    } catch (error) {
      console.error("Failed to download Java:", error);
    } finally {
      setDownloadingJava(null);
    }
  };

  const getRequiredJavaVersion = (): number => {
    const mcVersion = instance.minecraft_version;
    const parts = mcVersion.split('.').map(Number);
    if (parts[1] >= 21) return 21;
    if (parts[1] >= 18) return 17;
    if (parts[1] >= 17) return 16;
    return 8;
  };

  // Get current Java selection status
  const getCurrentJavaStatus = (): { isAuto: boolean; selectedJava: JavaInfo | null; compatibility: JavaCompatibilityResult | null } => {
    // Auto-detect is when java_path is null OR empty string
    const isAuto = !settings.java_path;
    const selectedJava = isAuto ? null : detectedJavas.find(j => j.path === settings.java_path) || null;
    const compatibility = selectedJava ? javaCompatibility.get(selectedJava.path) || null : null;
    
    return { isAuto, selectedJava, compatibility };
  };

  // Check if any compatible Java exists
  const hasCompatibleJava = (): boolean => {
    return Array.from(javaCompatibility.values()).some(c => c.compatible);
  };

  const requiredJava = getRequiredJavaVersion();
  const { isAuto, selectedJava, compatibility } = getCurrentJavaStatus();
  const hasAnyCompatible = hasCompatibleJava();

  return (
    <ScrollArea className="h-full">
      <div className="space-y-6 pr-4 pb-4">
        {/* Auto-save indicator */}
        {saveStatus !== "idle" && (
          <div className="fixed top-4 right-4 z-50">
            <Badge variant={saveStatus === "saving" ? "secondary" : "default"} className="gap-2">
              {saveStatus === "saving" ? (
                <>
                  <RefreshCw className="h-3 w-3 animate-spin" />
                  Saving...
                </>
              ) : (
                <>
                  <Check className="h-3 w-3" />
                  Saved
                </>
              )}
            </Badge>
          </div>
        )}

        <Tabs defaultValue="java" className="w-full">
          <TabsList className="grid w-full grid-cols-3 sm:grid-cols-6 mb-4">
            <TabsTrigger value="java">Java</TabsTrigger>
            <TabsTrigger value="memory">Memory</TabsTrigger>
            <TabsTrigger value="game">Game</TabsTrigger>
            <TabsTrigger value="launch">Launch</TabsTrigger>
            <TabsTrigger value="advanced">Advanced</TabsTrigger>
            <TabsTrigger value="debug">Debug</TabsTrigger>
          </TabsList>

          {/* Java Tab */}
          <TabsContent value="java" className="space-y-4">
            {/* Compatibility Alert - Always visible when incompatible */}
            {!isAuto && selectedJava && compatibility && !compatibility.compatible && (
              <Alert variant="destructive">
                <AlertTriangle className="h-4 w-4" />
                <AlertTitle>Java Incompatibility Detected</AlertTitle>
                <AlertDescription>
                  {compatibility.message}
                  <div className="mt-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => updateSetting("java_path", "")}
                    >
                      Switch to Auto-detect
                    </Button>
                  </div>
                </AlertDescription>
              </Alert>
            )}

            {/* No Compatible Java Alert */}
            {!hasAnyCompatible && detectedJavas.length > 0 && (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertTitle>No Compatible Java Found</AlertTitle>
                <AlertDescription>
                  <p className="mb-2">
                    None of your installed Java versions are compatible with Minecraft {instance.minecraft_version}.
                    You need Java {requiredJava} or compatible version.
                  </p>
                  <Button
                    size="sm"
                    onClick={() => downloadJavaVersion(requiredJava)}
                    disabled={downloadingJava !== null}
                  >
                    {downloadingJava === requiredJava ? (
                      <>
                        <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                        Downloading Java {requiredJava}...
                      </>
                    ) : (
                      <>
                        <Download className="h-4 w-4 mr-2" />
                        Download Java {requiredJava}
                      </>
                    )}
                  </Button>
                </AlertDescription>
              </Alert>
            )}

            {/* Requirements Info */}
            <Alert>
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>
                Minecraft {instance.minecraft_version} requires <strong>Java {requiredJava}</strong> or compatible version.
              </AlertDescription>
            </Alert>

            {/* Java Selection Card */}
            <Card>
              <CardHeader>
                <CardTitle>Java Selection</CardTitle>
                <CardDescription>Choose which Java installation to use for this instance</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                {/* Auto-detect Option */}
                <div
                  className={cn(
                    "flex items-center justify-between p-4 rounded-lg border cursor-pointer transition-colors",
                    isAuto ? "border-primary bg-primary/5" : "hover:bg-muted/50"
                  )}
                  onClick={() => updateSetting("java_path", "")}
                >
                  <div className="flex items-center gap-3">
                    <div className={cn(
                      "w-4 h-4 rounded-full border-2 flex items-center justify-center",
                      isAuto ? "border-primary" : "border-muted-foreground"
                    )}>
                      {isAuto && <div className="w-2 h-2 rounded-full bg-primary" />}
                    </div>
                    <div>
                      <div className="flex items-center gap-2">
                        <span className="font-medium">Auto-detect</span>
                        <Badge variant="secondary" className="text-xs">
                          <Sparkles className="h-3 w-3 mr-1" />
                          Recommended
                        </Badge>
                      </div>
                      <p className="text-sm text-muted-foreground">
                        Automatically select the best compatible Java version
                      </p>
                    </div>
                  </div>
                  {isAuto && <Check className="h-5 w-5 text-primary" />}
                </div>

                <Separator />

                {/* Detected Java List */}
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <Label>Installed Java Versions</Label>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={detectJavas}
                      disabled={detectingJava}
                    >
                      <RefreshCw className={cn("h-4 w-4 mr-2", detectingJava && "animate-spin")} />
                      Refresh
                    </Button>
                  </div>

                  {detectingJava ? (
                    <div className="flex items-center justify-center py-8">
                      <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
                      <span className="ml-2 text-muted-foreground">Detecting Java installations...</span>
                    </div>
                  ) : detectedJavas.length === 0 ? (
                    <div className="text-center py-8 space-y-3">
                      <p className="text-sm text-muted-foreground">
                        No Java installations detected on your system.
                      </p>
                      <Button
                        onClick={() => downloadJavaVersion(requiredJava)}
                        disabled={downloadingJava !== null}
                      >
                        {downloadingJava === requiredJava ? (
                          <>
                            <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                            Downloading...
                          </>
                        ) : (
                          <>
                            <Download className="h-4 w-4 mr-2" />
                            Download Java {requiredJava}
                          </>
                        )}
                      </Button>
                    </div>
                  ) : (
                    <div className="space-y-2">
                      {detectedJavas.map((java) => {
                        const compat = javaCompatibility.get(java.path);
                        const isSelected = settings.java_path === java.path;
                        
                        return (
                          <div
                            key={java.path}
                            className={cn(
                              "flex items-center justify-between p-3 rounded-lg border cursor-pointer transition-colors",
                              isSelected ? "border-primary bg-primary/5" : "hover:bg-muted/50",
                              compat && !compat.compatible && "border-destructive/50 bg-destructive/5"
                            )}
                            onClick={() => updateSetting("java_path", java.path)}
                          >
                            <div className="flex items-center gap-3">
                              <div className={cn(
                                "w-4 h-4 rounded-full border-2 flex items-center justify-center",
                                isSelected ? "border-primary" : "border-muted-foreground"
                              )}>
                                {isSelected && <div className="w-2 h-2 rounded-full bg-primary" />}
                              </div>
                              <div className="min-w-0">
                                <div className="flex items-center gap-2 flex-wrap">
                                  <span className="font-medium">Java {java.major_version}</span>
                                  <span className="text-sm text-muted-foreground">({java.version})</span>
                                  {compat && (
                                    <Badge 
                                      variant={compat.compatible ? "secondary" : "destructive"}
                                      className="text-xs"
                                    >
                                      {compat.compatible ? (
                                        <>
                                          <Check className="h-3 w-3 mr-1" />
                                          Compatible
                                        </>
                                      ) : (
                                        <>
                                          <AlertTriangle className="h-3 w-3 mr-1" />
                                          Incompatible
                                        </>
                                      )}
                                    </Badge>
                                  )}
                                </div>
                                <p className="text-xs text-muted-foreground truncate">
                                  {java.vendor} • {java.architecture} • {java.path}
                                </p>
                              </div>
                            </div>
                            {isSelected && <Check className="h-5 w-5 text-primary flex-shrink-0" />}
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              </CardContent>
            </Card>

            {/* Quick Actions Card */}
            <Card>
              <CardHeader>
                <CardTitle>Quick Actions</CardTitle>
                <CardDescription>Manage Java for this instance</CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                <Button
                  variant="outline"
                  className="w-full justify-start"
                  onClick={() => findBestJava(false)}
                  disabled={findingBestJava}
                >
                  {findingBestJava ? (
                    <>
                      <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                      Finding best Java...
                    </>
                  ) : (
                    <>
                      <Zap className="h-4 w-4 mr-2" />
                      Find & Select Best Java
                    </>
                  )}
                </Button>
                
                <Button
                  variant="outline"
                  className="w-full justify-start"
                  onClick={() => downloadJavaVersion(requiredJava)}
                  disabled={downloadingJava !== null}
                >
                  {downloadingJava === requiredJava ? (
                    <>
                      <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                      Downloading Java {requiredJava}...
                    </>
                  ) : (
                    <>
                      <Download className="h-4 w-4 mr-2" />
                      Download Java {requiredJava} (Recommended)
                    </>
                  )}
                </Button>
              </CardContent>
            </Card>

            {/* Custom Path Card */}
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
                    onChange={(e) => updateSetting("java_path", e.target.value)}
                    placeholder="Leave empty for auto-detect"
                  />
                  <p className="text-xs text-muted-foreground">
                    Use this to specify a Java installation not detected automatically. Leave empty for auto-detect.
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
                      <div className="flex items-center">
                        <Label>Minimum Memory</Label>
                        <SettingTooltip>
                          The initial amount of RAM allocated to Java. Setting this too high may slow down startup.
                        </SettingTooltip>
                      </div>
                      <span className="text-sm font-mono bg-muted px-2 py-1 rounded">{settings.memory_min_mb} MB</span>
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
                      <div className="flex items-center">
                        <Label>Maximum Memory</Label>
                        <SettingTooltip>
                          The maximum RAM Minecraft can use. Don't allocate more than 75% of your system RAM. Too much can actually hurt performance.
                        </SettingTooltip>
                      </div>
                      <span className="text-sm font-mono bg-muted px-2 py-1 rounded">{settings.memory_max_mb} MB</span>
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

                <div className="text-sm text-muted-foreground space-y-2">
                  <p className="font-medium">Recommendations:</p>
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
                <CardDescription>Custom game arguments passed to Minecraft</CardDescription>
              </CardHeader>
              <CardContent>
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
                  <div className="flex items-center">
                    <Label>Pre-Launch Command</Label>
                    <SettingTooltip>
                      A shell command that runs before Minecraft starts. Useful for scripts that need to run before the game, like backup tools or Discord presence updates.
                    </SettingTooltip>
                  </div>
                  <Input
                    value={settings.pre_launch_hook || ""}
                    onChange={(e) => updateSetting("pre_launch_hook", e.target.value || null)}
                    placeholder="Optional command to run before launch"
                    className="font-mono text-sm"
                  />
                </div>
                <div className="grid gap-2">
                  <div className="flex items-center">
                    <Label>Post-Exit Command</Label>
                    <SettingTooltip>
                      A shell command that runs after Minecraft closes. Useful for cleanup scripts or post-game actions.
                    </SettingTooltip>
                  </div>
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

          {/* Advanced Tab */}
          <TabsContent value="advanced" className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle>JVM Arguments</CardTitle>
                <CardDescription>Custom Java Virtual Machine arguments</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid gap-2">
                  <div className="flex items-center">
                    <Label>JVM Arguments</Label>
                    <SettingTooltip>
                      Advanced JVM flags for performance tuning. Common options include garbage collector settings (-XX:+UseG1GC), max heap size is set separately in Memory tab.
                    </SettingTooltip>
                  </div>
                  <Textarea
                    value={settings.java_args}
                    onChange={(e) => updateSetting("java_args", e.target.value)}
                    placeholder="-XX:+UseG1GC -XX:+UnlockExperimentalVMOptions"
                    rows={4}
                    className="font-mono text-sm"
                  />
                  <p className="text-xs text-muted-foreground">
                    Additional arguments passed to the Java Virtual Machine. Use with caution.
                  </p>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Instance Information</CardTitle>
                <CardDescription>Read-only instance details</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid gap-4">
                  <div className="flex justify-between items-center">
                    <Label className="text-muted-foreground">Instance Name</Label>
                    <span className="font-medium">{instance.name}</span>
                  </div>
                  <Separator />
                  <div className="flex justify-between items-center">
                    <Label className="text-muted-foreground">Minecraft Version</Label>
                    <span className="font-medium">{instance.minecraft_version}</span>
                  </div>
                  <Separator />
                  <div className="flex justify-between items-center">
                    <Label className="text-muted-foreground">Mod Loader</Label>
                    <span className="font-medium">
                      {instance.mod_loader}
                      {instance.mod_loader_version && ` (${instance.mod_loader_version})`}
                    </span>
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Data & Privacy</CardTitle>
                <CardDescription>Control data collection settings</CardDescription>
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
                <Separator />
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

          {/* Debug Tab */}
          <TabsContent value="debug" className="space-y-4">
            <Alert>
              <AlertCircle className="h-4 w-4" />
              <AlertTitle>Debug Mode</AlertTitle>
              <AlertDescription>
                These settings are for troubleshooting launch issues. Enable them to see Java console output and diagnose problems like Forge failing to launch.
              </AlertDescription>
            </Alert>

            <Card>
              <CardHeader>
                <CardTitle>Java Console Output</CardTitle>
                <CardDescription>
                  Control how Java runs and displays console output (Windows only)
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                  <div className="space-y-0.5">
                    <Label>Use java.exe (Show Console)</Label>
                    <p className="text-sm text-muted-foreground">
                      Force use of java.exe instead of javaw.exe to show console output in a separate window. 
                      Useful for debugging Forge/modloader launch issues.
                    </p>
                  </div>
                  <Switch
                    checked={settings.use_java_console}
                    onCheckedChange={(v) => updateSetting("use_java_console", v)}
                  />
                </div>
                <Separator />
                <div className="flex items-center justify-between">
                  <div className="space-y-0.5">
                    <Label>Disable Hidden Window Flag</Label>
                    <p className="text-sm text-muted-foreground">
                      Disable the CREATE_NO_WINDOW flag that hides the console. 
                      Combined with java.exe, this ensures console output is visible.
                    </p>
                  </div>
                  <Switch
                    checked={settings.disable_create_no_window}
                    onCheckedChange={(v) => updateSetting("disable_create_no_window", v)}
                  />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Launch Diagnostics</CardTitle>
                <CardDescription>Tools for diagnosing launch problems</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                  <div className="space-y-0.5">
                    <Label>Log Launch Command</Label>
                    <p className="text-sm text-muted-foreground">
                      Save the full launch command to a file in the instance directory (launch_command.log). 
                      Useful for manually testing launches or sharing with support.
                    </p>
                  </div>
                  <Switch
                    checked={settings.log_launch_command}
                    onCheckedChange={(v) => updateSetting("log_launch_command", v)}
                  />
                </div>
                <Separator />
                <div className="flex items-center justify-between">
                  <div className="space-y-0.5">
                    <Label>Skip Java Compatibility Check</Label>
                    <p className="text-sm text-muted-foreground">
                      Allow launching with incompatible Java versions. Use with caution - 
                      wrong Java versions may cause crashes.
                    </p>
                  </div>
                  <Switch
                    checked={settings.skip_java_compatibility_check}
                    onCheckedChange={(v) => updateSetting("skip_java_compatibility_check", v)}
                  />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Quick Debug Preset</CardTitle>
                <CardDescription>
                  Enable all debug settings at once for maximum visibility
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Button
                  variant="outline"
                  className="w-full"
                  onClick={() => {
                    updateSetting("use_java_console", true);
                    updateSetting("disable_create_no_window", true);
                    updateSetting("log_launch_command", true);
                  }}
                >
                  <AlertTriangle className="h-4 w-4 mr-2" />
                  Enable All Debug Settings
                </Button>
                <p className="text-xs text-muted-foreground mt-2">
                  This will show the Java console window and log the launch command. 
                  Remember to disable these after debugging.
                </p>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </ScrollArea>
  );
}
