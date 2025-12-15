import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useSettings } from "./context";

// Window Settings Sub-tab
function WindowSettings() {
  const { config, setConfig } = useSettings();
  if (!config) return null;

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Game Window</CardTitle>
          <CardDescription>
            Configure the Minecraft game window settings.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="gameWidth">Window Width</Label>
              <Input
                id="gameWidth"
                type="number"
                value={config.minecraft.window_width}
                onChange={(e) =>
                  setConfig({
                    ...config,
                    minecraft: {
                      ...config.minecraft,
                      window_width: parseInt(e.target.value) || 854,
                    },
                  })
                }
                min="640"
                max="7680"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="gameHeight">Window Height</Label>
              <Input
                id="gameHeight"
                type="number"
                value={config.minecraft.window_height}
                onChange={(e) =>
                  setConfig({
                    ...config,
                    minecraft: {
                      ...config.minecraft,
                      window_height: parseInt(e.target.value) || 480,
                    },
                  })
                }
                min="480"
                max="4320"
              />
            </div>
          </div>
          <p className="text-sm text-muted-foreground">
            The initial size of the Minecraft game window. Standard resolution is 854×480.
          </p>

          <div className="flex items-center justify-between pt-4">
            <div className="space-y-0.5">
              <Label htmlFor="launchMaximized">Launch Maximized</Label>
              <p className="text-sm text-muted-foreground">
                Start Minecraft in a maximized window.
              </p>
            </div>
            <Switch
              id="launchMaximized"
              checked={config.minecraft.launch_maximized}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config,
                  minecraft: { ...config.minecraft, launch_maximized: checked },
                })
              }
            />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>After Launch</CardTitle>
          <CardDescription>
            What happens after Minecraft starts.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="closeAfterLaunch">Close Launcher on Launch</Label>
              <p className="text-sm text-muted-foreground">
                Close Oxide Launcher after Minecraft starts.
              </p>
            </div>
            <Switch
              id="closeAfterLaunch"
              checked={config.minecraft.close_after_launch}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config,
                  minecraft: { ...config.minecraft, close_after_launch: checked },
                })
              }
            />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// Console Settings Sub-tab
function ConsoleSettings() {
  const { config, setConfig } = useSettings();
  if (!config) return null;

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Game Console</CardTitle>
          <CardDescription>
            Configure game console window behavior.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="showConsole">Show Console on Launch</Label>
              <p className="text-sm text-muted-foreground">
                Open the game console window when Minecraft starts.
              </p>
            </div>
            <Switch
              id="showConsole"
              checked={config.minecraft.show_console}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config,
                  minecraft: { ...config.minecraft, show_console: checked },
                })
              }
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="autoCloseConsole">Auto-Close Console</Label>
              <p className="text-sm text-muted-foreground">
                Automatically close the console when Minecraft exits normally.
              </p>
            </div>
            <Switch
              id="autoCloseConsole"
              checked={config.minecraft.auto_close_console}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config,
                  minecraft: { ...config.minecraft, auto_close_console: checked },
                })
              }
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="showConsoleOnError">Show Console on Error</Label>
              <p className="text-sm text-muted-foreground">
                Automatically show the console if Minecraft crashes or exits with an error.
              </p>
            </div>
            <Switch
              id="showConsoleOnError"
              checked={config.minecraft.show_console_on_error}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config,
                  minecraft: { ...config.minecraft, show_console_on_error: checked },
                })
              }
            />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// Game Time Settings Sub-tab
function GameTimeSettings() {
  const { config, setConfig } = useSettings();
  if (!config) return null;

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Play Time Tracking</CardTitle>
          <CardDescription>
            Configure how game time is tracked and displayed.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="recordGameTime">Record Game Time</Label>
              <p className="text-sm text-muted-foreground">
                Track how long you play each instance.
              </p>
            </div>
            <Switch
              id="recordGameTime"
              checked={config.minecraft.record_game_time}
              onCheckedChange={(checked) =>
                setConfig({
                  ...config,
                  minecraft: { ...config.minecraft, record_game_time: checked },
                })
              }
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="showGameTime">Show Game Time</Label>
              <p className="text-sm text-muted-foreground">
                Display total play time on instance cards and details.
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

// Main Minecraft Settings Component
export function MinecraftSettings() {
  return (
    <Tabs defaultValue="window" className="w-full">
      <TabsList className="mb-4">
        <TabsTrigger value="window">Window</TabsTrigger>
        <TabsTrigger value="console">Console</TabsTrigger>
        <TabsTrigger value="gametime">Game Time</TabsTrigger>
      </TabsList>

      <TabsContent value="window">
        <WindowSettings />
      </TabsContent>

      <TabsContent value="console">
        <ConsoleSettings />
      </TabsContent>

      <TabsContent value="gametime">
        <GameTimeSettings />
      </TabsContent>
    </Tabs>
  );
}
