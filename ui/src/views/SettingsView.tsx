import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Save } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

interface Config {
  java: {
    custom_path: string | null;
    extra_args: string;
  };
  memory: {
    min_memory: number;
    max_memory: number;
  };
}

export function SettingsView() {
  const [config, setConfig] = useState<Config | null>(null);
  const [loading, setLoading] = useState(true);
  const [saveSuccess, setSaveSuccess] = useState(false);

  useEffect(() => {
    loadConfig();
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

  const saveConfig = async () => {
    if (!config) return;
    try {
      await invoke("update_config", { config });
      setSaveSuccess(true);
    } catch (error) {
      console.error("Failed to save config:", error);
    }
  };

  if (loading) {
    return <div className="loading">Loading settings...</div>;
  }

  if (!config) {
    return <div className="error">Failed to load settings</div>;
  }

  return (
    <div className="max-w-3xl mx-auto">
      <h1 className="text-3xl font-bold mb-8">Settings</h1>

      <div className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle>Java Settings</CardTitle>
            <CardDescription>
              Configure Java runtime settings for Minecraft.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="javaPath">Custom Java Path (optional)</Label>
              <Input
                id="javaPath"
                value={config.java.custom_path || ""}
                onChange={(e) =>
                  setConfig({
                    ...config,
                    java: { ...config.java, custom_path: e.target.value || null },
                  })
                }
                placeholder="/path/to/java"
              />
              <p className="text-sm text-muted-foreground">
                Leave empty to auto-detect Java installation.
              </p>
            </div>
            <div className="space-y-2">
              <Label htmlFor="javaArgs">Extra Java Arguments</Label>
              <Input
                id="javaArgs"
                value={config.java.extra_args}
                onChange={(e) =>
                  setConfig({
                    ...config,
                    java: { ...config.java, extra_args: e.target.value },
                  })
                }
                placeholder="-XX:+UseG1GC"
              />
              <p className="text-sm text-muted-foreground">
                Additional JVM arguments passed to Minecraft.
              </p>
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
    </div>
  );
}
