import { useEffect, useState, useRef } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft, Save } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { cn } from "@/lib/utils";

// Import modular tab components
import {
  TABS,
  TabType,
  InstanceInfo,
  LogTab,
  VersionTab,
  ModsTab,
  ResourcePacksTab,
  ShaderPacksTab,
  NotesTab,
  WorldsTab,
  ScreenshotsTab,
  SettingsTab,
} from "./instance-details";

export function InstanceDetailsView() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = useState<TabType>("log");
  const [instance, setInstance] = useState<InstanceInfo | null>(null);
  const [loading, setLoading] = useState(true);

  // Log state
  const [logContent, setLogContent] = useState<string[]>([]);
  const logEndRef = useRef<HTMLDivElement>(null);

  // Notes state
  const [notes, setNotes] = useState("");
  const [savingNotes, setSavingNotes] = useState(false);

  useEffect(() => {
    loadInstance();

    const interval = setInterval(async () => {
      try {
        const logs = await invoke<string[]>("get_instance_logs", {
          instanceId: id,
        });
        if (logs.length > 0) {
          setLogContent(logs);
        }
      } catch (error) {
        console.error("Failed to fetch logs:", error);
      }
    }, 1000);

    return () => clearInterval(interval);
  }, [id]);

  useEffect(() => {
    if (activeTab === "notes") {
      loadNotes();
    }
  }, [activeTab, id]);

  const loadInstance = async () => {
    try {
      const data = await invoke<InstanceInfo>("get_instance_details", {
        instanceId: id,
      });
      setInstance(data);
    } catch (error) {
      console.error("Failed to load instance:", error);
    } finally {
      setLoading(false);
    }
  };

  const loadNotes = async () => {
    if (!id) return;
    try {
      const instanceNotes = await invoke<string>("get_instance_notes", {
        instanceId: id,
      });
      setNotes(instanceNotes);
    } catch (error) {
      console.error("Failed to load notes:", error);
    }
  };

  const saveNotes = async () => {
    if (!id) return;
    setSavingNotes(true);
    try {
      await invoke("save_instance_notes", {
        instanceId: id,
        notes,
      });
    } catch (error) {
      console.error("Failed to save notes:", error);
      alert("Failed to save notes: " + error);
    } finally {
      setSavingNotes(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-muted-foreground">Loading instance...</p>
      </div>
    );
  }

  if (!instance || !id) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <p className="text-muted-foreground">Instance not found</p>
        <Button variant="outline" onClick={() => navigate("/instances")}>
          <ArrowLeft className="mr-2 h-4 w-4" />
          Back to Instances
        </Button>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center gap-4 p-4 border-b">
        <Button variant="ghost" size="icon" onClick={() => navigate("/instances")}>
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div className="flex-1">
          <h1 className="text-xl font-semibold">{instance.name}</h1>
          <div className="flex items-center gap-2 mt-1">
            <Badge variant="secondary">{instance.minecraft_version}</Badge>
            <Badge variant="outline">{instance.mod_loader}</Badge>
            {instance.mod_loader_version && (
              <span className="text-xs text-muted-foreground">
                v{instance.mod_loader_version}
              </span>
            )}
          </div>
        </div>
      </div>

      {/* Tabs */}
      <Tabs
        value={activeTab}
        onValueChange={(v) => setActiveTab(v as TabType)}
        className="flex-1 flex flex-col overflow-hidden"
      >
        <TabsList className="w-full justify-start gap-1 rounded-none border-b bg-transparent px-4 h-auto py-1">
          {TABS.map((tab) => (
            <TabsTrigger
              key={tab.id}
              value={tab.id}
              className={cn(
                "data-[state=active]:bg-background data-[state=active]:shadow-sm",
                "rounded-t-md rounded-b-none border-b-2 border-transparent",
                "data-[state=active]:border-primary px-4 py-2"
              )}
            >
              {tab.label}
            </TabsTrigger>
          ))}
        </TabsList>

        <div className="flex-1 overflow-hidden p-4">
          {/* Log Tab */}
          <TabsContent value="log" className="h-full m-0">
            <LogTab
              instanceId={id}
              logContent={logContent}
              setLogContent={setLogContent}
            />
          </TabsContent>

          {/* Version Tab */}
          <TabsContent value="version" className="h-full m-0">
            <VersionTab />
          </TabsContent>

          {/* Mods Tab */}
          <TabsContent value="mods" className="h-full m-0">
            <ModsTab instanceId={id} instance={instance} />
          </TabsContent>

          {/* Resource Packs Tab */}
          <TabsContent value="resourcepacks" className="h-full m-0">
            <ResourcePacksTab instanceId={id} />
          </TabsContent>

          {/* Shader Packs Tab */}
          <TabsContent value="shaderpacks" className="h-full m-0">
            <ShaderPacksTab instanceId={id} />
          </TabsContent>

          {/* Notes Tab */}
          <TabsContent value="notes" className="h-full m-0">
            <div className="flex flex-col h-full gap-4">
              <NotesTab notes={notes} setNotes={setNotes} />
              <div className="flex justify-end">
                <Button onClick={saveNotes} disabled={savingNotes}>
                  <Save className="mr-2 h-4 w-4" />
                  {savingNotes ? "Saving..." : "Save Notes"}
                </Button>
              </div>
            </div>
          </TabsContent>

          {/* Worlds Tab */}
          <TabsContent value="worlds" className="h-full m-0">
            <WorldsTab instanceId={id} />
          </TabsContent>

          {/* Screenshots Tab */}
          <TabsContent value="screenshots" className="h-full m-0">
            <ScreenshotsTab instanceId={id} />
          </TabsContent>

          {/* Settings Tab */}
          <TabsContent value="settings" className="h-full m-0">
            <SettingsTab instanceId={id} instance={instance} />
          </TabsContent>
        </div>
      </Tabs>
    </div>
  );
}
