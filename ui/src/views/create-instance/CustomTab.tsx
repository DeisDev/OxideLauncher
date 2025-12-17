import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, Star } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";
import { MinecraftVersion, LoaderVersion } from "./types";
import { isLoaderCompatible, getVersionTypeLabel, getVersionTypeColor, getLoaderColor } from "./utils";

interface CustomTabProps {
  name: string;
  setName: (name: string) => void;
  group: string;
  setGroup: (group: string) => void;
  version: string;
  setVersion: (version: string) => void;
  modLoader: string;
  setModLoader: (loader: string) => void;
  loaderVersion: string;
  setLoaderVersion: (version: string) => void;
}

export function CustomTab({
  name,
  setName,
  group,
  setGroup,
  version,
  setVersion,
  modLoader,
  setModLoader,
  loaderVersion,
  setLoaderVersion,
}: CustomTabProps) {
  // Version list state
  const [versions, setVersions] = useState<MinecraftVersion[]>([]);
  const [filteredVersions, setFilteredVersions] = useState<MinecraftVersion[]>([]);
  const [loadingVersions, setLoadingVersions] = useState(true);
  const [versionSearch, setVersionSearch] = useState("");
  
  // Version filters
  const [showReleases, setShowReleases] = useState(true);
  const [showSnapshots, setShowSnapshots] = useState(false);
  const [showBetas, setShowBetas] = useState(false);
  const [showAlphas, setShowAlphas] = useState(false);

  // Loader version state
  const [loaderVersions, setLoaderVersions] = useState<LoaderVersion[]>([]);
  const [loadingLoaderVersions, setLoadingLoaderVersions] = useState(false);

  // Load Minecraft versions on mount and when filters change
  useEffect(() => {
    loadMinecraftVersions();
  }, [showReleases, showSnapshots, showBetas, showAlphas]);

  // Filter versions when search changes
  useEffect(() => {
    if (versionSearch) {
      setFilteredVersions(
        versions.filter((v) =>
          v.id.toLowerCase().includes(versionSearch.toLowerCase())
        )
      );
    } else {
      setFilteredVersions(versions);
    }
  }, [versionSearch, versions]);

  // Load loader versions when MC version or loader type changes
  useEffect(() => {
    if (version && modLoader !== "None") {
      loadLoaderVersions();
    } else {
      setLoaderVersions([]);
      setLoaderVersion("");
    }
  }, [version, modLoader]);

  // Auto-set name from version when version changes
  useEffect(() => {
    if (version && !name) {
      setName(version);
    }
  }, [version]);

  const loadMinecraftVersions = async () => {
    setLoadingVersions(true);
    try {
      const data = await invoke<MinecraftVersion[]>("get_minecraft_versions", {
        showReleases,
        showSnapshots,
        showOld: showBetas || showAlphas,
      });
      
      // Filter based on version type
      let filtered = data;
      if (!showReleases) {
        filtered = filtered.filter(v => v.version_type !== "release");
      }
      if (!showSnapshots) {
        filtered = filtered.filter(v => v.version_type !== "snapshot");
      }
      
      setVersions(filtered);
      setFilteredVersions(filtered);
      if (filtered.length > 0 && !version) {
        setVersion(filtered[0].id);
        setName(filtered[0].id);
      }
    } catch (error) {
      console.error("Failed to load versions:", error);
    } finally {
      setLoadingVersions(false);
    }
  };

  const loadLoaderVersions = async () => {
    if (!version) return;

    setLoadingLoaderVersions(true);
    setLoaderVersions([]);
    setLoaderVersion("");

    try {
      let data: LoaderVersion[] = [];

      switch (modLoader) {
        case "Forge":
          data = await invoke<LoaderVersion[]>("get_forge_versions", {
            minecraftVersion: version,
          });
          break;
        case "NeoForge":
          data = await invoke<LoaderVersion[]>("get_neoforge_versions", {
            minecraftVersion: version,
          });
          break;
        case "Fabric":
          data = await invoke<LoaderVersion[]>("get_fabric_versions", {
            minecraftVersion: version,
          });
          break;
        case "Quilt":
          data = await invoke<LoaderVersion[]>("get_quilt_versions", {
            minecraftVersion: version,
          });
          break;
        case "LiteLoader":
          data = await invoke<LoaderVersion[]>("get_liteloader_versions", {
            minecraftVersion: version,
          });
          break;
      }

      setLoaderVersions(data);

      // Auto-select recommended version
      const recommended = data.find((v) => v.recommended);
      if (recommended) {
        setLoaderVersion(recommended.version);
      } else if (data.length > 0) {
        setLoaderVersion(data[0].version);
      }
    } catch (error) {
      console.error("Failed to load loader versions:", error);
    } finally {
      setLoadingLoaderVersions(false);
    }
  };

  return (
    <div className="flex-1 flex flex-col gap-3 overflow-hidden">
      {/* Name and Group inputs */}
      <div className="flex flex-col sm:flex-row items-start sm:items-end gap-2 sm:gap-3 pb-3 border-b flex-shrink-0">
        <div className="flex-1 w-full sm:w-auto">
          <Label htmlFor="name" className="text-sm">Name</Label>
          <Input
            id="name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Instance name"
            className="h-9"
          />
        </div>
        <div className="w-full sm:w-40">
          <Label htmlFor="group" className="text-sm">Group</Label>
          <Input
            id="group"
            value={group}
            onChange={(e) => setGroup(e.target.value)}
            placeholder="Optional"
            className="h-9"
          />
        </div>
      </div>

      {/* Main content area */}
      <div className="flex flex-1 flex-col lg:flex-row gap-3 min-h-0 overflow-auto lg:overflow-hidden">
        {/* Version list section */}
        <div className="flex-1 flex flex-col min-w-0 min-h-0">
          <div className="flex items-center justify-between mb-2 flex-shrink-0">
            <Label className="text-sm font-semibold">Minecraft Version</Label>
            <Button variant="ghost" size="sm" className="h-7 w-7 p-0" onClick={loadMinecraftVersions}>
              <RefreshCw className="h-3.5 w-3.5" />
            </Button>
          </div>

          <ScrollArea className="flex-1 min-h-[150px] max-h-[200px] lg:max-h-none border rounded-md">
            {loadingVersions ? (
              <div className="p-3 text-muted-foreground text-sm">Loading versions...</div>
            ) : filteredVersions.length === 0 ? (
              <div className="p-3 text-muted-foreground text-sm">No versions found</div>
            ) : (
              <div className="divide-y">
                {filteredVersions.map((v, index) => (
                  <button
                    key={v.id}
                    type="button"
                    className={cn(
                      "w-full flex items-center justify-between gap-2 px-3 py-1.5 text-sm text-left transition-colors",
                      version === v.id
                        ? "bg-primary text-primary-foreground"
                        : "hover:bg-accent"
                    )}
                    onClick={() => {
                      setVersion(v.id);
                      if (!name || versions.some(ver => ver.id === name)) {
                        setName(v.id);
                      }
                    }}
                  >
                    <span className="flex items-center gap-1.5 truncate">
                      {index === 0 && <Star className="h-3 w-3 text-yellow-500 fill-yellow-500 flex-shrink-0" />}
                      {v.id}
                    </span>
                    <span className={cn(
                      "text-xs flex-shrink-0 hidden sm:inline",
                      version === v.id ? "opacity-70" : getVersionTypeColor(v.version_type)
                    )}>
                      {getVersionTypeLabel(v.version_type)}
                    </span>
                  </button>
                ))}
              </div>
            )}
          </ScrollArea>

          <div className="mt-2 flex-shrink-0">
            <Input
              value={versionSearch}
              onChange={(e) => setVersionSearch(e.target.value)}
              placeholder="Search versions..."
              className="h-8 text-sm"
            />
          </div>
        </div>

        {/* Sidebar with filters and mod loader */}
        <div className="w-full lg:w-56 flex flex-col gap-2 flex-shrink-0">
          {/* Filters - inline on mobile */}
          <Card className="flex-shrink-0">
            <CardHeader className="py-2 px-3">
              <CardTitle className="text-xs font-medium">Filters</CardTitle>
            </CardHeader>
            <CardContent className="px-3 pb-2 pt-0 flex flex-wrap gap-x-4 gap-y-1.5">
              {[
                { id: "releases", label: "Releases", checked: showReleases, onChange: setShowReleases },
                { id: "snapshots", label: "Snapshots", checked: showSnapshots, onChange: setShowSnapshots },
                { id: "betas", label: "Betas", checked: showBetas, onChange: setShowBetas },
                { id: "alphas", label: "Alphas", checked: showAlphas, onChange: setShowAlphas },
              ].map(({ id, label, checked, onChange }) => (
                <div key={id} className="flex items-center space-x-1.5">
                  <Checkbox
                    id={id}
                    checked={checked}
                    onCheckedChange={(val) => onChange(val as boolean)}
                    className="h-3.5 w-3.5"
                  />
                  <Label htmlFor={id} className="text-xs font-normal cursor-pointer">
                    {label}
                  </Label>
                </div>
              ))}
            </CardContent>
          </Card>

          {/* Mod Loader */}
          <Card className="flex-shrink-0">
            <CardHeader className="py-2 px-3">
              <CardTitle className="text-xs font-medium">Mod Loader</CardTitle>
            </CardHeader>
            <CardContent className="px-3 pb-2 pt-0 flex flex-wrap gap-x-3 gap-y-1">
              {["None", "NeoForge", "Forge", "Fabric", "Quilt", "LiteLoader"].map((loader) => {
                const compat = isLoaderCompatible(loader, version);
                const loaderColor = getLoaderColor(loader);
                return (
                  <div key={loader} className="flex items-center space-x-1.5">
                    <input
                      type="radio"
                      id={loader}
                      name="modLoader"
                      value={loader}
                      checked={modLoader === loader}
                      onChange={(e) => setModLoader(e.target.value)}
                      disabled={!compat.compatible}
                      className="h-3.5 w-3.5 disabled:opacity-50"
                    />
                    <Label 
                      htmlFor={loader} 
                      className={cn(
                        "text-xs font-normal cursor-pointer",
                        !compat.compatible && "opacity-50 cursor-not-allowed",
                        compat.compatible && modLoader === loader && loaderColor
                      )}
                      title={compat.reason}
                    >
                      {loader}
                    </Label>
                  </div>
                );
              })}
            </CardContent>
          </Card>

          {/* Loader Version */}
          {modLoader !== "None" && (
            <Card className="flex-1 min-h-0 flex flex-col">
              <CardHeader className="py-2 px-3 flex flex-row items-center justify-between">
                <CardTitle className={cn("text-xs font-medium", getLoaderColor(modLoader))}>{modLoader} Version</CardTitle>
                <Button variant="ghost" size="sm" className="h-6 w-6 p-0" onClick={loadLoaderVersions} disabled={loadingLoaderVersions}>
                  <RefreshCw className={cn("h-3 w-3", loadingLoaderVersions && "animate-spin")} />
                </Button>
              </CardHeader>
              <CardContent className="px-3 pb-2 pt-0 flex-1 min-h-0">
                {loadingLoaderVersions ? (
                  <p className="text-xs text-muted-foreground">Loading...</p>
                ) : loaderVersions.length === 0 ? (
                  <p className="text-xs text-muted-foreground">No versions for {version}</p>
                ) : (
                  <ScrollArea className="h-20 lg:h-24">
                    <div className="space-y-0.5">
                      {loaderVersions.map((v) => (
                        <button
                          key={v.version}
                          type="button"
                          className={cn(
                            "w-full text-left px-2 py-1 text-xs rounded transition-colors",
                            loaderVersion === v.version
                              ? "bg-primary text-primary-foreground"
                              : "hover:bg-accent"
                          )}
                          onClick={() => setLoaderVersion(v.version)}
                        >
                          {v.version} {v.recommended && "(Rec)"}
                        </button>
                      ))}
                    </div>
                  </ScrollArea>
                )}
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
}
