// Version tab component for managing instance version and mod loader
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

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  FolderOpen,
  Library,
  ChevronUp,
  ChevronDown,
  Trash2,
  Edit,
  Undo2,
  Download,
  Plus,
  FileBox,
  Bot,
  Import,
  RefreshCw,
  Wrench,
  AlertTriangle,
  XCircle,
  CheckCircle,
  Gamepad2,
  Cog,
  Map,
  Package,
  HelpCircle,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
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
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";

interface Component {
  uid: string;
  name: string;
  version: string;
  component_type: string;
  enabled: boolean;
  removable: boolean;
  version_changeable: boolean;
  customizable: boolean;
  revertible: boolean;
  custom: boolean;
  order: number;
  problems: ComponentProblem[];
}

interface ComponentProblem {
  severity: string;
  description: string;
}

interface VersionTabProps {
  instanceId: string;
}

interface LoaderVersion {
  version: string;
  recommended: boolean;
}

interface MinecraftVersion {
  id: string;
  version_type: string;
  release_time: string;
}

interface AgentConfig {
  file: string;
  args: string | null;
}

export function VersionTab({ instanceId }: VersionTabProps) {
  const [components, setComponents] = useState<Component[]>([]);
  const [selectedComponent, setSelectedComponent] = useState<Component | null>(null);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  
  // Dialogs
  const [installLoaderDialog, setInstallLoaderDialog] = useState(false);
  const [changeVersionDialog, setChangeVersionDialog] = useState<Component | null>(null);
  const [removeDialog, setRemoveDialog] = useState<Component | null>(null);
  const [jarModDialog, setJarModDialog] = useState(false);
  const [agentDialog, setAgentDialog] = useState(false);
  const [replaceJarDialog, setReplaceJarDialog] = useState(false);
  const [addEmptyDialog, setAddEmptyDialog] = useState(false);
  const [newComponentName, setNewComponentName] = useState("");
  
  // Jar mods
  const [jarMods, setJarMods] = useState<string[]>([]);
  const [agents, setAgents] = useState<AgentConfig[]>([]);
  const [hasCustomJar, setHasCustomJar] = useState(false);
  const [agentArgs, setAgentArgs] = useState("");
  
  // Loader installation
  const [selectedLoaderType, setSelectedLoaderType] = useState<string>("fabric");
  const [loaderVersions, setLoaderVersions] = useState<LoaderVersion[]>([]);
  const [selectedLoaderVersion, setSelectedLoaderVersion] = useState<string>("");
  const [loadingLoaderVersions, setLoadingLoaderVersions] = useState(false);
  const [installingLoader, setInstallingLoader] = useState(false);
  
  // Version change
  const [availableVersions, setAvailableVersions] = useState<string[]>([]);
  const [selectedNewVersion, setSelectedNewVersion] = useState<string>("");
  const [loadingVersions, setLoadingVersions] = useState(false);
  const [changingVersion, setChangingVersion] = useState(false);

  useEffect(() => {
    loadComponents();
    loadJarMods();
    loadAgents();
    checkCustomJar();
  }, [instanceId]);

  const loadComponents = async () => {
    setLoading(true);
    try {
      const comps = await invoke<Component[]>("get_instance_components", {
        instanceId,
      });
      setComponents(comps);
      if (comps.length > 0 && !selectedComponent) {
        setSelectedComponent(comps[0]);
      }
    } catch (error) {
      console.error("Failed to load components:", error);
    } finally {
      setLoading(false);
    }
  };
  
  const loadJarMods = async () => {
    try {
      const mods = await invoke<string[]>("get_jar_mods", { instanceId });
      setJarMods(mods);
    } catch (error) {
      console.error("Failed to load jar mods:", error);
    }
  };
  
  const loadAgents = async () => {
    try {
      const agentList = await invoke<AgentConfig[]>("get_java_agents", { instanceId });
      setAgents(agentList);
    } catch (error) {
      console.error("Failed to load agents:", error);
    }
  };
  
  const checkCustomJar = async () => {
    try {
      const hasCustom = await invoke<boolean>("has_custom_minecraft_jar", { instanceId });
      setHasCustomJar(hasCustom);
    } catch (error) {
      console.error("Failed to check custom jar:", error);
    }
  };
  const refreshComponents = async () => {
    setRefreshing(true);
    await loadComponents();
    setRefreshing(false);
  };

  const openMinecraftFolder = async () => {
    try {
      await invoke("open_minecraft_folder", { instanceId });
    } catch (error) {
      console.error("Failed to open folder:", error);
    }
  };

  const openLibrariesFolder = async () => {
    try {
      await invoke("open_libraries_folder", { instanceId });
    } catch (error) {
      console.error("Failed to open folder:", error);
    }
  };

  const handleRemoveComponent = async () => {
    if (!removeDialog) return;
    
    try {
      await invoke("remove_instance_component", {
        instanceId,
        componentUid: removeDialog.uid,
      });
      setRemoveDialog(null);
      await loadComponents();
    } catch (error) {
      console.error("Failed to remove component:", error);
      alert("Failed to remove component: " + error);
    }
  };

  const openInstallLoaderDialog = async () => {
    setInstallLoaderDialog(true);
    setSelectedLoaderType("fabric");
    setLoaderVersions([]);
    setSelectedLoaderVersion("");
    await loadLoaderVersions("fabric");
  };

  const loadLoaderVersions = async (loaderType: string) => {
    setLoadingLoaderVersions(true);
    setLoaderVersions([]);
    setSelectedLoaderVersion("");
    
    try {
      // Get minecraft version from the Minecraft component
      const mcComponent = components.find(c => c.component_type === "minecraft");
      const mcVersion = mcComponent?.version || "";
      
      let versions: LoaderVersion[] = [];
      switch (loaderType) {
        case "fabric":
          versions = await invoke<LoaderVersion[]>("get_fabric_versions", { minecraftVersion: mcVersion });
          break;
        case "quilt":
          versions = await invoke<LoaderVersion[]>("get_quilt_versions", { minecraftVersion: mcVersion });
          break;
        case "forge":
          versions = await invoke<LoaderVersion[]>("get_forge_versions", { minecraftVersion: mcVersion });
          break;
        case "neoforge":
          versions = await invoke<LoaderVersion[]>("get_neoforge_versions", { minecraftVersion: mcVersion });
          break;
      }
      
      setLoaderVersions(versions);
      const recommended = versions.find(v => v.recommended);
      if (recommended) {
        setSelectedLoaderVersion(recommended.version);
      } else if (versions.length > 0) {
        setSelectedLoaderVersion(versions[0].version);
      }
    } catch (error) {
      console.error("Failed to load loader versions:", error);
    } finally {
      setLoadingLoaderVersions(false);
    }
  };

  const handleInstallLoader = async () => {
    if (!selectedLoaderVersion) return;
    
    setInstallingLoader(true);
    try {
      await invoke("install_mod_loader", {
        instanceId,
        loaderType: selectedLoaderType,
        loaderVersion: selectedLoaderVersion,
      });
      setInstallLoaderDialog(false);
      await loadComponents();
    } catch (error) {
      console.error("Failed to install loader:", error);
      alert("Failed to install loader: " + error);
    } finally {
      setInstallingLoader(false);
    }
  };

  const openChangeVersionDialog = async (component: Component) => {
    setChangeVersionDialog(component);
    setLoadingVersions(true);
    setAvailableVersions([]);
    setSelectedNewVersion("");
    
    try {
      if (component.component_type === "minecraft") {
        const versions = await invoke<MinecraftVersion[]>("get_minecraft_versions", {
          showReleases: true,
          showSnapshots: true,
          showBetas: false,
          showAlphas: false,
          showExperimental: false,
        });
        setAvailableVersions(versions.map(v => v.id));
        setSelectedNewVersion(component.version);
      } else if (component.component_type === "mod_loader") {
        const mcComponent = components.find(c => c.component_type === "minecraft");
        const mcVersion = mcComponent?.version || "";
        
        let versions: LoaderVersion[] = [];
        switch (component.uid) {
          case "net.fabricmc.fabric-loader":
            versions = await invoke<LoaderVersion[]>("get_fabric_versions", { minecraftVersion: mcVersion });
            break;
          case "org.quiltmc.quilt-loader":
            versions = await invoke<LoaderVersion[]>("get_quilt_versions", { minecraftVersion: mcVersion });
            break;
          case "net.minecraftforge":
            versions = await invoke<LoaderVersion[]>("get_forge_versions", { minecraftVersion: mcVersion });
            break;
          case "net.neoforged":
            versions = await invoke<LoaderVersion[]>("get_neoforge_versions", { minecraftVersion: mcVersion });
            break;
        }
        setAvailableVersions(versions.map(v => v.version));
        setSelectedNewVersion(component.version);
      }
    } catch (error) {
      console.error("Failed to load versions:", error);
    } finally {
      setLoadingVersions(false);
    }
  };

  const handleChangeVersion = async () => {
    if (!changeVersionDialog || !selectedNewVersion) return;
    
    setChangingVersion(true);
    try {
      await invoke("change_component_version", {
        instanceId,
        componentUid: changeVersionDialog.uid,
        newVersion: selectedNewVersion,
      });
      setChangeVersionDialog(null);
      await loadComponents();
    } catch (error) {
      console.error("Failed to change version:", error);
      alert("Failed to change version: " + error);
    } finally {
      setChangingVersion(false);
    }
  };

  // Jar mod handlers
  const handleAddJarMod = async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: "Jar Files", extensions: ["jar", "zip"] }],
      });
      
      if (selected && Array.isArray(selected)) {
        for (const file of selected) {
          await invoke("add_jar_mod", { instanceId, jarPath: file });
        }
        await loadJarMods();
        setJarModDialog(false);
      } else if (selected) {
        await invoke("add_jar_mod", { instanceId, jarPath: selected });
        await loadJarMods();
        setJarModDialog(false);
      }
    } catch (error) {
      console.error("Failed to add jar mod:", error);
      alert("Failed to add jar mod: " + error);
    }
  };
  
  const handleRemoveJarMod = async (filename: string) => {
    try {
      await invoke("remove_jar_mod", { instanceId, filename });
      await loadJarMods();
    } catch (error) {
      console.error("Failed to remove jar mod:", error);
      alert("Failed to remove jar mod: " + error);
    }
  };
  
  // Agent handlers
  const handleAddAgent = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "Jar Files", extensions: ["jar"] }],
      });
      
      if (selected && typeof selected === "string") {
        const args = agentArgs.trim() || null;
        await invoke("add_java_agent", { instanceId, agentPath: selected, agentArgs: args });
        await loadAgents();
        setAgentDialog(false);
        setAgentArgs("");
      }
    } catch (error) {
      console.error("Failed to add agent:", error);
      alert("Failed to add agent: " + error);
    }
  };
  
  const handleRemoveAgent = async (filename: string) => {
    try {
      await invoke("remove_java_agent", { instanceId, filename });
      await loadAgents();
    } catch (error) {
      console.error("Failed to remove agent:", error);
      alert("Failed to remove agent: " + error);
    }
  };
  
  // Custom jar handlers
  const handleReplaceMinecraftJar = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "Jar Files", extensions: ["jar"] }],
      });
      
      if (selected && typeof selected === "string") {
        await invoke("replace_minecraft_jar", { instanceId, jarPath: selected });
        await checkCustomJar();
        setReplaceJarDialog(false);
      }
    } catch (error) {
      console.error("Failed to replace Minecraft jar:", error);
      alert("Failed to replace Minecraft jar: " + error);
    }
  };
  
  const handleRevertMinecraftJar = async () => {
    try {
      await invoke("revert_minecraft_jar", { instanceId });
      await checkCustomJar();
    } catch (error) {
      console.error("Failed to revert Minecraft jar:", error);
      alert("Failed to revert Minecraft jar: " + error);
    }
  };
  
  // Component ordering handlers
  const handleMoveUp = async () => {
    if (!selectedComponent) return;
    
    try {
      await invoke("move_component_up", {
        instanceId,
        componentUid: selectedComponent.uid,
      });
      await loadComponents();
    } catch (error) {
      console.error("Failed to move component up:", error);
      alert("Failed to move component up: " + error);
    }
  };
  
  const handleMoveDown = async () => {
    if (!selectedComponent) return;
    
    try {
      await invoke("move_component_down", {
        instanceId,
        componentUid: selectedComponent.uid,
      });
      await loadComponents();
    } catch (error) {
      console.error("Failed to move component down:", error);
      alert("Failed to move component down: " + error);
    }
  };
  
  const handleCustomize = async () => {
    if (!selectedComponent) return;
    
    try {
      await invoke("customize_component", {
        instanceId,
        componentUid: selectedComponent.uid,
      });
      await loadComponents();
    } catch (error) {
      console.error("Failed to customize component:", error);
      alert("Failed to customize component: " + error);
    }
  };
  
  const handleRevert = async () => {
    if (!selectedComponent) return;
    
    try {
      await invoke("revert_component", {
        instanceId,
        componentUid: selectedComponent.uid,
      });
      await loadComponents();
    } catch (error) {
      console.error("Failed to revert component:", error);
      alert("Failed to revert component: " + error);
    }
  };
  
  // Check if component can move
  const canMoveUp = () => {
    if (!selectedComponent) return false;
    const index = components.findIndex(c => c.uid === selectedComponent.uid);
    return index > 0;
  };
  
  const canMoveDown = () => {
    if (!selectedComponent) return false;
    const index = components.findIndex(c => c.uid === selectedComponent.uid);
    return index >= 0 && index < components.length - 1;
  };
  
  // Add empty component handler
  const handleAddEmpty = async () => {
    if (!newComponentName.trim()) return;
    
    try {
      await invoke("add_empty_component", {
        instanceId,
        name: newComponentName.trim(),
      });
      await loadComponents();
      setAddEmptyDialog(false);
      setNewComponentName("");
    } catch (error) {
      console.error("Failed to add empty component:", error);
      alert("Failed to add empty component: " + error);
    }
  };

  const getComponentIcon = (componentType: string) => {
    const iconClass = "h-4 w-4";
    switch (componentType) {
      case "minecraft":
        return <Gamepad2 className={iconClass} />;
      case "mod_loader":
        return <Cog className={iconClass} />;
      case "mappings":
        return <Map className={iconClass} />;
      case "library":
        return <Library className={iconClass} />;
      case "agent":
        return <Bot className={iconClass} />;
      case "jar_mod":
        return <Package className={iconClass} />;
      default:
        return <HelpCircle className={iconClass} />;
    }
  };

  const getProblemIcon = (severity: string) => {
    switch (severity) {
      case "error":
        return <XCircle className="h-4 w-4 text-destructive" />;
      case "warning":
        return <AlertTriangle className="h-4 w-4 text-yellow-500" />;
      default:
        return <CheckCircle className="h-4 w-4 text-green-500" />;
    }
  };

  const hasModLoader = components.some(c => c.component_type === "mod_loader");

  if (loading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Version Components</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-center py-8">
            <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-4 h-full overflow-hidden">
      {/* Main content area */}
      <div className="flex gap-4 h-full min-h-0">
        {/* Component list */}
        <Card className="flex-1">
          <CardHeader className="pb-2">
            <CardTitle className="text-lg">Components</CardTitle>
            <CardDescription>
              Components and libraries that make up this instance.
            </CardDescription>
          </CardHeader>
          <CardContent className="p-0">
            <ScrollArea className="h-[300px]">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-10"></TableHead>
                    <TableHead>Name</TableHead>
                    <TableHead>Version</TableHead>
                    <TableHead className="w-10"></TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {components.map((component) => (
                    <TableRow
                      key={component.uid}
                      className={cn(
                        "cursor-pointer",
                        selectedComponent?.uid === component.uid && "bg-accent"
                      )}
                      onClick={() => setSelectedComponent(component)}
                    >
                      <TableCell className="text-muted-foreground">
                        {getComponentIcon(component.component_type)}
                      </TableCell>
                      <TableCell>
                        <div className="flex items-center gap-2">
                          <span className={cn(!component.enabled && "text-muted-foreground")}>
                            {component.name}
                          </span>
                          {component.custom && (
                            <Badge variant="outline" className="text-xs">Custom</Badge>
                          )}
                        </div>
                      </TableCell>
                      <TableCell className="font-mono text-sm">
                        {component.version}
                      </TableCell>
                      <TableCell>
                        {component.problems.length > 0 ? (
                          getProblemIcon(component.problems[0].severity)
                        ) : (
                          <CheckCircle className="h-4 w-4 text-green-500" />
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </ScrollArea>
          </CardContent>
        </Card>

        {/* Actions sidebar */}
        <Card className="w-56 flex-shrink-0 flex flex-col overflow-hidden">
          <CardHeader className="pb-2 flex-shrink-0">
            <CardTitle className="text-lg">Actions</CardTitle>
          </CardHeader>
          <CardContent className="flex-1 overflow-auto p-0">
            <ScrollArea className="h-full p-4">
              <div className="space-y-2">
              {/* Component actions */}
              <TooltipProvider>
                <div className="space-y-1">
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full justify-start"
                        disabled={!selectedComponent?.version_changeable}
                        onClick={() => selectedComponent && openChangeVersionDialog(selectedComponent)}
                      >
                        <Edit className="h-4 w-4 mr-2" />
                        Change Version
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Change version of selected component</TooltipContent>
                  </Tooltip>

                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full justify-start"
                        disabled={!canMoveUp()}
                        onClick={handleMoveUp}
                      >
                        <ChevronUp className="h-4 w-4 mr-2" />
                        Move Up
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Make component apply earlier</TooltipContent>
                  </Tooltip>

                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full justify-start"
                        disabled={!canMoveDown()}
                        onClick={handleMoveDown}
                      >
                        <ChevronDown className="h-4 w-4 mr-2" />
                        Move Down
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Make component apply later</TooltipContent>
                  </Tooltip>

                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full justify-start text-destructive hover:text-destructive"
                        disabled={!selectedComponent?.removable}
                        onClick={() => selectedComponent && setRemoveDialog(selectedComponent)}
                      >
                        <Trash2 className="h-4 w-4 mr-2" />
                        Remove
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Remove selected component</TooltipContent>
                  </Tooltip>
                </div>

                <Separator className="my-3" />

                <div className="space-y-1">
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full justify-start"
                        disabled={!selectedComponent?.customizable || selectedComponent?.custom}
                        onClick={handleCustomize}
                      >
                        <Wrench className="h-4 w-4 mr-2" />
                        Customize
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Customize selected component</TooltipContent>
                  </Tooltip>

                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full justify-start"
                        disabled={!selectedComponent?.custom}
                      >
                        <Edit className="h-4 w-4 mr-2" />
                        Edit
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Edit custom component</TooltipContent>
                  </Tooltip>

                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full justify-start"
                        disabled={!selectedComponent?.revertible}
                        onClick={handleRevert}
                      >
                        <Undo2 className="h-4 w-4 mr-2" />
                        Revert
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Revert to default</TooltipContent>
                  </Tooltip>
                </div>

                <Separator className="my-3" />

                <div className="space-y-1">
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full justify-start"
                        onClick={openInstallLoaderDialog}
                        disabled={hasModLoader}
                      >
                        <Download className="h-4 w-4 mr-2" />
                        Install Loader
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                      {hasModLoader ? "Mod loader already installed" : "Install a mod loader"}
                    </TooltipContent>
                  </Tooltip>

                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full justify-start"
                        onClick={() => setJarModDialog(true)}
                      >
                        <Plus className="h-4 w-4 mr-2" />
                        Add to Minecraft.jar
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Add a mod to the Minecraft jar</TooltipContent>
                  </Tooltip>

                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full justify-start"
                        onClick={() => setReplaceJarDialog(true)}
                      >
                        <FileBox className="h-4 w-4 mr-2" />
                        Replace Minecraft.jar
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Replace the Minecraft jar</TooltipContent>
                  </Tooltip>

                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full justify-start"
                        onClick={() => setAgentDialog(true)}
                      >
                        <Bot className="h-4 w-4 mr-2" />
                        Add Agents
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Add Java agents</TooltipContent>
                  </Tooltip>

                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full justify-start"
                        onClick={() => setAddEmptyDialog(true)}
                      >
                        <Plus className="h-4 w-4 mr-2" />
                        Add Empty
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Add empty custom component</TooltipContent>
                  </Tooltip>

                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full justify-start"
                        disabled={true}
                      >
                        <Import className="h-4 w-4 mr-2" />
                        Import Components
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Import components from file</TooltipContent>
                  </Tooltip>
                </div>

                <Separator className="my-3" />

                <div className="space-y-1">
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full justify-start"
                        onClick={openMinecraftFolder}
                      >
                        <FolderOpen className="h-4 w-4 mr-2" />
                        Open .minecraft
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Open the .minecraft folder</TooltipContent>
                  </Tooltip>

                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full justify-start"
                        onClick={openLibrariesFolder}
                      >
                        <Library className="h-4 w-4 mr-2" />
                        Open Libraries
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Open the libraries folder</TooltipContent>
                  </Tooltip>

                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full justify-start"
                        onClick={refreshComponents}
                        disabled={refreshing}
                      >
                        <RefreshCw className={cn("h-4 w-4 mr-2", refreshing && "animate-spin")} />
                        Reload
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Reload component list</TooltipContent>
                  </Tooltip>
                </div>
              </TooltipProvider>
              </div>
            </ScrollArea>
          </CardContent>
        </Card>
      </div>

      {/* Component info panel */}
      {selectedComponent && selectedComponent.problems.length > 0 && (
        <Card className="border-yellow-500/50 bg-yellow-500/5">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center gap-2">
              <AlertTriangle className="h-4 w-4 text-yellow-500" />
              {selectedComponent.name} has issues
            </CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="text-sm space-y-1">
              {selectedComponent.problems.map((problem, idx) => (
                <li key={idx} className="flex items-start gap-2">
                  {getProblemIcon(problem.severity)}
                  <span>{problem.description}</span>
                </li>
              ))}
            </ul>
          </CardContent>
        </Card>
      )}

      {/* Install Loader Dialog */}
      <Dialog open={installLoaderDialog} onOpenChange={setInstallLoaderDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Install Mod Loader</DialogTitle>
            <DialogDescription>
              Select a mod loader to install for this instance.
            </DialogDescription>
          </DialogHeader>
          
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">Loader Type</label>
              <Select
                value={selectedLoaderType}
                onValueChange={(value) => {
                  setSelectedLoaderType(value);
                  loadLoaderVersions(value);
                }}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="fabric">Fabric</SelectItem>
                  <SelectItem value="quilt">Quilt</SelectItem>
                  <SelectItem value="forge">Forge</SelectItem>
                  <SelectItem value="neoforge">NeoForge</SelectItem>
                </SelectContent>
              </Select>
            </div>
            
            <div className="space-y-2">
              <label className="text-sm font-medium">Version</label>
              {loadingLoaderVersions ? (
                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                  <RefreshCw className="h-4 w-4 animate-spin" />
                  Loading versions...
                </div>
              ) : loaderVersions.length === 0 ? (
                <div className="text-sm text-muted-foreground">
                  No versions available for this Minecraft version
                </div>
              ) : (
                <Select
                  value={selectedLoaderVersion}
                  onValueChange={setSelectedLoaderVersion}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {loaderVersions.map((v) => (
                      <SelectItem key={v.version} value={v.version}>
                        {v.version} {v.recommended && "(recommended)"}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              )}
            </div>
          </div>
          
          <DialogFooter>
            <Button variant="outline" onClick={() => setInstallLoaderDialog(false)}>
              Cancel
            </Button>
            <Button
              onClick={handleInstallLoader}
              disabled={!selectedLoaderVersion || installingLoader}
            >
              {installingLoader ? (
                <>
                  <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                  Installing...
                </>
              ) : (
                "Install"
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Change Version Dialog */}
      <Dialog open={!!changeVersionDialog} onOpenChange={(open) => !open && setChangeVersionDialog(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Change Version</DialogTitle>
            <DialogDescription>
              Select a new version for {changeVersionDialog?.name}
            </DialogDescription>
          </DialogHeader>
          
          <div className="py-4">
            {loadingVersions ? (
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <RefreshCw className="h-4 w-4 animate-spin" />
                Loading versions...
              </div>
            ) : availableVersions.length === 0 ? (
              <div className="text-sm text-muted-foreground">
                No versions available
              </div>
            ) : (
              <Select
                value={selectedNewVersion}
                onValueChange={setSelectedNewVersion}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent className="max-h-[300px]">
                  {availableVersions.map((v) => (
                    <SelectItem key={v} value={v}>
                      {v}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}
          </div>
          
          <DialogFooter>
            <Button variant="outline" onClick={() => setChangeVersionDialog(null)}>
              Cancel
            </Button>
            <Button
              onClick={handleChangeVersion}
              disabled={!selectedNewVersion || changingVersion}
            >
              {changingVersion ? (
                <>
                  <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                  Changing...
                </>
              ) : (
                "Change"
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Remove Component Dialog */}
      <AlertDialog open={!!removeDialog} onOpenChange={(open) => !open && setRemoveDialog(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Remove Component</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to remove "{removeDialog?.name}"?
              {removeDialog?.custom && " This is a custom component and will be permanently deleted."}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleRemoveComponent}>
              Remove
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Jar Mod Dialog */}
      <Dialog open={jarModDialog} onOpenChange={setJarModDialog}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle>Add to Minecraft.jar</DialogTitle>
            <DialogDescription>
              Add jar mods that will be injected into the Minecraft jar file at runtime.
              These are legacy mods that don't use a modern mod loader.
            </DialogDescription>
          </DialogHeader>
          
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>Current Jar Mods</Label>
              {jarMods.length === 0 ? (
                <div className="text-sm text-muted-foreground p-4 border rounded-lg text-center">
                  No jar mods installed
                </div>
              ) : (
                <ScrollArea className="h-[150px] border rounded-lg p-2">
                  <div className="space-y-1">
                    {jarMods.map((mod) => (
                      <div key={mod} className="flex items-center justify-between p-2 rounded hover:bg-accent">
                        <span className="text-sm font-mono truncate">{mod}</span>
                        <Button
                          variant="ghost"
                          size="sm"
                          className="text-destructive hover:text-destructive"
                          onClick={() => handleRemoveJarMod(mod)}
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                    ))}
                  </div>
                </ScrollArea>
              )}
            </div>
          </div>
          
          <DialogFooter>
            <Button variant="outline" onClick={() => setJarModDialog(false)}>
              Close
            </Button>
            <Button onClick={handleAddJarMod}>
              <Plus className="h-4 w-4 mr-2" />
              Add Jar Mod
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Agent Dialog */}
      <Dialog open={agentDialog} onOpenChange={setAgentDialog}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle>Java Agents</DialogTitle>
            <DialogDescription>
              Add Java agents that will be loaded at game startup via -javaagent flag.
              Useful for debugging, profiling, or mods like LiteLoader.
            </DialogDescription>
          </DialogHeader>
          
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>Current Agents</Label>
              {agents.length === 0 ? (
                <div className="text-sm text-muted-foreground p-4 border rounded-lg text-center">
                  No agents installed
                </div>
              ) : (
                <ScrollArea className="h-[150px] border rounded-lg p-2">
                  <div className="space-y-1">
                    {agents.map((agent) => (
                      <div key={agent.file} className="flex items-center justify-between p-2 rounded hover:bg-accent">
                        <div className="flex-1 truncate">
                          <span className="text-sm font-mono">{agent.file}</span>
                          {agent.args && (
                            <span className="text-xs text-muted-foreground ml-2">({agent.args})</span>
                          )}
                        </div>
                        <Button
                          variant="ghost"
                          size="sm"
                          className="text-destructive hover:text-destructive"
                          onClick={() => handleRemoveAgent(agent.file)}
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                    ))}
                  </div>
                </ScrollArea>
              )}
            </div>
            
            <Separator />
            
            <div className="space-y-2">
              <Label htmlFor="agentArgs">Agent Arguments (optional)</Label>
              <Input
                id="agentArgs"
                value={agentArgs}
                onChange={(e) => setAgentArgs(e.target.value)}
                placeholder="Arguments to pass to the agent"
              />
              <div className="text-xs text-muted-foreground">
                These arguments will be appended after the = sign in -javaagent:agent.jar=args
              </div>
            </div>
          </div>
          
          <DialogFooter>
            <Button variant="outline" onClick={() => setAgentDialog(false)}>
              Close
            </Button>
            <Button onClick={handleAddAgent}>
              <Plus className="h-4 w-4 mr-2" />
              Add Agent
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Replace Minecraft.jar Dialog */}
      <Dialog open={replaceJarDialog} onOpenChange={setReplaceJarDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Replace Minecraft.jar</DialogTitle>
            <DialogDescription>
              Replace the entire Minecraft jar with a custom one.
              This is useful for heavily modified jar files or modpacks.
            </DialogDescription>
          </DialogHeader>
          
          <div className="space-y-4 py-4">
            {hasCustomJar ? (
              <div className="space-y-4">
                <div className="flex items-center gap-2 p-4 border rounded-lg bg-yellow-500/10 border-yellow-500/20">
                  <AlertTriangle className="h-5 w-5 text-yellow-500" />
                  <div className="text-sm">
                    <p className="font-medium">Custom jar is currently active</p>
                    <p className="text-muted-foreground">
                      You can revert to the original Minecraft jar or replace it with a different one.
                    </p>
                  </div>
                </div>
                <Button
                  variant="outline"
                  className="w-full"
                  onClick={handleRevertMinecraftJar}
                >
                  <Undo2 className="h-4 w-4 mr-2" />
                  Revert to Original
                </Button>
              </div>
            ) : (
              <div className="text-sm text-muted-foreground p-4 border rounded-lg text-center">
                No custom jar is installed. The original Minecraft jar is being used.
              </div>
            )}
          </div>
          
          <DialogFooter>
            <Button variant="outline" onClick={() => setReplaceJarDialog(false)}>
              Close
            </Button>
            <Button onClick={handleReplaceMinecraftJar}>
              <FileBox className="h-4 w-4 mr-2" />
              Select Jar File
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Add Empty Component Dialog */}
      <Dialog open={addEmptyDialog} onOpenChange={setAddEmptyDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Add Empty Component</DialogTitle>
            <DialogDescription>
              Create a new empty custom component. You can customize its content later.
            </DialogDescription>
          </DialogHeader>
          
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="componentName">Component Name</Label>
              <Input
                id="componentName"
                value={newComponentName}
                onChange={(e) => setNewComponentName(e.target.value)}
                placeholder="Enter component name"
              />
            </div>
          </div>
          
          <DialogFooter>
            <Button variant="outline" onClick={() => {
              setAddEmptyDialog(false);
              setNewComponentName("");
            }}>
              Cancel
            </Button>
            <Button onClick={handleAddEmpty} disabled={!newComponentName.trim()}>
              <Plus className="h-4 w-4 mr-2" />
              Add Component
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}