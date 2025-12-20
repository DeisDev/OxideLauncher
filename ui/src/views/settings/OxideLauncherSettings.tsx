// General launcher settings tab for appearance, paths, and behavior
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

import { FolderOpen, Cog, Monitor } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
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
import { useTheme } from "@/hooks/useTheme";
import type { InstanceViewMode } from "./types";

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

// Color scheme options
const COLOR_SCHEMES = [
  { value: "ocean", label: "Ocean (Blue)" },
  { value: "emerald", label: "Emerald (Green)" },
  { value: "forest", label: "Forest (Teal)" },
  { value: "sunset", label: "Sunset (Orange)" },
  { value: "lavender", label: "Lavender (Purple)" },
  { value: "crimson", label: "Crimson (Red)" },
];

// Instance sort options
const SORT_OPTIONS = [
  { value: "name", label: "Name" },
  { value: "last_played", label: "Last Played" },
  { value: "date_created", label: "Date Created" },
  { value: "minecraft_version", label: "Minecraft Version" },
];

// Grid size options
const GRID_SIZES = [
  { value: "small", label: "Small" },
  { value: "medium", label: "Medium" },
  { value: "large", label: "Large" },
];

// General Settings Sub-tab
function GeneralSettings() {
  const { config, setConfig } = useSettings();
  if (!config) return null;

  const openDataDirectory = async () => {
    try {
      await invoke("open_data_directory");
    } catch (error) {
      console.error("Failed to open data directory:", error);
    }
  };

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Directories</CardTitle>
          <CardDescription>
            Configure where Oxide Launcher stores its data.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="dataDir">Data Directory</Label>
            <div className="flex gap-2">
              <Input
                id="dataDir"
                value={config.data_dir}
                readOnly
                className="flex-1"
              />
              <Button variant="outline" size="icon" onClick={openDataDirectory}>
                <FolderOpen className="h-4 w-4" />
              </Button>
            </div>
            <p className="text-sm text-muted-foreground">
              Root directory for all launcher data including instances, libraries, and assets.
            </p>
          </div>
          
          <div className="space-y-2">
            <Label htmlFor="instancesDir">Instances Directory (optional)</Label>
            <Input
              id="instancesDir"
              value={config.instances_dir || ""}
              onChange={(e) =>
                setConfig({
                  ...config,
                  instances_dir: e.target.value || null,
                })
              }
              placeholder={`${config.data_dir}/instances`}
            />
            <p className="text-sm text-muted-foreground">
              Custom directory for Minecraft instances. Leave empty to use default.
            </p>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Launcher Behavior</CardTitle>
          <CardDescription>
            General launcher settings.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="showNews">Show News</Label>
              <p className="text-sm text-muted-foreground">
                Display Minecraft and mod news on the home page.
              </p>
            </div>
            <Switch
              id="showNews"
              checked={config.ui.show_news}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config,
                  ui: { ...config.ui, show_news: checked },
                })
              }
            />
          </div>
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="openInstanceAfterInstall">Open Instance After Install</Label>
              <p className="text-sm text-muted-foreground">
                Automatically open the instance details view after installing a modpack.
              </p>
            </div>
            <Switch
              id="openInstanceAfterInstall"
              checked={config.ui.open_instance_after_install}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config,
                  ui: { ...config.ui, open_instance_after_install: checked },
                })
              }
            />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// Instance Settings Sub-tab
function InstanceSettings() {
  const { config, setConfig } = useSettings();
  if (!config) return null;

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Instance Display</CardTitle>
          <CardDescription>
            Configure how instances are displayed in the launcher.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="viewMode">View Mode</Label>
              <Select
                value={config.ui.instance_view}
                onValueChange={(value: InstanceViewMode) =>
                  setConfig({
                    ...config,
                    ui: { ...config.ui, instance_view: value },
                  })
                }
              >
                <SelectTrigger id="viewMode">
                  <SelectValue placeholder="Select view" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Grid">Grid</SelectItem>
                  <SelectItem value="List">List</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <Label htmlFor="gridSize">Grid Size</Label>
              <Select
                value={config.ui.instance_grid_size}
                onValueChange={(value) =>
                  setConfig({
                    ...config,
                    ui: { ...config.ui, instance_grid_size: value },
                  })
                }
              >
                <SelectTrigger id="gridSize">
                  <SelectValue placeholder="Select size" />
                </SelectTrigger>
                <SelectContent>
                  {GRID_SIZES.map((size) => (
                    <SelectItem key={size.value} value={size.value}>
                      {size.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="sortBy">Sort By</Label>
              <Select
                value={config.ui.instance_sort_by}
                onValueChange={(value) =>
                  setConfig({
                    ...config,
                    ui: { ...config.ui, instance_sort_by: value },
                  })
                }
              >
                <SelectTrigger id="sortBy">
                  <SelectValue placeholder="Select sort" />
                </SelectTrigger>
                <SelectContent>
                  {SORT_OPTIONS.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <Label htmlFor="sortDirection">Sort Direction</Label>
              <Select
                value={config.ui.instance_sort_asc ? "asc" : "desc"}
                onValueChange={(value) =>
                  setConfig({
                    ...config,
                    ui: { ...config.ui, instance_sort_asc: value === "asc" },
                  })
                }
              >
                <SelectTrigger id="sortDirection">
                  <SelectValue placeholder="Select direction" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="asc">Ascending (A-Z)</SelectItem>
                  <SelectItem value="desc">Descending (Z-A)</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Game Time</CardTitle>
          <CardDescription>
            Configure play time tracking display.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="showGameTime">Show Game Time</Label>
              <p className="text-sm text-muted-foreground">
                Display total play time on instance cards.
              </p>
            </div>
            <Switch
              id="showGameTime"
              checked={config.minecraft.show_game_time}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config,
                  minecraft: { ...config.minecraft, show_game_time: checked },
                })
              }
            />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// Window Settings Sub-tab
function WindowSettings() {
  const { config, setConfig } = useSettings();
  if (!config) return null;

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Monitor className="h-5 w-5" />
            Window Position Memory
          </CardTitle>
          <CardDescription>
            Remember window positions and sizes across sessions.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="rememberMainWindow" className="inline-flex items-center">
                Remember Main Window Position
                <SettingTooltip>
                  Save and restore the main launcher window position and size when you restart the launcher.
                </SettingTooltip>
              </Label>
              <p className="text-sm text-muted-foreground">
                The launcher will open in the same position and size as when you closed it.
              </p>
            </div>
            <Switch
              id="rememberMainWindow"
              checked={config.ui.remember_main_window_position}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config,
                  ui: { ...config.ui, remember_main_window_position: checked },
                })
              }
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="rememberDialogWindows" className="inline-flex items-center">
                Remember Dialog Window Positions
                <SettingTooltip>
                  Save and restore positions for dialog windows like the mod browser, modpack browser, etc.
                </SettingTooltip>
              </Label>
              <p className="text-sm text-muted-foreground">
                Browser windows will remember their last position and size.
              </p>
            </div>
            <Switch
              id="rememberDialogWindows"
              checked={config.ui.remember_dialog_window_positions}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config,
                  ui: { ...config.ui, remember_dialog_window_positions: checked },
                })
              }
            />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// Appearance Settings Sub-tab
function AppearanceSettings() {
  const { config, setConfig } = useSettings();
  const { setTheme, setColorScheme, setRustMode } = useTheme();
  
  if (!config) return null;

  const handleThemeChange = (value: string) => {
    setConfig({
      ...config,
      theme: value,
    });
    setTheme(value); // Apply immediately
  };

  const handleColorSchemeChange = (value: string) => {
    setConfig({
      ...config,
      ui: { ...config.ui, color_scheme: value },
    });
    setColorScheme(value); // Apply immediately
  };

  const handleRustModeChange = (checked: boolean) => {
    setConfig({
      ...config,
      ui: { ...config.ui, rust_mode: checked },
    });
    setRustMode(checked); // Apply immediately
  };

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Theme</CardTitle>
          <CardDescription>
            Customize the look and feel of the launcher.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="theme">Theme Mode</Label>
              <Select
                value={config.theme}
                onValueChange={handleThemeChange}
              >
                <SelectTrigger id="theme">
                  <SelectValue placeholder="Select theme" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="dark">Dark</SelectItem>
                  <SelectItem value="light">Light</SelectItem>
                  <SelectItem value="system">System</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <Label htmlFor="colorScheme" className="inline-flex items-center">
                Color Scheme
                <SettingTooltip>
                  Changes the accent color throughout the launcher. Try different schemes to find your favorite!
                </SettingTooltip>
              </Label>
              <Select
                value={config.ui.color_scheme}
                onValueChange={handleColorSchemeChange}
              >
                <SelectTrigger id="colorScheme">
                  <SelectValue placeholder="Select scheme" />
                </SelectTrigger>
                <SelectContent>
                  {COLOR_SCHEMES.map((scheme) => (
                    <SelectItem key={scheme.value} value={scheme.value}>
                      {scheme.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Fun Stuff</CardTitle>
          <CardDescription>
            Easter eggs and fun features.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Cog className="h-5 w-5 text-orange-500" />
              <div className="space-y-0.5">
                <Label htmlFor="rustMode" className="inline-flex items-center">
                  Rust Mode
                  <SettingTooltip>
                    Activates an obnoxiously Rust-themed color scheme with orange and ferris crab vibes. Memory safe AND stylish!
                  </SettingTooltip>
                </Label>
                <p className="text-sm text-muted-foreground">
                  Because everything is better when it's <span className="text-orange-500 font-semibold">blazingly fast</span>. 🦀
                </p>
              </div>
            </div>
            <Switch
              id="rustMode"
              checked={config.ui.rust_mode}
              onCheckedChange={handleRustModeChange}
            />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// Debug Settings Sub-tab
function DebugSettings() {
  const { config, setConfig } = useSettings();
  if (!config) return null;

  const openLogsFolder = async () => {
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
          <CardTitle>Debug Information</CardTitle>
          <CardDescription>
            These settings apply globally to all instances. Individual instances can override these in their own settings.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-2">
            <Button variant="outline" onClick={openLogsFolder}>
              <FolderOpen className="h-4 w-4 mr-2" />
              Open Logs Folder
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Java Console Output</CardTitle>
          <CardDescription>
            Control how Java runs and displays console output globally (Windows only)
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="forceJavaConsole" className="inline-flex items-center">
                Force java.exe Console
                <SettingTooltip>
                  Uses java.exe instead of javaw.exe for all instances, showing console output in a separate window. Useful for debugging modloader issues.
                </SettingTooltip>
              </Label>
              <p className="text-sm text-muted-foreground">
                Always show Java console output (uses java.exe instead of javaw.exe)
              </p>
            </div>
            <Switch
              id="forceJavaConsole"
              checked={config.debug.force_java_console}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config,
                  debug: { ...config.debug, force_java_console: checked },
                })
              }
            />
          </div>
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="disableCreateNoWindow" className="inline-flex items-center">
                Disable Hidden Window Flag
                <SettingTooltip>
                  Disables the CREATE_NO_WINDOW flag that normally hides the console. Combined with java.exe, ensures console is visible.
                </SettingTooltip>
              </Label>
              <p className="text-sm text-muted-foreground">
                Allow console windows to appear (disable CREATE_NO_WINDOW)
              </p>
            </div>
            <Switch
              id="disableCreateNoWindow"
              checked={config.debug.disable_create_no_window}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config,
                  debug: { ...config.debug, disable_create_no_window: checked },
                })
              }
            />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Launch Diagnostics</CardTitle>
          <CardDescription>
            Tools for diagnosing launch and runtime issues
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="logLaunchCommands" className="inline-flex items-center">
                Log Launch Commands
                <SettingTooltip>
                  Saves the full Java launch command to launch_command.log in each instance's directory. Helpful for manually testing or debugging launches.
                </SettingTooltip>
              </Label>
              <p className="text-sm text-muted-foreground">
                Save launch commands to file for all instances
              </p>
            </div>
            <Switch
              id="logLaunchCommands"
              checked={config.debug.log_launch_commands}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config,
                  debug: { ...config.debug, log_launch_commands: checked },
                })
              }
            />
          </div>
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="verboseLogging" className="inline-flex items-center">
                Verbose Logging
                <SettingTooltip>
                  Enables more detailed logging throughout the application for debugging purposes.
                </SettingTooltip>
              </Label>
              <p className="text-sm text-muted-foreground">
                Enable detailed debug logging
              </p>
            </div>
            <Switch
              id="verboseLogging"
              checked={config.debug.verbose_logging}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config,
                  debug: { ...config.debug, verbose_logging: checked },
                })
              }
            />
          </div>
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="keepNatives" className="inline-flex items-center">
                Keep Natives After Launch
                <SettingTooltip>
                  Don't clean up the natives directory after launching. Useful for debugging native library issues.
                </SettingTooltip>
              </Label>
              <p className="text-sm text-muted-foreground">
                Preserve natives directory for inspection
              </p>
            </div>
            <Switch
              id="keepNatives"
              checked={config.debug.keep_natives_after_launch}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config,
                  debug: { ...config.debug, keep_natives_after_launch: checked },
                })
              }
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
        <CardContent className="space-y-3">
          <Button
            variant="outline"
            className="w-full"
            onClick={() =>
              setConfig({
                ...config,
                debug: {
                  ...config.debug,
                  force_java_console: true,
                  disable_create_no_window: true,
                  log_launch_commands: true,
                  verbose_logging: true,
                },
              })
            }
          >
            Enable All Debug Settings
          </Button>
          <Button
            variant="ghost"
            className="w-full"
            onClick={() =>
              setConfig({
                ...config,
                debug: {
                  force_java_console: false,
                  disable_create_no_window: false,
                  log_launch_commands: false,
                  verbose_logging: false,
                  keep_natives_after_launch: false,
                  pause_before_launch: false,
                },
              })
            }
          >
            Reset All Debug Settings
          </Button>
          <p className="text-xs text-muted-foreground">
            Remember to disable debug settings when not troubleshooting - they may impact performance.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}

// Main Oxide Launcher Settings Component
export function OxideLauncherSettings() {
  return (
    <Tabs defaultValue="general" className="w-full">
      <TabsList className="mb-4">
        <TabsTrigger value="general">General</TabsTrigger>
        <TabsTrigger value="instances">Instances</TabsTrigger>
        <TabsTrigger value="window">Window</TabsTrigger>
        <TabsTrigger value="appearance">Appearance</TabsTrigger>
        <TabsTrigger value="debug">Debug</TabsTrigger>
      </TabsList>

      <TabsContent value="general">
        <GeneralSettings />
      </TabsContent>

      <TabsContent value="instances">
        <InstanceSettings />
      </TabsContent>

      <TabsContent value="window">
        <WindowSettings />
      </TabsContent>

      <TabsContent value="appearance">
        <AppearanceSettings />
      </TabsContent>

      <TabsContent value="debug">
        <DebugSettings />
      </TabsContent>
    </Tabs>
  );
}
