// Advanced settings tab for logging and debugging options
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

import { invoke } from "@tauri-apps/api/core";
import { FolderOpen } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useSettings } from "./context";

// Logging Settings Sub-tab
function LoggingSettings() {
  const { config, setConfig } = useSettings();
  if (!config) return null;

  const openLogsDirectory = async () => {
    try {
      await invoke("open_logs_directory");
    } catch (error) {
      console.error("Failed to open logs directory:", error);
    }
  };

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Debug Logging</CardTitle>
          <CardDescription>
            Configure logging for troubleshooting.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="debugLogging">Enable Debug Logging</Label>
              <p className="text-sm text-muted-foreground">
                Write detailed debug logs to a file for troubleshooting issues.
              </p>
            </div>
            <Switch
              id="debugLogging"
              checked={config.logging.debug_to_file}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config,
                  logging: { ...config.logging, debug_to_file: checked },
                })
              }
            />
          </div>

          <div className="grid grid-cols-2 gap-4 pt-4">
            <div className="space-y-2">
              <Label htmlFor="maxLogSize">Max Log Size (MB)</Label>
              <Input
                id="maxLogSize"
                type="number"
                value={config.logging.max_file_size_mb}
                onChange={(e) =>
                  setConfig({
                    ...config,
                    logging: {
                      ...config.logging,
                      max_file_size_mb: parseInt(e.target.value) || 10,
                    },
                  })
                }
                min="1"
                max="100"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="maxLogFiles">Max Log Files</Label>
              <Input
                id="maxLogFiles"
                type="number"
                value={config.logging.max_files}
                onChange={(e) =>
                  setConfig({
                    ...config,
                    logging: {
                      ...config.logging,
                      max_files: parseInt(e.target.value) || 5,
                    },
                  })
                }
                min="1"
                max="30"
              />
            </div>
          </div>

          <Button variant="outline" size="sm" onClick={openLogsDirectory}>
            <FolderOpen className="mr-2 h-4 w-4" />
            Open Logs Folder
          </Button>

          <p className="text-xs text-muted-foreground">
            Note: Requires app restart to take effect. Logs are rotated and kept for {config.logging.max_files} files.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}

// API Keys Settings Sub-tab
function ApiKeysSettings() {
  const { config, setConfig } = useSettings();
  if (!config) return null;

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>API Keys</CardTitle>
          <CardDescription>
            Configure API keys for third-party services. These are optional and only needed for specific features.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="msaClientId">Microsoft Azure Client ID</Label>
            <Input
              id="msaClientId"
              type="password"
              value={config.api_keys.msa_client_id || ""}
              onChange={(e) =>
                setConfig({
                  ...config,
                  api_keys: { ...config.api_keys, msa_client_id: e.target.value || null },
                })
              }
              placeholder="Custom Microsoft Azure Client ID"
            />
            <p className="text-sm text-muted-foreground">
              Override the default Azure Client ID for Microsoft authentication.
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="curseforgeKey">CurseForge API Key</Label>
            <Input
              id="curseforgeKey"
              type="password"
              value={config.api_keys.curseforge_api_key || ""}
              onChange={(e) =>
                setConfig({
                  ...config,
                  api_keys: { ...config.api_keys, curseforge_api_key: e.target.value || null },
                })
              }
              placeholder="Your CurseForge API key"
            />
            <p className="text-sm text-muted-foreground">
              Required for downloading CurseForge mods and modpacks.
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="modrinthToken">Modrinth API Token</Label>
            <Input
              id="modrinthToken"
              type="password"
              value={config.api_keys.modrinth_api_token || ""}
              onChange={(e) =>
                setConfig({
                  ...config,
                  api_keys: { ...config.api_keys, modrinth_api_token: e.target.value || null },
                })
              }
              placeholder="Your Modrinth API token"
            />
            <p className="text-sm text-muted-foreground">
              Optional token for authenticated Modrinth API requests.
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// Custom Commands Settings Sub-tab
function CustomCommandsSettings() {
  const { config, setConfig } = useSettings();
  if (!config) return null;

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Custom Commands</CardTitle>
          <CardDescription>
            Run custom commands at different stages of the launch process.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="preLaunch">Pre-Launch Command</Label>
            <Textarea
              id="preLaunch"
              value={config.commands.pre_launch || ""}
              onChange={(e) =>
                setConfig({
                  ...config,
                  commands: { ...config.commands, pre_launch: e.target.value || null },
                })
              }
              placeholder="Command to run before launching Minecraft"
              rows={2}
            />
            <p className="text-sm text-muted-foreground">
              Runs before Minecraft starts. Available variables: $INST_NAME, $INST_ID, $INST_DIR
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="wrapperCommand">Wrapper Command</Label>
            <Textarea
              id="wrapperCommand"
              value={config.commands.wrapper_command || ""}
              onChange={(e) =>
                setConfig({
                  ...config,
                  commands: { ...config.commands, wrapper_command: e.target.value || null },
                })
              }
              placeholder="e.g., gamemoderun, mangohud"
              rows={2}
            />
            <p className="text-sm text-muted-foreground">
              Wraps the Minecraft launch command. The game command will be appended to this.
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="postExit">Post-Exit Command</Label>
            <Textarea
              id="postExit"
              value={config.commands.post_exit || ""}
              onChange={(e) =>
                setConfig({
                  ...config,
                  commands: { ...config.commands, post_exit: e.target.value || null },
                })
              }
              placeholder="Command to run after Minecraft closes"
              rows={2}
            />
            <p className="text-sm text-muted-foreground">
              Runs after Minecraft exits. Available variables: $INST_NAME, $INST_ID, $INST_DIR
            </p>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Variable Reference</CardTitle>
          <CardDescription>
            Variables available in custom commands.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 gap-2 text-sm">
            <code className="bg-muted px-2 py-1 rounded">$INST_NAME</code>
            <span className="text-muted-foreground">Instance name</span>
            <code className="bg-muted px-2 py-1 rounded">$INST_ID</code>
            <span className="text-muted-foreground">Instance ID</span>
            <code className="bg-muted px-2 py-1 rounded">$INST_DIR</code>
            <span className="text-muted-foreground">Instance directory</span>
            <code className="bg-muted px-2 py-1 rounded">$INST_MC_DIR</code>
            <span className="text-muted-foreground">Minecraft directory</span>
            <code className="bg-muted px-2 py-1 rounded">$INST_JAVA</code>
            <span className="text-muted-foreground">Java executable path</span>
            <code className="bg-muted px-2 py-1 rounded">$INST_JAVA_ARGS</code>
            <span className="text-muted-foreground">Java arguments</span>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// Network Settings Sub-tab
function NetworkSettings() {
  const { config, setConfig } = useSettings();
  if (!config) return null;

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>User Agent</CardTitle>
          <CardDescription>
            Configure the HTTP User-Agent header sent with requests.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="userAgent">Custom User Agent</Label>
            <Input
              id="userAgent"
              value={config.network.user_agent}
              onChange={(e) =>
                setConfig({
                  ...config,
                  network: { ...config.network, user_agent: e.target.value },
                })
              }
              placeholder="OxideLauncher/1.0.0"
            />
            <p className="text-sm text-muted-foreground">
              The User-Agent string sent with HTTP requests. Some APIs may require a specific format.
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// Main Advanced Settings Component
export function AdvancedSettings() {
  return (
    <Tabs defaultValue="logging" className="w-full">
      <TabsList className="mb-4">
        <TabsTrigger value="logging">Logging</TabsTrigger>
        <TabsTrigger value="api">API Keys</TabsTrigger>
        <TabsTrigger value="commands">Commands</TabsTrigger>
        <TabsTrigger value="network">Network</TabsTrigger>
      </TabsList>

      <TabsContent value="logging">
        <LoggingSettings />
      </TabsContent>

      <TabsContent value="api">
        <ApiKeysSettings />
      </TabsContent>

      <TabsContent value="commands">
        <CustomCommandsSettings />
      </TabsContent>

      <TabsContent value="network">
        <NetworkSettings />
      </TabsContent>
    </Tabs>
  );
}
