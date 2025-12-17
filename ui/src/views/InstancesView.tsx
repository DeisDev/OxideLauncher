import { useEffect, useState, useMemo } from "react";
import { Link, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import {
  Plus, Play, Trash2, Info, Pencil, Folder, Copy, FileOutput, FileInput,
  Image, Settings, Square, FolderTree, Link as LinkIcon, Feather,
  ArrowUpDown, Grid, List, Clock, ChevronDown, ChevronRight
} from "lucide-react";
import { cn } from "@/lib/utils";

// Mod loader icon paths (relative to public or using URL constructor)
const GrassIcon = new URL("../../art/grass.svg", import.meta.url).href;
const FabricIcon = new URL("../../art/fabricmc.svg", import.meta.url).href;
const ForgeIcon = new URL("../../art/forge.svg", import.meta.url).href;
const NeoForgeIcon = new URL("../../art/neoforged.svg", import.meta.url).href;
const QuiltIcon = new URL("../../art/quiltmc.svg", import.meta.url).href;
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
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
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useConfig } from "@/hooks/useConfig";
import { ExportInstanceDialog } from "@/components/dialogs";

interface InstanceInfo {
  id: string;
  name: string;
  minecraft_version: string;
  mod_loader: string;
  mod_loader_version: string | null;
  icon: string | null;
  last_played: string | null;
  total_played_seconds: number;
  group?: string | null;
  date_created?: string;
}

// Helper to get instance icon URL - converts file paths to asset URLs
function getInstanceIconUrl(icon: string | null): string | null {
  if (!icon || icon === "default") {
    return null;
  }
  // If the icon looks like a file path (contains backslash or forward slash with drive letter or starts with /)
  // convert it to an asset URL that Tauri can serve
  if (icon.includes("\\") || icon.includes("/") || icon.match(/^[A-Za-z]:/)) {
    return convertFileSrc(icon);
  }
  return icon;
}

// Helper to get mod loader icon
function getModLoaderIcon(loader: string): { icon: string | null; isLucide: boolean } {
  switch (loader) {
    case "Fabric":
      return { icon: FabricIcon, isLucide: false };
    case "Forge":
      return { icon: ForgeIcon, isLucide: false };
    case "NeoForge":
      return { icon: NeoForgeIcon, isLucide: false };
    case "Quilt":
      return { icon: QuiltIcon, isLucide: false };
    case "LiteLoader":
      return { icon: null, isLucide: true };
    case "Vanilla":
    default:
      return { icon: GrassIcon, isLucide: false };
  }
}

// Format play time in human readable format
function formatPlayTime(seconds: number): string {
  if (seconds === 0) return "Never played";
  
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  
  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  if (minutes > 0) {
    return `${minutes}m`;
  }
  return "< 1m";
}

// Format last played date
function formatLastPlayed(dateString: string | null): string {
  if (!dateString) return "Never";
  
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
  
  if (diffDays === 0) return "Today";
  if (diffDays === 1) return "Yesterday";
  if (diffDays < 7) return `${diffDays} days ago`;
  if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
  if (diffDays < 365) return `${Math.floor(diffDays / 30)} months ago`;
  return `${Math.floor(diffDays / 365)} years ago`;
}

export function InstancesView() {
  const navigate = useNavigate();
  const { config, updateUiConfig } = useConfig();
  const [instances, setInstances] = useState<InstanceInfo[]>([]);
  const [loading, setLoading] = useState(true);
  
  // Use config values directly
  const sortBy = config?.ui.instance_sort_by || "name";
  const sortAsc = config?.ui.instance_sort_asc ?? true;
  const gridSize = config?.ui.instance_grid_size || "medium";
  const viewMode = config?.ui.instance_view || "Grid";
  const showGameTime = config?.minecraft.show_game_time ?? true;
  
  // Dialog states
  const [renameDialogOpen, setRenameDialogOpen] = useState(false);
  const [groupDialogOpen, setGroupDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [exportDialogOpen, setExportDialogOpen] = useState(false);
  const [shortcutDialogOpen, setShortcutDialogOpen] = useState(false);
  const [selectedInstance, setSelectedInstance] = useState<string | null>(null);
  const [selectedInstanceName, setSelectedInstanceName] = useState<string>("");
  const [renameName, setRenameName] = useState("");
  const [groupName, setGroupName] = useState("");
  
  // Drag and drop state
  const [draggedInstance, setDraggedInstance] = useState<string | null>(null);
  const [dropTargetGroup, setDropTargetGroup] = useState<string | null>(null);

  // Get existing groups for dropdown
  const existingGroups = useMemo(() => {
    const groups = new Set<string>();
    for (const instance of instances) {
      if (instance.group) {
        groups.add(instance.group);
      }
    }
    return Array.from(groups).sort();
  }, [instances]);

  useEffect(() => {
    loadInstances();
    
    // Listen for instances-changed event from dialog windows
    let unlisten: UnlistenFn | undefined;
    (async () => {
      unlisten = await listen("instances-changed", () => {
        loadInstances();
      });
    })();
    
    return () => {
      unlisten?.();
    };
  }, []);

  const loadInstances = async () => {
    try {
      const data = await invoke<InstanceInfo[]>("get_instances");
      setInstances(data);
    } catch (error) {
      console.error("Failed to load instances:", error);
    } finally {
      setLoading(false);
    }
  };

  const launchInstance = async (id: string) => {
    try {
      await invoke("launch_instance", { instanceId: id });
      // Navigate to instance details with log tab if show_console is enabled
      if (config?.minecraft.show_console) {
        navigate(`/instance/${id}?tab=log`);
      }
    } catch (error) {
      console.error("Failed to launch instance:", error);
    }
  };

  const handleDelete = async () => {
    if (!selectedInstance) return;
    try {
      await invoke("delete_instance", { instanceId: selectedInstance });
      loadInstances();
    } catch (error) {
      console.error("Failed to delete instance:", error);
    } finally {
      setDeleteDialogOpen(false);
      setSelectedInstance(null);
    }
  };

  const handleRename = async () => {
    if (!selectedInstance || !renameName.trim()) return;
    try {
      await invoke("rename_instance", {
        instanceId: selectedInstance,
        newName: renameName,
      });
      loadInstances();
    } catch (error) {
      console.error("Failed to rename instance:", error);
    } finally {
      setRenameDialogOpen(false);
      setSelectedInstance(null);
      setRenameName("");
    }
  };

  const handleCopy = async (instanceId: string) => {
    try {
      await invoke("copy_instance", { instanceId });
      loadInstances();
    } catch (error) {
      console.error("Failed to copy instance:", error);
    }
  };

  const handleChangeGroup = async () => {
    if (!selectedInstance) return;
    try {
      await invoke("change_instance_group", {
        instanceId: selectedInstance,
        group: groupName || null,
      });
      loadInstances();
    } catch (error) {
      console.error("Failed to change group:", error);
    } finally {
      setGroupDialogOpen(false);
      setSelectedInstance(null);
      setGroupName("");
    }
  };

  // Drag and drop handlers
  const handleDragStart = (e: React.DragEvent, instanceId: string) => {
    setDraggedInstance(instanceId);
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", instanceId);
  };

  const handleDragEnd = () => {
    setDraggedInstance(null);
    setDropTargetGroup(null);
  };

  const handleDragOver = (e: React.DragEvent, groupName: string) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
    if (dropTargetGroup !== groupName) {
      setDropTargetGroup(groupName);
    }
  };

  const handleDragLeave = () => {
    setDropTargetGroup(null);
  };

  const handleDrop = async (e: React.DragEvent, targetGroup: string) => {
    e.preventDefault();
    const instanceId = e.dataTransfer.getData("text/plain");
    
    if (!instanceId) return;
    
    // Find the instance to check its current group
    const instance = instances.find(i => i.id === instanceId);
    if (!instance) return;
    
    // Determine the new group value
    const newGroup = targetGroup === "Ungrouped" ? null : targetGroup;
    
    // Only update if the group is different
    if (instance.group !== newGroup && (instance.group || null) !== newGroup) {
      try {
        await invoke("change_instance_group", {
          instanceId,
          group: newGroup,
        });
        loadInstances();
      } catch (error) {
        console.error("Failed to change group:", error);
      }
    }
    
    setDraggedInstance(null);
    setDropTargetGroup(null);
  };

  const handleOpenFolder = async (instanceId: string) => {
    try {
      await invoke("open_instance_folder", { instanceId });
    } catch (error) {
      console.error("Failed to open folder:", error);
    }
  };

  const handleCreateShortcut = async (instanceId: string, location: "desktop" | "start_menu") => {
    try {
      await invoke("create_instance_shortcut", { instanceId, location });
      setShortcutDialogOpen(false);
      setSelectedInstance(null);
    } catch (error) {
      console.error("Failed to create shortcut:", error);
      alert("Failed to create shortcut: " + error);
    }
  };

  const openShortcutDialog = (instance: InstanceInfo) => {
    setSelectedInstance(instance.id);
    setSelectedInstanceName(instance.name);
    setShortcutDialogOpen(true);
  };

  const handleKill = async (instanceId: string) => {
    try {
      await invoke("kill_instance", { instanceId });
    } catch (error) {
      console.error("Failed to kill instance:", error);
    }
  };

  const openRenameDialog = (instance: InstanceInfo) => {
    setSelectedInstance(instance.id);
    setRenameName(instance.name);
    setRenameDialogOpen(true);
  };

  const openGroupDialog = (instance: InstanceInfo) => {
    setSelectedInstance(instance.id);
    setGroupName(instance.group || "");
    setGroupDialogOpen(true);
  };

  const openDeleteDialog = (instanceId: string) => {
    setSelectedInstance(instanceId);
    setDeleteDialogOpen(true);
  };

  const openExportDialog = (instance: InstanceInfo) => {
    setSelectedInstance(instance.id);
    setSelectedInstanceName(instance.name);
    setExportDialogOpen(true);
  };

  // Grid size classes based on settings
  const gridClasses = useMemo(() => {
    switch (gridSize) {
      case "small":
        return "grid-cols-[repeat(auto-fill,minmax(160px,1fr))]";
      case "large":
        return "grid-cols-[repeat(auto-fill,minmax(280px,1fr))]";
      case "medium":
      default:
        return "grid-cols-[repeat(auto-fill,minmax(200px,1fr))]";
    }
  }, [gridSize]);

  // Sorted instances based on settings
  const sortedInstances = useMemo(() => {
    const sorted = [...instances].sort((a, b) => {
      let comparison = 0;
      
      switch (sortBy) {
        case "name":
          comparison = a.name.localeCompare(b.name);
          break;
        case "last_played":
          const aTime = a.last_played ? new Date(a.last_played).getTime() : 0;
          const bTime = b.last_played ? new Date(b.last_played).getTime() : 0;
          comparison = bTime - aTime; // More recent first by default
          break;
        case "date_created":
          const aCreated = a.date_created ? new Date(a.date_created).getTime() : 0;
          const bCreated = b.date_created ? new Date(b.date_created).getTime() : 0;
          comparison = bCreated - aCreated; // More recent first by default
          break;
        case "minecraft_version":
          comparison = a.minecraft_version.localeCompare(b.minecraft_version, undefined, { numeric: true });
          break;
        default:
          comparison = a.name.localeCompare(b.name);
      }
      
      return sortAsc ? comparison : -comparison;
    });
    
    return sorted;
  }, [instances, sortBy, sortAsc]);

  // Group sorted instances by their group property
  const groupedInstances = useMemo(() => {
    const groups: { [key: string]: InstanceInfo[] } = {};
    
    for (const instance of sortedInstances) {
      const groupKey = instance.group || "Ungrouped";
      if (!groups[groupKey]) {
        groups[groupKey] = [];
      }
      groups[groupKey].push(instance);
    }
    
    // Sort group names, keeping "Ungrouped" at the end
    const sortedGroupNames = Object.keys(groups).sort((a, b) => {
      if (a === "Ungrouped") return 1;
      if (b === "Ungrouped") return -1;
      return a.localeCompare(b);
    });
    
    return sortedGroupNames.map(name => ({
      name,
      instances: groups[name],
      isUngrouped: name === "Ungrouped",
    }));
  }, [sortedInstances]);

  // Collapsed groups state
  const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(new Set());

  const toggleGroupCollapse = (groupName: string) => {
    setCollapsedGroups(prev => {
      const next = new Set(prev);
      if (next.has(groupName)) {
        next.delete(groupName);
      } else {
        next.add(groupName);
      }
      return next;
    });
  };

  const getSortLabel = (sort: string) => {
    switch (sort) {
      case "name": return "Name";
      case "last_played": return "Last Played";
      case "date_created": return "Date Created";
      case "minecraft_version": return "Version";
      default: return "Name";
    }
  };

  const handleSortChange = (newSortBy: string) => {
    if (newSortBy === sortBy) {
      // Toggle direction if clicking same sort option
      updateUiConfig({ instance_sort_asc: !sortAsc });
    } else {
      updateUiConfig({ instance_sort_by: newSortBy, instance_sort_asc: true });
    }
  };

  const handleViewModeChange = (mode: "Grid" | "List") => {
    updateUiConfig({ instance_view: mode });
  };

  const handleGridSizeChange = (size: string) => {
    updateUiConfig({ instance_grid_size: size });
  };

  // Skeleton loading component that maintains layout consistency
  const LoadingSkeleton = () => (
    <div className="w-full">
      <div className="flex flex-wrap justify-between items-center gap-4 mb-8 pb-5 border-b border-border">
        <div className="skeleton h-9 w-64" />
        <div className="skeleton h-10 w-40" />
      </div>
      <div className="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-4">
        {[1, 2, 3, 4, 5, 6].map((i) => (
          <Card key={i} className="overflow-hidden">
            <div className="skeleton h-28" />
            <CardContent className="p-3 space-y-2">
              <div className="skeleton h-5 w-3/4" />
              <div className="skeleton h-4 w-1/2" />
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );

  // Instance card component for grid view
  const InstanceCard = ({ instance }: { instance: InstanceInfo }) => {
    const loaderInfo = getModLoaderIcon(instance.mod_loader);
    const iconUrl = getInstanceIconUrl(instance.icon);
    
    // Color coding for mod loaders
    const getLoaderColor = (loader: string) => {
      switch (loader.toLowerCase()) {
        case "fabric": return "text-amber-600 dark:text-amber-400";
        case "forge": return "text-blue-600 dark:text-blue-400";
        case "neoforge": return "text-orange-600 dark:text-orange-400";
        case "quilt": return "text-purple-600 dark:text-purple-400";
        case "liteloader": return "text-cyan-600 dark:text-cyan-400";
        default: return "text-muted-foreground";
      }
    };
    
    return (
      <ContextMenu>
        <ContextMenuTrigger asChild>
          <Card 
            className={cn(
              "overflow-hidden cursor-pointer transition-all hover:shadow-xl hover:border-primary/50 h-full flex flex-col cursor-grab active:cursor-grabbing",
              draggedInstance === instance.id && "opacity-50 scale-95"
            )}
            draggable
            onDragStart={(e) => handleDragStart(e, instance.id)}
            onDragEnd={handleDragEnd}
          >
            {/* Fixed height image container */}
            <div className="h-24 sm:h-28 flex items-center justify-center bg-gradient-to-br from-muted to-card overflow-hidden flex-shrink-0">
              {iconUrl ? (
                <img
                  src={iconUrl}
                  alt={instance.name}
                  className="w-full h-full object-cover"
                />
              ) : (
                <div className="w-full h-full flex items-center justify-center bg-gradient-to-br from-primary/10 to-primary/5">
                  <img
                    src={GrassIcon}
                    alt="Minecraft"
                    className="h-10 w-10 sm:h-14 sm:w-14 object-contain opacity-80"
                  />
                </div>
              )}
            </div>
            {/* Fixed height content container */}
            <CardContent className="p-2 sm:p-3 flex flex-col flex-1">
              <h3 className="font-semibold text-xs sm:text-sm mb-1 truncate" title={instance.name}>
                {instance.name}
              </h3>
              <p className="text-xs text-emerald-600 dark:text-emerald-400 font-medium">
                {instance.minecraft_version}
              </p>
              <div className={cn("flex items-center gap-1 text-xs", getLoaderColor(instance.mod_loader))}>
                {loaderInfo.isLucide ? (
                  <Feather className="h-3 w-3 flex-shrink-0" />
                ) : loaderInfo.icon && instance.mod_loader !== "Vanilla" ? (
                  <img src={loaderInfo.icon} alt={instance.mod_loader} className="h-3 w-3 flex-shrink-0" />
                ) : null}
                <span className="truncate">
                  {instance.mod_loader !== "Vanilla" 
                    ? `${instance.mod_loader}${instance.mod_loader_version ? ` ${instance.mod_loader_version}` : ""}`
                    : "Vanilla"}
                </span>
              </div>
              {/* Play time */}
              {showGameTime && (
                <div className="flex items-center gap-1 text-xs text-muted-foreground mt-1">
                  <Clock className="h-3 w-3 flex-shrink-0" />
                  <span>{formatPlayTime(instance.total_played_seconds)}</span>
                </div>
              )}
              {/* Buttons always at bottom */}
              <div className="flex gap-1 mt-auto pt-2">
                <Button
                  size="sm"
                  className="h-6 sm:h-7 text-xs px-2 flex-1"
                  onClick={(e) => {
                    e.stopPropagation();
                    launchInstance(instance.id);
                  }}
                >
                  <Play className="mr-1 h-3 w-3" /> Play
                </Button>
                <Link to={`/instance/${instance.id}`} onClick={(e) => e.stopPropagation()}>
                  <Button size="sm" variant="secondary" className="h-6 sm:h-7 px-2">
                    <Info className="h-3 w-3" />
                  </Button>
                </Link>
              </div>
            </CardContent>
          </Card>
        </ContextMenuTrigger>
        <InstanceContextMenu instance={instance} />
      </ContextMenu>
    );
  };

  // Instance row component for list view
  const InstanceRow = ({ instance }: { instance: InstanceInfo }) => {
    const loaderInfo = getModLoaderIcon(instance.mod_loader);
    const iconUrl = getInstanceIconUrl(instance.icon);
    
    // Color coding for mod loaders
    const getLoaderColor = (loader: string) => {
      switch (loader.toLowerCase()) {
        case "fabric": return "text-amber-600 dark:text-amber-400";
        case "forge": return "text-blue-600 dark:text-blue-400";
        case "neoforge": return "text-orange-600 dark:text-orange-400";
        case "quilt": return "text-purple-600 dark:text-purple-400";
        case "liteloader": return "text-cyan-600 dark:text-cyan-400";
        default: return "text-muted-foreground";
      }
    };
    
    return (
      <ContextMenu>
        <ContextMenuTrigger asChild>
          <div 
            className={cn(
              "flex items-center gap-4 p-3 rounded-lg border bg-card hover:bg-accent/50 transition-colors cursor-grab active:cursor-grabbing",
              draggedInstance === instance.id && "opacity-50 scale-95"
            )}
            draggable
            onDragStart={(e) => handleDragStart(e, instance.id)}
            onDragEnd={handleDragEnd}
          >
            {/* Icon */}
            <div className="h-12 w-12 flex-shrink-0 rounded-lg overflow-hidden bg-gradient-to-br from-muted to-card flex items-center justify-center">
              {iconUrl ? (
                <img
                  src={iconUrl}
                  alt={instance.name}
                  className="w-full h-full object-cover"
                />
              ) : (
                <img
                  src={GrassIcon}
                  alt="Minecraft"
                  className="h-8 w-8 object-contain opacity-80"
                />
              )}
            </div>
            
            {/* Info */}
            <div className="flex-1 min-w-0">
              <h3 className="font-semibold text-sm truncate">{instance.name}</h3>
              <div className="flex items-center gap-3 text-xs">
                <span className="text-emerald-600 dark:text-emerald-400 font-medium">{instance.minecraft_version}</span>
                <span className={cn("flex items-center gap-1", getLoaderColor(instance.mod_loader))}>
                  {loaderInfo.isLucide ? (
                    <Feather className="h-3 w-3" />
                  ) : loaderInfo.icon && instance.mod_loader !== "Vanilla" ? (
                    <img src={loaderInfo.icon} alt={instance.mod_loader} className="h-3 w-3" />
                  ) : null}
                  {instance.mod_loader !== "Vanilla" ? instance.mod_loader : "Vanilla"}
                </span>
                {showGameTime && (
                  <span className="flex items-center gap-1 text-muted-foreground">
                    <Clock className="h-3 w-3" />
                    {formatPlayTime(instance.total_played_seconds)}
                  </span>
                )}
                <span className="text-muted-foreground">Last played: {formatLastPlayed(instance.last_played)}</span>
              </div>
            </div>
            
            {/* Actions */}
            <div className="flex gap-2 flex-shrink-0">
              <Button
                size="sm"
                onClick={(e) => {
                  e.stopPropagation();
                  launchInstance(instance.id);
                }}
              >
                <Play className="mr-1 h-4 w-4" /> Play
              </Button>
              <Link to={`/instance/${instance.id}`} onClick={(e) => e.stopPropagation()}>
                <Button size="sm" variant="outline">
                  <Info className="h-4 w-4" />
                </Button>
              </Link>
            </div>
          </div>
        </ContextMenuTrigger>
        <InstanceContextMenu instance={instance} />
      </ContextMenu>
    );
  };

  // Shared context menu for instances
  const InstanceContextMenu = ({ instance }: { instance: InstanceInfo }) => (
    <ContextMenuContent className="w-56">
      <ContextMenuItem onClick={() => launchInstance(instance.id)}>
        <Play className="mr-2 h-4 w-4" />
        Launch
      </ContextMenuItem>
      <ContextMenuItem onClick={() => handleKill(instance.id)}>
        <Square className="mr-2 h-4 w-4" />
        Kill
      </ContextMenuItem>
      <ContextMenuSeparator />
      <ContextMenuItem onClick={() => openRenameDialog(instance)}>
        <Pencil className="mr-2 h-4 w-4" />
        Rename
      </ContextMenuItem>
      <ContextMenuItem>
        <Image className="mr-2 h-4 w-4" />
        Change Icon
      </ContextMenuItem>
      <ContextMenuItem onClick={() => navigate(`/instance/${instance.id}`)}>
        <Settings className="mr-2 h-4 w-4" />
        Edit...
      </ContextMenuItem>
      <ContextMenuItem onClick={() => openGroupDialog(instance)}>
        <FolderTree className="mr-2 h-4 w-4" />
        Change Group...
      </ContextMenuItem>
      <ContextMenuSeparator />
      <ContextMenuItem onClick={() => handleOpenFolder(instance.id)}>
        <Folder className="mr-2 h-4 w-4" />
        Folder
      </ContextMenuItem>
      <ContextMenuItem onClick={() => openExportDialog(instance)}>
        <FileOutput className="mr-2 h-4 w-4" />
        Export
      </ContextMenuItem>
      <ContextMenuItem onClick={() => handleCopy(instance.id)}>
        <Copy className="mr-2 h-4 w-4" />
        Copy
      </ContextMenuItem>
      <ContextMenuItem onClick={() => openShortcutDialog(instance)}>
        <LinkIcon className="mr-2 h-4 w-4" />
        Create Shortcut
      </ContextMenuItem>
      <ContextMenuSeparator />
      <ContextMenuItem
        className="text-destructive focus:text-destructive"
        onClick={() => openDeleteDialog(instance.id)}
      >
        <Trash2 className="mr-2 h-4 w-4" />
        Delete
      </ContextMenuItem>
    </ContextMenuContent>
  );

  if (loading) {
    return <LoadingSkeleton />;
  }

  return (
    <div className="w-full h-full flex flex-col overflow-hidden">
      <div className="flex flex-col sm:flex-row flex-wrap justify-between items-start sm:items-center gap-3 mb-4 pb-4 border-b border-border flex-shrink-0">
        <h1 className="text-2xl sm:text-3xl font-bold bg-gradient-to-r from-foreground to-muted-foreground bg-clip-text text-transparent">
          Minecraft Instances
        </h1>
        <div className="flex flex-wrap items-center gap-2">
          {/* View Mode Toggle */}
          <div className="flex border rounded-lg overflow-hidden">
            <Button
              variant={viewMode === "Grid" ? "default" : "ghost"}
              size="sm"
              className="rounded-none h-8 sm:h-9 px-2 sm:px-3"
              onClick={() => handleViewModeChange("Grid")}
            >
              <Grid className="h-4 w-4" />
            </Button>
            <Button
              variant={viewMode === "List" ? "default" : "ghost"}
              size="sm"
              className="rounded-none h-8 sm:h-9 px-2 sm:px-3"
              onClick={() => handleViewModeChange("List")}
            >
              <List className="h-4 w-4" />
            </Button>
          </div>

          {/* Grid Size (only in grid view) */}
          {viewMode === "Grid" && (
            <Select value={gridSize} onValueChange={handleGridSizeChange}>
              <SelectTrigger className="w-24 sm:w-28 h-8 sm:h-9 text-xs sm:text-sm">
                <SelectValue placeholder="Size" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="small">Small</SelectItem>
                <SelectItem value="medium">Medium</SelectItem>
                <SelectItem value="large">Large</SelectItem>
              </SelectContent>
            </Select>
          )}

          {/* Sort Dropdown */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="sm" className="h-8 sm:h-9 text-xs sm:text-sm">
                <ArrowUpDown className="mr-1 sm:mr-2 h-3 w-3 sm:h-4 sm:w-4" />
                <span className="hidden sm:inline">{getSortLabel(sortBy)}</span> {sortAsc ? "↑" : "↓"}
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={() => handleSortChange("name")}>
                Name {sortBy === "name" && (sortAsc ? "↑" : "↓")}
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => handleSortChange("last_played")}>
                Last Played {sortBy === "last_played" && (sortAsc ? "↑" : "↓")}
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => handleSortChange("date_created")}>
                Date Created {sortBy === "date_created" && (sortAsc ? "↑" : "↓")}
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => handleSortChange("minecraft_version")}>
                Minecraft Version {sortBy === "minecraft_version" && (sortAsc ? "↑" : "↓")}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>

          <Button 
            variant="outline" 
            className="h-8 sm:h-9 text-xs sm:text-sm"
            onClick={() => navigate("/create-instance?source=import")}
          >
            <FileInput className="h-3 w-3 sm:h-4 sm:w-4 sm:mr-2" />
            <span className="hidden sm:inline">Import</span>
          </Button>

          <Link to="/create-instance">
            <Button className="h-8 sm:h-9 text-xs sm:text-sm">
              <Plus className="h-3 w-3 sm:h-4 sm:w-4 sm:mr-2" />
              <span className="hidden sm:inline">Create</span>
            </Button>
          </Link>
        </div>
      </div>

      {sortedInstances.length === 0 ? (
        <ContextMenu>
          <ContextMenuTrigger asChild>
            <div className="flex-1 flex flex-col items-center justify-center">
              <p className="mb-4">No instances found. Create your first instance to get started!</p>
              <Link to="/create-instance">
                <Button>
                  <Plus className="mr-2 h-4 w-4" /> Create Instance
                </Button>
              </Link>
            </div>
          </ContextMenuTrigger>
          <ContextMenuContent className="w-48">
            <ContextMenuItem onClick={() => navigate("/create-instance")}>
              <Plus className="mr-2 h-4 w-4" />
              Create Instance
            </ContextMenuItem>
            <ContextMenuItem onClick={() => navigate("/create-instance?source=import")}>
              <FileInput className="mr-2 h-4 w-4" />
              Import Instance
            </ContextMenuItem>
          </ContextMenuContent>
        </ContextMenu>
      ) : (
        <ContextMenu>
          <ContextMenuTrigger asChild>
            <div className="flex-1 overflow-auto pr-1">
              {groupedInstances.map((group) => (
                <div 
                  key={group.name} 
                  className={cn(
                    "mb-3 md:mb-4 rounded-lg transition-colors",
                    dropTargetGroup === group.name && draggedInstance && "bg-primary/10 ring-2 ring-primary/50"
                  )}
                  onDragOver={(e) => handleDragOver(e, group.name)}
                  onDragLeave={handleDragLeave}
                  onDrop={(e) => handleDrop(e, group.name)}
                >
                  {/* Group Header - Show when dragging OR when there are multiple groups */}
                  {(groupedInstances.length > 1 || !group.isUngrouped || draggedInstance) && (
                    <div
                      onClick={() => toggleGroupCollapse(group.name)}
                      className={cn(
                        "flex items-center gap-2 w-full p-2 mb-2 rounded-md hover:bg-muted transition-colors text-left cursor-pointer select-none",
                        dropTargetGroup === group.name && draggedInstance && "bg-primary/20 ring-2 ring-primary/50"
                      )}
                    >
                      {collapsedGroups.has(group.name) ? (
                        <ChevronRight className="h-4 w-4 text-muted-foreground" />
                      ) : (
                        <ChevronDown className="h-4 w-4 text-muted-foreground" />
                      )}
                      <FolderTree className="h-4 w-4 text-muted-foreground" />
                      <span className="font-semibold text-sm md:text-base">{group.name}</span>
                      <span className="text-xs text-muted-foreground">
                        ({group.instances.length})
                      </span>
                    </div>
                  )}
                  
                  {/* Group Content */}
                  {!collapsedGroups.has(group.name) && (
                    viewMode === "Grid" ? (
                      <div className={`grid ${gridClasses} gap-2 md:gap-4 content-start`}>
                        {group.instances.map((instance) => (
                          <InstanceCard key={instance.id} instance={instance} />
                        ))}
                      </div>
                    ) : (
                      <div className="space-y-2">
                        {group.instances.map((instance) => (
                          <InstanceRow key={instance.id} instance={instance} />
                        ))}
                      </div>
                    )
                  )}
                </div>
              ))}
            </div>
          </ContextMenuTrigger>
          <ContextMenuContent className="w-48">
            <ContextMenuItem onClick={() => navigate("/create-instance")}>
              <Plus className="mr-2 h-4 w-4" />
              Create Instance
            </ContextMenuItem>
            <ContextMenuItem onClick={() => navigate("/create-instance?source=import")}>
              <FileInput className="mr-2 h-4 w-4" />
              Import Instance
            </ContextMenuItem>
          </ContextMenuContent>
        </ContextMenu>
      )}

      {/* Rename Dialog */}
      <Dialog open={renameDialogOpen} onOpenChange={setRenameDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Rename Instance</DialogTitle>
            <DialogDescription>
              Enter a new name for this instance.
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Label htmlFor="name">Name</Label>
            <Input
              id="name"
              value={renameName}
              onChange={(e) => setRenameName(e.target.value)}
              placeholder="Enter new name"
              onKeyDown={(e) => {
                if (e.key === "Enter") handleRename();
              }}
            />
          </div>
          <DialogFooter>
            <Button variant="secondary" onClick={() => setRenameDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleRename}>Rename</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Group Dialog */}
      <Dialog open={groupDialogOpen} onOpenChange={setGroupDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Change Group</DialogTitle>
            <DialogDescription>
              Select an existing group or enter a new group name. Leave empty to ungroup.
            </DialogDescription>
          </DialogHeader>
          <div className="py-4 space-y-4">
            {/* Existing Groups Dropdown */}
            {existingGroups.length > 0 && (
              <div>
                <Label className="mb-2 block">Existing Groups</Label>
                <div className="flex flex-wrap gap-2">
                  <Button
                    variant={groupName === "" ? "default" : "outline"}
                    size="sm"
                    onClick={() => setGroupName("")}
                  >
                    Ungrouped
                  </Button>
                  {existingGroups.map((g) => (
                    <Button
                      key={g}
                      variant={groupName === g ? "default" : "outline"}
                      size="sm"
                      onClick={() => setGroupName(g)}
                    >
                      {g}
                    </Button>
                  ))}
                </div>
              </div>
            )}
            
            {/* Custom Group Input */}
            <div>
              <Label htmlFor="group">{existingGroups.length > 0 ? "Or enter a new group name" : "Group Name"}</Label>
              <Input
                id="group"
                value={groupName}
                onChange={(e) => setGroupName(e.target.value)}
                placeholder="Enter group name (or leave empty)"
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleChangeGroup();
                }}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="secondary" onClick={() => setGroupDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleChangeGroup}>Change Group</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation */}
      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Instance?</AlertDialogTitle>
            <AlertDialogDescription>
              This action cannot be undone. This will permanently delete the instance
              and all its data.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDelete}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Shortcut Dialog */}
      <Dialog open={shortcutDialogOpen} onOpenChange={setShortcutDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create Shortcut</DialogTitle>
            <DialogDescription>
              Create a shortcut to launch "{selectedInstanceName}" directly.
            </DialogDescription>
          </DialogHeader>
          <div className="py-4 space-y-3">
            <Button 
              className="w-full justify-start" 
              variant="outline"
              onClick={() => selectedInstance && handleCreateShortcut(selectedInstance, "desktop")}
            >
              <LinkIcon className="mr-2 h-4 w-4" />
              Desktop Shortcut
            </Button>
            <Button 
              className="w-full justify-start" 
              variant="outline"
              onClick={() => selectedInstance && handleCreateShortcut(selectedInstance, "start_menu")}
            >
              <LinkIcon className="mr-2 h-4 w-4" />
              Start Menu Shortcut
            </Button>
          </div>
          <DialogFooter>
            <Button variant="secondary" onClick={() => setShortcutDialogOpen(false)}>
              Cancel
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Export Instance Dialog */}
      {selectedInstance && (
        <ExportInstanceDialog
          open={exportDialogOpen}
          onOpenChange={setExportDialogOpen}
          instanceId={selectedInstance}
          instanceName={selectedInstanceName}
        />
      )}
    </div>
  );
}
