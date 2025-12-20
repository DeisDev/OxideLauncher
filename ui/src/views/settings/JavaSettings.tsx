// Java settings tab for runtime management and memory configuration
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

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, Download, Trash2, Check, X, HelpCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Switch } from "@/components/ui/switch";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogCancel,
} from "@/components/ui/alert-dialog";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useSettings, extraArgsToString, stringToExtraArgs } from "./context";
import type { JavaInstallation, AvailableJavaVersion } from "./types";

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

export function JavaSettings() {
  const { config, setConfig } = useSettings();
  
  const [javaInstallations, setJavaInstallations] = useState<JavaInstallation[]>([]);
  const [detectingJava, setDetectingJava] = useState(false);
  const [availableVersions, setAvailableVersions] = useState<AvailableJavaVersion[]>([]);
  const [downloadingJava, setDownloadingJava] = useState<number | null>(null);
  const [downloadProgress, setDownloadProgress] = useState<string>("");
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);
  const [javaToDelete, setJavaToDelete] = useState<JavaInstallation | null>(null);

  useEffect(() => {
    detectJavaInstallations();
    fetchAvailableJavaVersions();
  }, []);

  const detectJavaInstallations = async () => {
    setDetectingJava(true);
    try {
      const installations = await invoke<JavaInstallation[]>("detect_java");
      setJavaInstallations(installations);
    } catch (error) {
      console.error("Failed to detect Java:", error);
    } finally {
      setDetectingJava(false);
    }
  };

  const fetchAvailableJavaVersions = async () => {
    try {
      const versions = await invoke<AvailableJavaVersion[]>("fetch_available_java_versions");
      setAvailableVersions(versions);
    } catch (error) {
      console.error("Failed to fetch available Java versions:", error);
    }
  };

  const downloadJava = async (majorVersion: number) => {
    setDownloadingJava(majorVersion);
    setDownloadProgress("Starting download...");
    
    try {
      await invoke<JavaInstallation>("download_java", { majorVersion });
      setDownloadProgress("Download complete!");
      setTimeout(() => {
        setDownloadProgress("");
        setDownloadingJava(null);
      }, 2000);
      await detectJavaInstallations();
    } catch (error) {
      console.error("Failed to download Java:", error);
      setDownloadProgress(`Error: ${error}`);
      setTimeout(() => {
        setDownloadProgress("");
        setDownloadingJava(null);
      }, 3000);
    }
  };

  const deleteJava = async (java: JavaInstallation) => {
    try {
      await invoke("delete_java", { javaPath: java.path });
      await detectJavaInstallations();
      setShowDeleteDialog(false);
      setJavaToDelete(null);
    } catch (error) {
      console.error("Failed to delete Java:", error);
      alert(`Failed to delete Java: ${error}`);
    }
  };

  const selectJava = (path: string) => {
    if (!config) return;
    setConfig({
      ...config,
      java: { ...config.java, custom_path: path || null },
    });
  };

  if (!config) return null;

  return (
    <div className="space-y-6">
      {/* Java Installations */}
      <Card>
        <CardHeader className="pb-3">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
            <div>
              <CardTitle className="text-base sm:text-lg">Java Installations</CardTitle>
              <CardDescription className="text-xs sm:text-sm">
                Manage Java runtimes for Minecraft instances.
              </CardDescription>
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={detectJavaInstallations}
              disabled={detectingJava}
              className="w-full sm:w-auto"
            >
              <RefreshCw className={`mr-2 h-4 w-4 ${detectingJava ? 'animate-spin' : ''}`} />
              {detectingJava ? "Detecting..." : "Detect Java"}
            </Button>
          </div>
        </CardHeader>
        <CardContent className="space-y-4 pt-0">
          {javaInstallations.length > 0 ? (
            <div className="space-y-2">
              <Label className="text-sm">Detected Installations</Label>
              <ScrollArea className="h-[180px] sm:h-[240px] border rounded-md">
                <div className="p-2 space-y-2">
                  {javaInstallations.map((java) => (
                    <div
                      key={java.id}
                      className={`p-2 rounded-lg border transition-colors ${
                        config?.java.custom_path === java.path
                          ? 'border-primary bg-primary/5'
                          : 'hover:bg-muted/50'
                      }`}
                    >
                      <div className="flex items-start justify-between gap-2">
                        <div className="flex-1 min-w-0 space-y-0.5">
                          <div className="flex items-center gap-1 flex-wrap">
                            <p className="font-medium text-sm">Java {java.major_version}</p>
                            {java.is_managed && (
                              <Badge variant="secondary" className="text-xs">Managed</Badge>
                            )}
                            {java.recommended && (
                              <Badge variant="default" className="text-xs hidden sm:inline-flex">Recommended</Badge>
                            )}
                            {java.is_64bit && (
                              <Badge variant="outline" className="text-xs hidden sm:inline-flex">64-bit</Badge>
                            )}
                          </div>
                          <p className="text-xs text-muted-foreground truncate">
                            {java.vendor} • {java.version}
                          </p>
                        </div>
                        <div className="flex gap-1 flex-shrink-0">
                          {config?.java.custom_path === java.path ? (
                            <Button variant="ghost" size="sm" className="h-7 w-7 p-0" onClick={() => selectJava("")}>
                              <X className="h-4 w-4" />
                            </Button>
                          ) : (
                            <Button variant="ghost" size="sm" className="h-7 w-7 p-0" onClick={() => selectJava(java.path)}>
                              <Check className="h-4 w-4" />
                            </Button>
                          )}
                          {java.is_managed && (
                            <Button
                              variant="ghost"
                              size="sm"
                              className="h-7 w-7 p-0"
                              onClick={() => {
                                setJavaToDelete(java);
                                setShowDeleteDialog(true);
                              }}
                            >
                              <Trash2 className="h-4 w-4" />
                            </Button>
                          )}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </ScrollArea>
            </div>
          ) : (
            <div className="text-center py-4 text-muted-foreground text-sm">
              {detectingJava ? "Detecting Java installations..." : "No Java installations found. Click 'Detect Java' to scan your system."}
            </div>
          )}

          {/* Download Java */}
          {availableVersions.length > 0 && (
            <div className="space-y-2 pt-3 border-t">
              <Label className="text-sm">Download Java</Label>
              <p className="text-xs text-muted-foreground">
                Download official Eclipse Temurin JDK builds.
              </p>
              <ScrollArea className="h-[140px] sm:h-[160px] border rounded-md">
                <div className="p-2 space-y-1.5">
                  {availableVersions.map((version) => (
                    <div
                      key={version.major}
                      className="flex items-center justify-between gap-2 p-2 border rounded-lg hover:bg-muted/50"
                    >
                      <div className="flex items-center gap-1.5 min-w-0">
                        <span className="font-medium text-sm">Java {version.major}</span>
                        {version.is_lts && (
                          <Badge variant="secondary" className="text-xs">LTS</Badge>
                        )}
                      </div>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => downloadJava(version.major)}
                        disabled={downloadingJava !== null}
                        className="flex-shrink-0"
                      >
                        {downloadingJava === version.major ? (
                          <>
                            <RefreshCw className="h-4 w-4 animate-spin sm:mr-2" />
                            <span className="hidden sm:inline">Downloading...</span>
                          </>
                        ) : (
                          <>
                            <Download className="h-4 w-4 sm:mr-2" />
                            <span className="hidden sm:inline">Download</span>
                          </>
                        )}
                      </Button>
                    </div>
                  ))}
                </div>
              </ScrollArea>
              {downloadProgress && (
                <p className="text-xs text-muted-foreground">{downloadProgress}</p>
              )}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Java Configuration */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-base sm:text-lg">Java Configuration</CardTitle>
          <CardDescription className="text-xs sm:text-sm">
            Configure Java path and JVM arguments.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3 pt-0">
          <div className="space-y-1.5">
            <Label htmlFor="javaPath" className="inline-flex items-center text-sm">
              Custom Java Path (optional)
              <SettingTooltip>
                Specify a custom Java executable path. This overrides automatic Java detection. Useful for using a specific Java version.
              </SettingTooltip>
            </Label>
            <Input
              id="javaPath"
              value={config?.java.custom_path || ""}
              onChange={(e) =>
                setConfig({
                  ...config!,
                  java: { ...config!.java, custom_path: e.target.value || null },
                })
              }
              placeholder="/path/to/java"
              className="h-9"
            />
            <p className="text-xs text-muted-foreground">
              Leave empty to auto-detect, or select from the list above.
            </p>
          </div>
          
          <div className="space-y-1.5">
            <Label htmlFor="javaArgs" className="inline-flex items-center text-sm">
              Extra Java Arguments
              <SettingTooltip>
                Advanced JVM flags for performance tuning. Common options: -XX:+UseG1GC for garbage collection, -XX:+UnlockExperimentalVMOptions for experimental features.
              </SettingTooltip>
            </Label>
            <Input
              id="javaArgs"
              value={extraArgsToString(config?.java.extra_args ?? [])}
              onChange={(e) =>
                setConfig({
                  ...config!,
                  java: { ...config!.java, extra_args: stringToExtraArgs(e.target.value) },
                })
              }
              placeholder="-XX:+UseG1GC -XX:+ParallelRefProcEnabled"
              className="h-9"
            />
            <p className="text-xs text-muted-foreground">
              Additional JVM arguments. Separate multiple with spaces.
            </p>
          </div>
        </CardContent>
      </Card>

      {/* Memory Settings */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-base sm:text-lg">Memory Settings</CardTitle>
          <CardDescription className="text-xs sm:text-sm">
            Configure how much RAM Minecraft can use.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3 pt-0">
          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <Label htmlFor="minMemory" className="inline-flex items-center text-sm">
                Min (MB)
                <SettingTooltip>
                  Initial heap size for Java. Setting this equal to maximum memory can improve startup time.
                </SettingTooltip>
              </Label>
              <Input
                id="minMemory"
                type="number"
                value={config.memory.min_memory}
                onChange={(e) =>
                  setConfig({
                    ...config,
                    memory: {
                      ...config.memory,
                      min_memory: parseInt(e.target.value) || 512,
                    },
                  })
                }
                min="512"
                max="32768"
                className="h-9"
              />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="maxMemory" className="inline-flex items-center text-sm">
                Max (MB)
                <SettingTooltip>
                  Maximum heap size for Java. Too little causes lag, too much can cause stuttering. Recommended: 4-8GB for modded, 2-4GB for vanilla.
                </SettingTooltip>
              </Label>
              <Input
                id="maxMemory"
                type="number"
                value={config.memory.max_memory}
                onChange={(e) =>
                  setConfig({
                    ...config,
                    memory: {
                      ...config.memory,
                      max_memory: parseInt(e.target.value) || 4096,
                    },
                  })
                }
                min="1024"
                max="32768"
                className="h-9"
              />
            </div>
          </div>
          <p className="text-xs text-muted-foreground">
            Recommended: Set max to half of your system RAM.
          </p>
        </CardContent>
      </Card>

      {/* Java Behavior */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-base sm:text-lg">Java Behavior</CardTitle>
          <CardDescription className="text-xs sm:text-sm">
            Configure automatic Java management.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3 pt-0">
          <div className="flex items-center justify-between gap-3">
            <div className="space-y-0.5 min-w-0 flex-1">
              <Label htmlFor="autoDownloadJava" className="text-sm">Auto-Download Java</Label>
              <p className="text-xs text-muted-foreground">
                Auto-download required Java versions when launching.
              </p>
            </div>
            <Switch
              id="autoDownloadJava"
              checked={config?.java.auto_download ?? true}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config!,
                  java: { ...config!.java, auto_download: checked },
                })
              }
            />
          </div>
          <div className="flex items-center justify-between gap-3">
            <div className="space-y-0.5 min-w-0 flex-1">
              <Label htmlFor="skipCompatibilityCheck" className="text-sm">Skip Compatibility Check</Label>
              <p className="text-xs text-muted-foreground">
                Don't verify Java versions match Minecraft requirements.
              </p>
            </div>
            <Switch
              id="skipCompatibilityCheck"
              checked={config?.java.skip_compatibility_check ?? false}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config!,
                  java: { ...config!.java, skip_compatibility_check: checked },
                })
              }
            />
          </div>
        </CardContent>
      </Card>

      {/* Delete Java Dialog */}
      <AlertDialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Java Installation</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete Java {javaToDelete?.major_version} ({javaToDelete?.vendor})?
              This will remove the installation from your system.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={() => setJavaToDelete(null)}>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={() => javaToDelete && deleteJava(javaToDelete)}>
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
