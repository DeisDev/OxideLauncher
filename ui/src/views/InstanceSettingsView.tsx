import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { Save, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

interface InstanceInfo {
  id: string;
  name: string;
  minecraft_version: string;
  mod_loader: string;
}

export function InstanceSettingsView() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [instance, setInstance] = useState<InstanceInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [saveSuccess, setSaveSuccess] = useState(false);

  // Settings state
  const [name, setName] = useState("");
  const [javaPath, setJavaPath] = useState("");
  const [javaArgs, setJavaArgs] = useState("");
  const [minMemory, setMinMemory] = useState("512");
  const [maxMemory, setMaxMemory] = useState("4096");
  const [width, setWidth] = useState("854");
  const [height, setHeight] = useState("480");

  useEffect(() => {
    loadInstance();
  }, [id]);

  const loadInstance = async () => {
    try {
      const data = await invoke<InstanceInfo>("get_instance_details", {
        instanceId: id,
      });
      setInstance(data);
      setName(data.name);
    } catch (error) {
      console.error("Failed to load instance:", error);
    } finally {
      setLoading(false);
    }
  };

  const saveSettings = async () => {
    // TODO: Implement save settings command
    setSaveSuccess(true);
  };

  if (loading) {
    return <div className="flex items-center justify-center h-full">Loading settings...</div>;
  }

  if (!instance) {
    return <div className="flex items-center justify-center h-full text-destructive">Instance not found</div>;
  }

  return (
    <div className="max-w-4xl mx-auto">
      <h1 className="text-3xl font-bold mb-8">Settings for {instance.name}</h1>

      <Tabs defaultValue="general" className="w-full">
        <TabsList className="grid w-full grid-cols-4 mb-6">
          <TabsTrigger value="general">General</TabsTrigger>
          <TabsTrigger value="java">Java</TabsTrigger>
          <TabsTrigger value="memory">Memory</TabsTrigger>
          <TabsTrigger value="game">Game Window</TabsTrigger>
        </TabsList>

        <TabsContent value="general">
          <Card>
            <CardHeader>
              <CardTitle>General Settings</CardTitle>
              <CardDescription>Basic instance configuration.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="name">Instance Name</Label>
                <Input
                  id="name"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                />
              </div>
              <div className="space-y-2">
                <Label>Minecraft Version</Label>
                <Input value={instance.minecraft_version} disabled />
              </div>
              <div className="space-y-2">
                <Label>Mod Loader</Label>
                <Input value={instance.mod_loader} disabled />
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="java">
          <Card>
            <CardHeader>
              <CardTitle>Java Settings</CardTitle>
              <CardDescription>
                Configure Java runtime settings for this instance.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="javaPath">Custom Java Path (optional)</Label>
                <Input
                  id="javaPath"
                  value={javaPath}
                  onChange={(e) => setJavaPath(e.target.value)}
                  placeholder="Leave empty to use default"
                />
                <p className="text-sm text-muted-foreground">
                  Leave empty to use the global Java configuration.
                </p>
              </div>
              <div className="space-y-2">
                <Label htmlFor="javaArgs">Extra Java Arguments</Label>
                <Input
                  id="javaArgs"
                  value={javaArgs}
                  onChange={(e) => setJavaArgs(e.target.value)}
                  placeholder="-XX:+UseG1GC -Dsun.rmi.dgc.server.gcInterval=2147483646"
                />
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="memory">
          <Card>
            <CardHeader>
              <CardTitle>Memory Settings</CardTitle>
              <CardDescription>
                Configure how much RAM this instance can use.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="minMemory">Minimum Memory (MB)</Label>
                  <Input
                    id="minMemory"
                    type="number"
                    value={minMemory}
                    onChange={(e) => setMinMemory(e.target.value)}
                    min="512"
                    max="32768"
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="maxMemory">Maximum Memory (MB)</Label>
                  <Input
                    id="maxMemory"
                    type="number"
                    value={maxMemory}
                    onChange={(e) => setMaxMemory(e.target.value)}
                    min="1024"
                    max="32768"
                  />
                </div>
              </div>
              <p className="text-sm text-muted-foreground">
                Leave empty to use the global memory configuration.
              </p>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="game">
          <Card>
            <CardHeader>
              <CardTitle>Game Window</CardTitle>
              <CardDescription>
                Configure the Minecraft window size on launch.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="width">Window Width</Label>
                  <Input
                    id="width"
                    type="number"
                    value={width}
                    onChange={(e) => setWidth(e.target.value)}
                    min="640"
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="height">Window Height</Label>
                  <Input
                    id="height"
                    type="number"
                    value={height}
                    onChange={(e) => setHeight(e.target.value)}
                    min="480"
                  />
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      <div className="flex justify-end gap-4 mt-8">
        <Button variant="outline" onClick={() => navigate("/")}>
          <X className="mr-2 h-4 w-4" /> Cancel
        </Button>
        <Button onClick={saveSettings}>
          <Save className="mr-2 h-4 w-4" /> Save Settings
        </Button>
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
            <AlertDialogAction onClick={() => navigate("/")}>OK</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
