import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Save, RefreshCw, Download, Trash2, Check, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
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

import { Switch } from "@/components/ui/switch";

interface Config {
  data_dir: string;
  instances_dir: string | null;
  theme: string;
  java: {
    custom_path: string | null;
    use_bundled: boolean;
    auto_detect: boolean;
    extra_args: string[];
    skip_compatibility_check: boolean;
    auto_download: boolean;
  };
  network: {
    proxy: null;
    max_concurrent_downloads: number;
    timeout_seconds: number;
    user_agent: string;
  };
  ui: {
    show_news: boolean;
    instance_view: string;
    window_width: number;
    window_height: number;
    last_instance: string | null;
    cat_mode: boolean;
  };
  memory: {
    min_memory: number;
    max_memory: number;
    permgen: number;
  };
  logging: {
    debug_to_file: boolean;
    max_file_size_mb: number;
    max_files: number;
  };
  api_keys: {
    msa_client_id: string | null;
    curseforge_api_key: string | null;
    modrinth_api_token: string | null;
  };
}

// Helper to convert extra_args array to string for display
function extraArgsToString(args: string[]): string {
  return args.join(' ');
}

// Helper to convert extra_args string to array for storage
function stringToExtraArgs(str: string): string[] {
  if (!str.trim()) return [];
  return str.split(/\s+/).filter(arg => arg.length > 0);
}

interface JavaInstallation {
  id: string;
  path: string;
  version: string;
  major_version: number;
  arch: string;
  vendor: string;
  is_64bit: boolean;
  is_managed: boolean;
  recommended: boolean;
}

interface AvailableJavaVersion {
  major: number;
  name: string;
  is_lts: boolean;
}

export function SettingsView() {
  const [config, setConfig] = useState<Config | null>(null);
  const [loading, setLoading] = useState(true);
  const [saveSuccess, setSaveSuccess] = useState(false);
  
  // Java management state
  const [javaInstallations, setJavaInstallations] = useState<JavaInstallation[]>([]);
  const [detectingJava, setDetectingJava] = useState(false);
  const [availableVersions, setAvailableVersions] = useState<AvailableJavaVersion[]>([]);
  const [downloadingJava, setDownloadingJava] = useState<number | null>(null);
  const [downloadProgress, setDownloadProgress] = useState<string>("");
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);
  const [javaToDelete, setJavaToDelete] = useState<JavaInstallation | null>(null);

  useEffect(() => {
    loadConfig();
    detectJavaInstallations();
    fetchAvailableJavaVersions();
  }, []);

  const loadConfig = async () => {
    try {
      const data = await invoke<Config>("get_config");
      setConfig(data);
    } catch (error) {
      console.error("Failed to load config:", error);
    } finally {
      setLoading(false);
    }
  };
  
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
      await invoke<JavaInstallation>("download_java", {
        majorVersion,
      });
      
      setDownloadProgress("Download complete!");
      setTimeout(() => {
        setDownloadProgress("");
        setDownloadingJava(null);
      }, 2000);
      
      // Refresh installations
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
      java: { ...config.java, custom_path: path },
    });
  };

  const saveConfig = async () => {
    if (!config) return;
    try {
      await invoke("update_config", { config });
      setSaveSuccess(true);
    } catch (error) {
      console.error("Failed to save config:", error);
    }
  };

  // Skeleton loading component that maintains layout consistency
  const LoadingSkeleton = () => (
    <div className="max-w-3xl mx-auto">
      <div className="skeleton h-9 w-32 mb-8" />
      <div className="space-y-6">
        <Card>
          <CardHeader>
            <div className="skeleton h-6 w-40" />
            <div className="skeleton h-4 w-64 mt-2" />
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="skeleton h-[300px]" />
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <div className="skeleton h-6 w-40" />
            <div className="skeleton h-4 w-64 mt-2" />
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="skeleton h-10" />
              <div className="skeleton h-10" />
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );

  if (loading) {
    return <LoadingSkeleton />;
  }

  if (!config) {
    return <div className="error">Failed to load settings</div>;
  }

  return (
    <div className="max-w-3xl mx-auto">
      <h1 className="text-3xl font-bold mb-8">Settings</h1>

      <div className="space-y-6">
        {/* Java Management */}
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle>Java Installations</CardTitle>
                <CardDescription>
                  Manage Java runtimes for Minecraft instances.
                </CardDescription>
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={detectJavaInstallations}
                disabled={detectingJava}
              >
                <RefreshCw className={`mr-2 h-4 w-4 ${detectingJava ? 'animate-spin' : ''}`} />
                {detectingJava ? "Detecting..." : "Detect Java"}
              </Button>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Detected Java Installations */}
            {javaInstallations.length > 0 ? (
              <div className="space-y-2">
                <Label>Detected Installations</Label>
                <ScrollArea className="h-[300px] border rounded-md">
                  <div className="p-4 space-y-2">
                    {javaInstallations.map((java) => (
                      <div
                        key={java.id}
                        className={`p-3 rounded-lg border transition-colors ${
                          config?.java.custom_path === java.path
                            ? 'border-primary bg-primary/5'
                            : 'hover:bg-muted/50'
                        }`}
                      >
                        <div className="flex items-start justify-between">
                          <div className="flex-1 min-w-0 space-y-1">
                            <div className="flex items-center gap-2">
                              <p className="font-medium">
                                Java {java.major_version}
                              </p>
                              {java.is_managed && (
                                <Badge variant="secondary" className="text-xs">
                                  Managed
                                </Badge>
                              )}
                              {java.recommended && (
                                <Badge variant="default" className="text-xs">
                                  Recommended
                                </Badge>
                              )}
                              {java.is_64bit && (
                                <Badge variant="outline" className="text-xs">
                                  64-bit
                                </Badge>
                              )}
                            </div>
                            <p className="text-sm text-muted-foreground">
                              {java.vendor} • {java.version}
                            </p>
                            <p className="text-xs text-muted-foreground truncate">
                              {java.path}
                            </p>
                          </div>
                          <div className="flex gap-1 ml-2">
                            {config?.java.custom_path === java.path ? (
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => selectJava("")}
                              >
                                <X className="h-4 w-4" />
                              </Button>
                            ) : (
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => selectJava(java.path)}
                              >
                                <Check className="h-4 w-4" />
                              </Button>
                            )}
                            {java.is_managed && (
                              <Button
                                variant="ghost"
                                size="sm"
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
              <div className="text-center py-8 text-muted-foreground">
                {detectingJava ? "Detecting Java installations..." : "No Java installations found. Click 'Detect Java' to scan your system."}
              </div>
            )}

            {/* Download Java */}
            {availableVersions.length > 0 && (
              <div className="space-y-2 pt-4 border-t">
                <Label>Download Java</Label>
                <p className="text-sm text-muted-foreground">
                  Download official Eclipse Temurin JDK builds.
                </p>
                <ScrollArea className="h-[200px] border rounded-md">
                  <div className="p-3 space-y-2">
                    {availableVersions.map((version) => (
                      <div
                        key={version.major}
                        className="flex items-center justify-between p-2 border rounded-lg hover:bg-muted/50"
                      >
                        <div className="flex items-center gap-2">
                          <span className="font-medium text-sm">Java {version.major}</span>
                          {version.is_lts && (
                            <Badge variant="secondary" className="text-xs">
                              LTS
                            </Badge>
                          )}
                        </div>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => downloadJava(version.major)}
                          disabled={downloadingJava !== null}
                        >
                          {downloadingJava === version.major ? (
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
                </ScrollArea>
                {downloadProgress && (
                  <p className="text-sm text-muted-foreground mt-2">
                    {downloadProgress}
                  </p>
                )}
              </div>
            )}

            {/* Custom Java Path */}
            <div className="space-y-2 pt-4 border-t">
              <Label htmlFor="javaPath">Custom Java Path (optional)</Label>
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
              />
              <p className="text-sm text-muted-foreground">
                Leave empty to auto-detect Java installation, or select one from the list above.
              </p>
            </div>
            
            {/* Extra Java Arguments */}
            <div className="space-y-2">
              <Label htmlFor="javaArgs">Extra Java Arguments</Label>
              <Input
                id="javaArgs"
                value={extraArgsToString(config?.java.extra_args ?? [])}
                onChange={(e) =>
                  setConfig({
                    ...config!,
                    java: { ...config!.java, extra_args: stringToExtraArgs(e.target.value) },
                  })
                }
                placeholder="-XX:+UseG1GC"
              />
              <p className="text-sm text-muted-foreground">
                Additional JVM arguments passed to Minecraft. Separate multiple arguments with spaces.
              </p>
            </div>
            
            {/* Java Behavior Settings */}
            <div className="space-y-4 pt-4 border-t">
              <Label className="text-base font-medium">Java Behavior</Label>
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label htmlFor="autoDownloadJava">Auto-Download Java</Label>
                  <p className="text-sm text-muted-foreground">
                    Automatically download required Java versions when launching instances.
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
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label htmlFor="skipCompatibilityCheck">Skip Java Compatibility Check</Label>
                  <p className="text-sm text-muted-foreground">
                    Don't verify that Java versions match Minecraft requirements.
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
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Memory Settings</CardTitle>
            <CardDescription>
              Configure how much RAM Minecraft can use.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="minMemory">Minimum Memory (MB)</Label>
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
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="maxMemory">Maximum Memory (MB)</Label>
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
                />
              </div>
            </div>
            <p className="text-sm text-muted-foreground">
              Recommended: Set maximum memory to half of your system RAM.
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Logging & Debugging</CardTitle>
            <CardDescription>
              Configure debug logging for troubleshooting.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label htmlFor="debugLogging">Enable Debug Logging to File</Label>
                <p className="text-sm text-muted-foreground">
                  Write detailed debug logs to a file. Useful for troubleshooting issues.
                </p>
              </div>
              <Switch
                id="debugLogging"
                checked={config?.logging.debug_to_file ?? false}
                onCheckedChange={(checked) =>
                  setConfig({
                    ...config!,
                    logging: { ...config!.logging, debug_to_file: checked },
                  })
                }
              />
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={async () => {
                try {
                  await invoke("open_logs_directory");
                } catch (error) {
                  console.error("Failed to open logs directory:", error);
                }
              }}
            >
              Open Logs Folder
            </Button>
            <p className="text-xs text-muted-foreground">
              Note: Requires app restart to take effect. Logs are rotated daily and kept for {config?.logging.max_files ?? 5} days.
            </p>
          </CardContent>
        </Card>

        <div className="flex justify-end pt-4">
          <Button onClick={saveConfig} size="lg">
            <Save className="mr-2 h-4 w-4" /> Save Settings
          </Button>
        </div>
      </div>

      {/* Save Success Dialog */}
      <AlertDialog open={saveSuccess} onOpenChange={setSaveSuccess}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Settings Saved</AlertDialogTitle>
            <AlertDialogDescription>
              Your settings have been saved successfully.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogAction>OK</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Delete Java Confirmation Dialog */}
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
