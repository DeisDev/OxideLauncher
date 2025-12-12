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
  Save,
  Plus,
  Check,
  XCircle,
  FolderOpen,
  FileText,
  RefreshCw,
  Package,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
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
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
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
  mod_loader_version: string | null;
}

interface ModSearchResult {
  id: string;
  name: string;
  description: string;
  author: string;
  downloads: number;
  icon_url: string | null;
  project_type: string;
  platform: string;
}

interface InstalledMod {
  filename: string;
  name: string;
  version: string | null;
  enabled: boolean;
  size: number;
  modified: string | null;
  provider: string | null;
  icon_url: string | null;
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
  const [logContent, setLogContent] = useState<string[]>([]);
  const [autoScroll, setAutoScroll] = useState(true);
  const [wrapLines, setWrapLines] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const logEndRef = useRef<HTMLDivElement>(null);

  // Mods state
  const [modSearchQuery, setModSearchQuery] = useState("");
  const [modSearchResults, setModSearchResults] = useState<ModSearchResult[]>([]);
  const [installedMods, setInstalledMods] = useState<InstalledMod[]>([]);
  const [filteredMods, setFilteredMods] = useState<InstalledMod[]>([]);
  const [modFilter, setModFilter] = useState("");
  const [selectedMods, setSelectedMods] = useState<Set<string>>(new Set());
  const [searchingMods, setSearchingMods] = useState(false);
  const [downloadingMod, setDownloadingMod] = useState<string | null>(null);
  const [deleteModDialog, setDeleteModDialog] = useState<string | null>(null);
  const [showModSearch, setShowModSearch] = useState(false);
  const [searchPlatform, setSearchPlatform] = useState<"modrinth" | "curseforge">("modrinth");
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Notes state
  const [notes, setNotes] = useState("");

  // Settings state
  const [settingsName, setSettingsName] = useState("");
  const [javaPath, setJavaPath] = useState("");
  const [javaArgs, setJavaArgs] = useState("");
  const [minMemory, setMinMemory] = useState("512");
  const [maxMemory, setMaxMemory] = useState("4096");
  const [windowWidth, setWindowWidth] = useState("854");
  const [windowHeight, setWindowHeight] = useState("480");

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
      setSettingsName(data.name);
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
        minecraftVersion: instance?.minecraft_version,
        modLoader: instance?.mod_loader.toLowerCase(),
        platform: searchPlatform,
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

  const downloadMod = async (modId: string, platform: string) => {
    setDownloadingMod(modId);
    try {
      await invoke("download_mod", {
        instanceId: id,
        modId: modId,
        platform: platform.toLowerCase(),
      });
      await loadInstalledMods();
    } catch (error) {
      console.error("Failed to download mod:", error);
      alert("Failed to download mod: " + error);
    } finally {
      setDownloadingMod(null);
    }
  };

  const toggleMod = async (filename: string, currentEnabled: boolean) => {
    try {
      await invoke("toggle_mod", {
        instanceId: id,
        filename: filename,
        enabled: !currentEnabled,
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
      setSelectedMods(prev => {
        const next = new Set(prev);
        next.delete(filename);
        return next;
      });
    } catch (error) {
      console.error("Failed to delete mod:", error);
      alert("Failed to delete mod: " + error);
    } finally {
      setDeleteModDialog(null);
    }
  };

  const deleteSelectedMods = async () => {
    if (selectedMods.size === 0) return;
    try {
      await invoke("delete_mods", {
        instanceId: id,
        filenames: Array.from(selectedMods),
      });
      setSelectedMods(new Set());
      await loadInstalledMods();
    } catch (error) {
      console.error("Failed to delete mods:", error);
      alert("Failed to delete mods: " + error);
    }
  };

  const enableSelectedMods = async () => {
    if (selectedMods.size === 0) return;
    try {
      await invoke("enable_mods", {
        instanceId: id,
        filenames: Array.from(selectedMods),
      });
      await loadInstalledMods();
    } catch (error) {
      console.error("Failed to enable mods:", error);
      alert("Failed to enable mods: " + error);
    }
  };

  const disableSelectedMods = async () => {
    if (selectedMods.size === 0) return;
    try {
      await invoke("disable_mods", {
        instanceId: id,
        filenames: Array.from(selectedMods),
      });
      await loadInstalledMods();
    } catch (error) {
      console.error("Failed to disable mods:", error);
      alert("Failed to disable mods: " + error);
    }
  };

  const openModsFolder = async () => {
    try {
      await invoke("open_mods_folder", { instanceId: id });
    } catch (error) {
      console.error("Failed to open mods folder:", error);
      alert("Failed to open folder: " + error);
    }
  };

  const openConfigsFolder = async () => {
    try {
      await invoke("open_configs_folder", { instanceId: id });
    } catch (error) {
      console.error("Failed to open configs folder:", error);
      alert("Failed to open folder: " + error);
    }
  };

  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (!files || files.length === 0) return;
    
    for (const file of Array.from(files)) {
      if (!file.name.endsWith('.jar')) {
        alert(`Skipping ${file.name}: Only .jar files are supported`);
        continue;
      }
      try {
        // For web, we need to read the file and send it to the backend
        // This is a simplified approach - in production you'd want proper file handling
        await invoke("add_local_mod", {
          instanceId: id,
          filePath: file.name, // Note: This won't work directly, see below
        });
      } catch (error) {
        console.error("Failed to add mod:", error);
        alert(`Failed to add ${file.name}: ${error}`);
      }
    }
    await loadInstalledMods();
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  };

  const toggleModSelection = (filename: string) => {
    setSelectedMods(prev => {
      const next = new Set(prev);
      if (next.has(filename)) {
        next.delete(filename);
      } else {
        next.add(filename);
      }
      return next;
    });
  };

  const toggleAllMods = () => {
    const displayedMods = modFilter 
      ? installedMods.filter(m => m.name.toLowerCase().includes(modFilter.toLowerCase()))
      : installedMods;
    
    if (selectedMods.size === displayedMods.length) {
      setSelectedMods(new Set());
    } else {
      setSelectedMods(new Set(displayedMods.map(m => m.filename)));
    }
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  // Filter mods based on search
  useEffect(() => {
    if (modFilter) {
      setFilteredMods(installedMods.filter(m => 
        m.name.toLowerCase().includes(modFilter.toLowerCase()) ||
        m.filename.toLowerCase().includes(modFilter.toLowerCase())
      ));
    } else {
      setFilteredMods(installedMods);
    }
  }, [modFilter, installedMods]);

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
          {instance.mod_loader !== "Vanilla" && (
            <Badge variant="outline">
              {instance.mod_loader}
              {instance.mod_loader_version && ` ${instance.mod_loader_version}`}
            </Badge>
          )}
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
            <div className="flex flex-col h-full overflow-hidden">
              {/* Hidden file input for adding local mods */}
              <input
                type="file"
                ref={fileInputRef}
                onChange={handleFileSelect}
                accept=".jar"
                multiple
                className="hidden"
              />

              {/* Action Toolbar */}
              <div className="flex items-center gap-2 pb-4 border-b flex-wrap">
                <Button 
                  variant="default" 
                  size="sm" 
                  onClick={() => setShowModSearch(!showModSearch)}
                >
                  <Download className="mr-2 h-4 w-4" />
                  Download Mods
                </Button>
                <Button 
                  variant="outline" 
                  size="sm" 
                  onClick={() => fileInputRef.current?.click()}
                >
                  <Plus className="mr-2 h-4 w-4" />
                  Add File
                </Button>
                
                <div className="h-6 w-px bg-border mx-1" />
                
                <Button 
                  variant="outline" 
                  size="sm" 
                  onClick={deleteSelectedMods}
                  disabled={selectedMods.size === 0}
                >
                  <Trash2 className="mr-2 h-4 w-4" />
                  Remove
                </Button>
                <Button 
                  variant="outline" 
                  size="sm" 
                  onClick={enableSelectedMods}
                  disabled={selectedMods.size === 0}
                >
                  <Check className="mr-2 h-4 w-4" />
                  Enable
                </Button>
                <Button 
                  variant="outline" 
                  size="sm" 
                  onClick={disableSelectedMods}
                  disabled={selectedMods.size === 0}
                >
                  <XCircle className="mr-2 h-4 w-4" />
                  Disable
                </Button>
                
                <div className="flex-1" />
                
                <Button 
                  variant="ghost" 
                  size="sm" 
                  onClick={loadInstalledMods}
                >
                  <RefreshCw className="h-4 w-4" />
                </Button>
              </div>

              {/* Mod Search Panel (collapsible) */}
              {showModSearch && (
                <div className="py-4 border-b">
                  {/* Platform Selection */}
                  <div className="flex items-center gap-2 mb-3">
                    <span className="text-sm text-muted-foreground">Search on:</span>
                    <div className="flex gap-1">
                      <Button
                        variant={searchPlatform === "modrinth" ? "default" : "outline"}
                        size="sm"
                        onClick={() => setSearchPlatform("modrinth")}
                        className="h-7 px-3 text-xs"
                      >
                        Modrinth
                      </Button>
                      <Button
                        variant={searchPlatform === "curseforge" ? "default" : "outline"}
                        size="sm"
                        onClick={() => setSearchPlatform("curseforge")}
                        className="h-7 px-3 text-xs"
                      >
                        CurseForge
                      </Button>
                    </div>
                  </div>
                  
                  <div className="flex items-center gap-4 mb-4">
                    <Input
                      placeholder={`Search mods on ${searchPlatform === "modrinth" ? "Modrinth" : "CurseForge"}...`}
                      value={modSearchQuery}
                      onChange={(e) => setModSearchQuery(e.target.value)}
                      onKeyDown={(e) => e.key === "Enter" && searchMods()}
                      className="flex-1"
                    />
                    <Button onClick={searchMods} disabled={searchingMods}>
                      <Search className="mr-2 h-4 w-4" />
                      {searchingMods ? "Searching..." : "Search"}
                    </Button>
                    <Button 
                      variant="ghost" 
                      size="sm" 
                      onClick={() => {
                        setShowModSearch(false);
                        setModSearchResults([]);
                      }}
                    >
                      <X className="h-4 w-4" />
                    </Button>
                  </div>
                  
                  {modSearchResults.length > 0 && (
                    <ScrollArea className="h-64">
                      <div className="grid gap-2">
                        {modSearchResults.map((mod) => (
                          <div 
                            key={mod.id} 
                            className="flex items-center gap-3 p-2 rounded-md border bg-card hover:bg-muted/50"
                          >
                            {mod.icon_url ? (
                              <img
                                src={mod.icon_url}
                                alt={mod.name}
                                className="w-10 h-10 rounded object-cover"
                              />
                            ) : (
                              <div className="w-10 h-10 rounded bg-muted flex items-center justify-center">
                                <Package className="h-5 w-5 text-muted-foreground" />
                              </div>
                            )}
                            <div className="flex-1 min-w-0">
                              <div className="flex items-center gap-2">
                                <p className="font-medium truncate">{mod.name}</p>
                                <Badge variant="outline" className="text-[10px] px-1.5 py-0">
                                  {mod.platform === "curseforge" ? "CF" : "MR"}
                                </Badge>
                              </div>
                              <p className="text-xs text-muted-foreground truncate">
                                by {mod.author} • {mod.downloads.toLocaleString()} downloads
                              </p>
                            </div>
                            <Button
                              onClick={() => downloadMod(mod.id, mod.platform)}
                              disabled={downloadingMod === mod.id}
                              size="sm"
                            >
                              {downloadingMod === mod.id ? (
                                <>
                                  <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                                  Installing...
                                </>
                              ) : (
                                <>
                                  <Download className="mr-2 h-4 w-4" />
                                  Install
                                </>
                              )}
                            </Button>
                          </div>
                        ))}
                      </div>
                    </ScrollArea>
                  )}
                </div>
              )}

              {/* Filter */}
              <div className="py-3">
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                  <Input
                    placeholder="Filter installed mods..."
                    value={modFilter}
                    onChange={(e) => setModFilter(e.target.value)}
                    className="pl-9"
                  />
                </div>
              </div>

              {/* Mods Table */}
              <div className="flex-1 overflow-hidden border rounded-md">
                <ScrollArea className="h-full">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead className="w-10">
                          <Checkbox
                            checked={selectedMods.size === filteredMods.length && filteredMods.length > 0}
                            onCheckedChange={toggleAllMods}
                          />
                        </TableHead>
                        <TableHead className="w-10">On</TableHead>
                        <TableHead className="w-10"></TableHead>
                        <TableHead>Name</TableHead>
                        <TableHead className="w-32">Version</TableHead>
                        <TableHead className="w-36">Last Modified</TableHead>
                        <TableHead className="w-24">Provider</TableHead>
                        <TableHead className="w-20 text-right">Size</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {filteredMods.length === 0 ? (
                        <TableRow>
                          <TableCell colSpan={8} className="text-center py-8 text-muted-foreground">
                            {installedMods.length === 0 
                              ? "No mods installed. Click 'Download Mods' to get started."
                              : "No mods match your filter."
                            }
                          </TableCell>
                        </TableRow>
                      ) : (
                        filteredMods.map((mod) => (
                          <TableRow 
                            key={mod.filename}
                            className={cn(selectedMods.has(mod.filename) && "bg-muted/50")}
                          >
                            <TableCell>
                              <Checkbox
                                checked={selectedMods.has(mod.filename)}
                                onCheckedChange={() => toggleModSelection(mod.filename)}
                              />
                            </TableCell>
                            <TableCell>
                              <Switch
                                checked={mod.enabled}
                                onCheckedChange={() => toggleMod(mod.filename, mod.enabled)}
                              />
                            </TableCell>
                            <TableCell>
                              {mod.icon_url ? (
                                <img
                                  src={mod.icon_url}
                                  alt=""
                                  className="w-8 h-8 rounded object-cover"
                                />
                              ) : (
                                <div className="w-8 h-8 rounded bg-muted flex items-center justify-center">
                                  <Package className="h-4 w-4 text-muted-foreground" />
                                </div>
                              )}
                            </TableCell>
                            <TableCell>
                              <div className={cn(!mod.enabled && "text-muted-foreground")}>
                                <p className="font-medium">{mod.name}</p>
                                <p className="text-xs text-muted-foreground truncate max-w-xs">
                                  {mod.filename}
                                </p>
                              </div>
                            </TableCell>
                            <TableCell className={cn(!mod.enabled && "text-muted-foreground")}>
                              {mod.version || "-"}
                            </TableCell>
                            <TableCell className="text-muted-foreground text-sm">
                              {mod.modified || "-"}
                            </TableCell>
                            <TableCell>
                              {mod.provider && (
                                <Badge variant="secondary" className="text-xs">
                                  {mod.provider}
                                </Badge>
                              )}
                            </TableCell>
                            <TableCell className="text-right text-muted-foreground text-sm">
                              {formatFileSize(mod.size)}
                            </TableCell>
                          </TableRow>
                        ))
                      )}
                    </TableBody>
                  </Table>
                </ScrollArea>
              </div>

              {/* Footer Actions */}
              <div className="flex items-center gap-2 pt-4 border-t">
                <Button variant="outline" size="sm" onClick={openConfigsFolder}>
                  <FileText className="mr-2 h-4 w-4" />
                  View Configs
                </Button>
                <Button variant="outline" size="sm" onClick={openModsFolder}>
                  <FolderOpen className="mr-2 h-4 w-4" />
                  View Folder
                </Button>
                <div className="flex-1" />
                <span className="text-sm text-muted-foreground">
                  {installedMods.length} mod{installedMods.length !== 1 ? 's' : ''} installed
                  {selectedMods.size > 0 && ` • ${selectedMods.size} selected`}
                </span>
              </div>
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
            <ScrollArea className="h-full">
              <Tabs defaultValue="general" className="w-full">
                <TabsList className="grid w-full grid-cols-4 mb-4">
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
                        <Label htmlFor="settingsName">Instance Name</Label>
                        <Input
                          id="settingsName"
                          value={settingsName}
                          onChange={(e) => setSettingsName(e.target.value)}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>Minecraft Version</Label>
                        <Input value={instance?.minecraft_version || ""} disabled />
                      </div>
                      <div className="space-y-2">
                        <Label>Mod Loader</Label>
                        <Input 
                          value={instance?.mod_loader !== "Vanilla" 
                            ? `${instance?.mod_loader}${instance?.mod_loader_version ? ` ${instance?.mod_loader_version}` : ""}`
                            : "Vanilla"} 
                          disabled 
                        />
                      </div>
                    </CardContent>
                  </Card>
                </TabsContent>

                <TabsContent value="java">
                  <Card>
                    <CardHeader>
                      <CardTitle>Java Settings</CardTitle>
                      <CardDescription>Configure Java runtime settings for this instance.</CardDescription>
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
                          placeholder="-XX:+UseG1GC"
                        />
                      </div>
                    </CardContent>
                  </Card>
                </TabsContent>

                <TabsContent value="memory">
                  <Card>
                    <CardHeader>
                      <CardTitle>Memory Settings</CardTitle>
                      <CardDescription>Configure how much RAM this instance can use.</CardDescription>
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
                      <CardDescription>Configure the Minecraft window size on launch.</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                      <div className="grid grid-cols-2 gap-4">
                        <div className="space-y-2">
                          <Label htmlFor="windowWidth">Window Width</Label>
                          <Input
                            id="windowWidth"
                            type="number"
                            value={windowWidth}
                            onChange={(e) => setWindowWidth(e.target.value)}
                            min="640"
                          />
                        </div>
                        <div className="space-y-2">
                          <Label htmlFor="windowHeight">Window Height</Label>
                          <Input
                            id="windowHeight"
                            type="number"
                            value={windowHeight}
                            onChange={(e) => setWindowHeight(e.target.value)}
                            min="480"
                          />
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                </TabsContent>
              </Tabs>

              <div className="flex justify-end pt-4">
                <Button onClick={() => alert("Settings save not implemented yet")}>
                  <Save className="mr-2 h-4 w-4" /> Save Settings
                </Button>
              </div>
            </ScrollArea>
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
