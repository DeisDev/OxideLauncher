import { useEffect, useState, useRef } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import {
  ArrowLeft,
  Copy,
  Upload,
  Trash2,
  Search,
  X,
  Download,
  Settings,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";

type TabType =
  | "log"
  | "version"
  | "mods"
  | "resourcepacks"
  | "shaderpacks"
  | "notes"
  | "worlds"
  | "screenshots"
  | "settings";

interface InstanceInfo {
  id: string;
  name: string;
  minecraft_version: string;
  mod_loader: string;
}

interface ModSearchResult {
  id: string;
  name: string;
  description: string;
  author: string;
  downloads: number;
  icon_url: string | null;
  project_type: string;
}

interface InstalledMod {
  filename: string;
  enabled: boolean;
}

const TABS: { id: TabType; label: string }[] = [
  { id: "log", label: "Minecraft Log" },
  { id: "version", label: "Version" },
  { id: "mods", label: "Mods" },
  { id: "resourcepacks", label: "Resource Packs" },
  { id: "shaderpacks", label: "Shader Packs" },
  { id: "notes", label: "Notes" },
  { id: "worlds", label: "Worlds" },
  { id: "screenshots", label: "Screenshots" },
  { id: "settings", label: "Settings" },
];

export function InstanceDetailsView() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = useState<TabType>("log");
  const [instance, setInstance] = useState<InstanceInfo | null>(null);
  const [loading, setLoading] = useState(true);

  // Log state
  const [logContent, setLogContent] = useState<string[]>([
    "[00:00:00] [main/INFO]: Loading Minecraft...",
    "[00:00:01] [main/INFO]: Starting game version 1.21.1",
    "[00:00:02] [Render thread/INFO]: Backend library: LWJGL version 3.3.3",
    "[00:00:03] [main/INFO]: Found mod: Fabric Loader 0.16.14",
    "[00:00:04] [main/INFO]: Initializing game",
  ]);
  const [autoScroll, setAutoScroll] = useState(true);
  const [wrapLines, setWrapLines] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const logEndRef = useRef<HTMLDivElement>(null);

  // Mods state
  const [modSearchQuery, setModSearchQuery] = useState("");
  const [modSearchResults, setModSearchResults] = useState<ModSearchResult[]>([]);
  const [installedMods, setInstalledMods] = useState<InstalledMod[]>([]);
  const [searchingMods, setSearchingMods] = useState(false);
  const [downloadingMod, setDownloadingMod] = useState<string | null>(null);
  const [deleteModDialog, setDeleteModDialog] = useState<string | null>(null);

  // Notes state
  const [notes, setNotes] = useState("");

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
    if (autoScroll && logEndRef.current) {
      logEndRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [logContent, autoScroll]);

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

  const copyLogs = () => {
    navigator.clipboard.writeText(logContent.join("\n"));
  };

  const uploadLogs = () => {
    alert("Upload to pastebin/logs service not implemented yet");
  };

  const clearLogs = () => {
    setLogContent([]);
  };

  const searchMods = async () => {
    if (!modSearchQuery.trim()) return;

    setSearchingMods(true);
    try {
      const results = await invoke<ModSearchResult[]>("search_mods", {
        query: modSearchQuery,
        gameVersion: instance?.minecraft_version,
        loader: instance?.mod_loader.toLowerCase(),
      });
      setModSearchResults(results);
    } catch (error) {
      console.error("Failed to search mods:", error);
      alert("Failed to search mods: " + error);
    } finally {
      setSearchingMods(false);
    }
  };

  const loadInstalledMods = async () => {
    try {
      const mods = await invoke<InstalledMod[]>("get_installed_mods", {
        instanceId: id,
      });
      setInstalledMods(mods);
    } catch (error) {
      console.error("Failed to load installed mods:", error);
    }
  };

  const downloadMod = async (modId: string) => {
    setDownloadingMod(modId);
    try {
      await invoke("download_mod", {
        instanceId: id,
        modId: modId,
      });
      await loadInstalledMods();
      alert("Mod downloaded successfully!");
    } catch (error) {
      console.error("Failed to download mod:", error);
      alert("Failed to download mod: " + error);
    } finally {
      setDownloadingMod(null);
    }
  };

  const toggleMod = async (filename: string) => {
    try {
      await invoke("toggle_mod", {
        instanceId: id,
        filename: filename,
      });
      await loadInstalledMods();
    } catch (error) {
      console.error("Failed to toggle mod:", error);
      alert("Failed to toggle mod: " + error);
    }
  };

  const deleteMod = async (filename: string) => {
    try {
      await invoke("delete_mod", {
        instanceId: id,
        filename: filename,
      });
      await loadInstalledMods();
    } catch (error) {
      console.error("Failed to delete mod:", error);
      alert("Failed to delete mod: " + error);
    } finally {
      setDeleteModDialog(null);
    }
  };

  useEffect(() => {
    if (activeTab === "mods") {
      loadInstalledMods();
    }
  }, [activeTab]);

  const filteredLogs = searchTerm
    ? logContent.filter((line) =>
        line.toLowerCase().includes(searchTerm.toLowerCase())
      )
    : logContent;

  if (loading) {
    return <div className="flex items-center justify-center h-full">Loading instance...</div>;
  }

  if (!instance) {
    return <div className="flex items-center justify-center h-full text-destructive">Instance not found</div>;
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center gap-4 pb-4 border-b">
        <Button variant="outline" onClick={() => navigate("/")}>
          <ArrowLeft className="mr-2 h-4 w-4" /> Back
        </Button>
        <h1 className="text-2xl font-bold">{instance.name}</h1>
        <div className="flex gap-2">
          <Badge variant="secondary">{instance.minecraft_version}</Badge>
          <Badge variant="outline">{instance.mod_loader}</Badge>
        </div>
      </div>

      {/* Body */}
      <div className="flex flex-1 gap-6 pt-6 overflow-hidden">
        {/* Sidebar */}
        <div className="w-48 flex-shrink-0 space-y-1">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              className={cn(
                "w-full px-4 py-2 text-left text-sm rounded-md transition-colors",
                activeTab === tab.id
                  ? "bg-primary text-primary-foreground"
                  : "hover:bg-muted"
              )}
              onClick={() => setActiveTab(tab.id)}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {/* Content */}
        <div className="flex-1 overflow-hidden">
          {/* Log Tab */}
          {activeTab === "log" && (
            <div className="flex flex-col h-full gap-4">
              <div className="flex items-center gap-2 flex-wrap">
                <Button variant="outline" size="sm" onClick={copyLogs}>
                  <Copy className="mr-2 h-4 w-4" /> Copy
                </Button>
                <Button variant="outline" size="sm" onClick={uploadLogs}>
                  <Upload className="mr-2 h-4 w-4" /> Upload
                </Button>
                <Button variant="destructive" size="sm" onClick={clearLogs}>
                  <Trash2 className="mr-2 h-4 w-4" /> Clear
                </Button>

                <div className="flex-1" />

                <div className="flex items-center space-x-2">
                  <Checkbox
                    id="autoScroll"
                    checked={autoScroll}
                    onCheckedChange={(checked) => setAutoScroll(checked as boolean)}
                  />
                  <Label htmlFor="autoScroll" className="text-sm">Auto-scroll</Label>
                </div>

                <div className="flex items-center space-x-2">
                  <Checkbox
                    id="wrapLines"
                    checked={wrapLines}
                    onCheckedChange={(checked) => setWrapLines(checked as boolean)}
                  />
                  <Label htmlFor="wrapLines" className="text-sm">Wrap lines</Label>
                </div>
              </div>

              <div className="flex items-center gap-2">
                <div className="relative flex-1">
                  <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                  <Input
                    placeholder="Search logs..."
                    value={searchTerm}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    className="pl-9"
                  />
                </div>
                <Button variant="outline" size="sm" onClick={() => setSearchTerm("")}>
                  <X className="h-4 w-4" />
                </Button>
              </div>

              <ScrollArea className="flex-1 rounded-md border bg-black/50">
                <div className={cn("p-4 font-mono text-xs", wrapLines ? "whitespace-pre-wrap" : "whitespace-pre")}>
                  {filteredLogs.map((line, index) => (
                    <div key={index} className="hover:bg-white/5">
                      {line}
                    </div>
                  ))}
                  <div ref={logEndRef} />
                </div>
              </ScrollArea>
            </div>
          )}

          {/* Version Tab */}
          {activeTab === "version" && (
            <Card>
              <CardHeader>
                <CardTitle>Version Components</CardTitle>
                <CardDescription>
                  Components and libraries for this instance.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <p className="text-muted-foreground">
                  This tab would show the components/libraries for this instance.
                </p>
              </CardContent>
            </Card>
          )}

          {/* Mods Tab */}
          {activeTab === "mods" && (
            <div className="flex flex-col gap-6 h-full overflow-hidden">
              <div className="flex items-center gap-4">
                <Input
                  placeholder="Search mods on Modrinth..."
                  value={modSearchQuery}
                  onChange={(e) => setModSearchQuery(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && searchMods()}
                  className="flex-1"
                />
                <Button onClick={searchMods} disabled={searchingMods}>
                  <Search className="mr-2 h-4 w-4" />
                  {searchingMods ? "Searching..." : "Search"}
                </Button>
              </div>

              <ScrollArea className="flex-1">
                {modSearchResults.length > 0 && (
                  <div className="mb-6">
                    <h3 className="text-lg font-semibold mb-4">Search Results</h3>
                    <div className="grid gap-4">
                      {modSearchResults.map((mod) => (
                        <Card key={mod.id}>
                          <CardContent className="flex items-start gap-4 p-4">
                            {mod.icon_url && (
                              <img
                                src={mod.icon_url}
                                alt={mod.name}
                                className="w-12 h-12 rounded-md object-cover"
                              />
                            )}
                            <div className="flex-1 min-w-0">
                              <h4 className="font-semibold">{mod.name}</h4>
                              <p className="text-sm text-muted-foreground">by {mod.author}</p>
                              <p className="text-sm mt-1 line-clamp-2">{mod.description}</p>
                              <p className="text-xs text-muted-foreground mt-2">
                                {mod.downloads.toLocaleString()} downloads
                              </p>
                            </div>
                            <Button
                              onClick={() => downloadMod(mod.id)}
                              disabled={downloadingMod === mod.id}
                              size="sm"
                            >
                              <Download className="mr-2 h-4 w-4" />
                              {downloadingMod === mod.id ? "Installing..." : "Install"}
                            </Button>
                          </CardContent>
                        </Card>
                      ))}
                    </div>
                  </div>
                )}

                <div>
                  <h3 className="text-lg font-semibold mb-4">
                    Installed Mods ({installedMods.length})
                  </h3>
                  {installedMods.length === 0 ? (
                    <p className="text-muted-foreground">
                      No mods installed yet. Search and install mods above.
                    </p>
                  ) : (
                    <div className="space-y-2">
                      {installedMods.map((mod) => (
                        <div
                          key={mod.filename}
                          className="flex items-center justify-between p-3 rounded-md border bg-card"
                        >
                          <div className="flex items-center gap-3">
                            <Switch
                              checked={mod.enabled}
                              onCheckedChange={() => toggleMod(mod.filename)}
                            />
                            <span className={cn(!mod.enabled && "text-muted-foreground")}>
                              {mod.filename}
                            </span>
                          </div>
                          <Button
                            variant="destructive"
                            size="sm"
                            onClick={() => setDeleteModDialog(mod.filename)}
                          >
                            <Trash2 className="mr-2 h-4 w-4" /> Delete
                          </Button>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </ScrollArea>
            </div>
          )}

          {/* Resource Packs Tab */}
          {activeTab === "resourcepacks" && (
            <Card>
              <CardHeader>
                <CardTitle>Resource Packs</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-muted-foreground">Resource pack management coming soon...</p>
              </CardContent>
            </Card>
          )}

          {/* Shader Packs Tab */}
          {activeTab === "shaderpacks" && (
            <Card>
              <CardHeader>
                <CardTitle>Shader Packs</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-muted-foreground">Shader pack management coming soon...</p>
              </CardContent>
            </Card>
          )}

          {/* Notes Tab */}
          {activeTab === "notes" && (
            <Card className="h-full">
              <CardHeader>
                <CardTitle>Notes</CardTitle>
                <CardDescription>Add notes about this instance.</CardDescription>
              </CardHeader>
              <CardContent>
                <Textarea
                  placeholder="Add notes about this instance..."
                  value={notes}
                  onChange={(e) => setNotes(e.target.value)}
                  className="min-h-[300px]"
                />
              </CardContent>
            </Card>
          )}

          {/* Worlds Tab */}
          {activeTab === "worlds" && (
            <Card>
              <CardHeader>
                <CardTitle>Worlds</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-muted-foreground">World management coming soon...</p>
              </CardContent>
            </Card>
          )}

          {/* Screenshots Tab */}
          {activeTab === "screenshots" && (
            <Card>
              <CardHeader>
                <CardTitle>Screenshots</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-muted-foreground">Screenshot viewer coming soon...</p>
              </CardContent>
            </Card>
          )}

          {/* Settings Tab */}
          {activeTab === "settings" && (
            <Card>
              <CardHeader>
                <CardTitle>Settings</CardTitle>
                <CardDescription>Configure settings for this instance.</CardDescription>
              </CardHeader>
              <CardContent>
                <Button onClick={() => navigate(`/instance/${id}/settings`)}>
                  <Settings className="mr-2 h-4 w-4" /> Open Settings Editor
                </Button>
              </CardContent>
            </Card>
          )}
        </div>
      </div>

      {/* Delete Mod Dialog */}
      <AlertDialog open={!!deleteModDialog} onOpenChange={() => setDeleteModDialog(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Mod</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete "{deleteModDialog}"? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => deleteModDialog && deleteMod(deleteModDialog)}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
