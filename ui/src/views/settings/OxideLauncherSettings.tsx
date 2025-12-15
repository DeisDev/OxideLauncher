import { FolderOpen, Cog } from "lucide-react";
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

// Main Oxide Launcher Settings Component
export function OxideLauncherSettings() {
  return (
    <Tabs defaultValue="general" className="w-full">
      <TabsList className="mb-4">
        <TabsTrigger value="general">General</TabsTrigger>
        <TabsTrigger value="instances">Instances</TabsTrigger>
        <TabsTrigger value="appearance">Appearance</TabsTrigger>
      </TabsList>

      <TabsContent value="general">
        <GeneralSettings />
      </TabsContent>

      <TabsContent value="instances">
        <InstanceSettings />
      </TabsContent>

      <TabsContent value="appearance">
        <AppearanceSettings />
      </TabsContent>
    </Tabs>
  );
}
