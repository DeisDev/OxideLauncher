import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { Search, Upload, RefreshCw, Star } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";

// Source sidebar items matching Prism Launcher
type SourceType = "custom" | "import" | "atlauncher" | "curseforge" | "ftb-legacy" | "ftb-app" | "modrinth" | "technic";

const SOURCES: { id: SourceType; label: string }[] = [
  { id: "custom", label: "Custom" },
  { id: "import", label: "Import" },
  { id: "atlauncher", label: "ATLauncher" },
  { id: "curseforge", label: "CurseForge" },
  { id: "ftb-legacy", label: "FTB Legacy" },
  { id: "ftb-app", label: "FTB App Import" },
  { id: "modrinth", label: "Modrinth" },
  { id: "technic", label: "Technic" },
];

interface MinecraftVersion {
  id: string;
  version_type: string;
  release_time: string;
}

interface LoaderVersion {
  version: string;
  recommended: boolean;
}

interface ModpackResult {
  id: string;
  name: string;
  description: string;
  author: string;
  downloads: number;
  icon_url: string | null;
  versions?: string[];
}

export function CreateInstanceView() {
  const navigate = useNavigate();
  const [activeSource, setActiveSource] = useState<SourceType>("custom");
  
  // Instance creation state
  const [name, setName] = useState("");
  const [version, setVersion] = useState("");
  const [modLoader, setModLoader] = useState("None");
  const [loaderVersion, setLoaderVersion] = useState("");
  const [creating, setCreating] = useState(false);
  const [group, setGroup] = useState("Modpacks");

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

  // Modpack search state
  const [modpackSearch, setModpackSearch] = useState("");
  const [modpackResults, setModpackResults] = useState<ModpackResult[]>([]);
  const [searchingModpacks, setSearchingModpacks] = useState(false);
  const [selectedModpack, setSelectedModpack] = useState<ModpackResult | null>(null);
  const [selectedModpackVersion, setSelectedModpackVersion] = useState("");

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

  const searchModpacks = async () => {
    if (!modpackSearch.trim()) return;
    
    setSearchingModpacks(true);
    try {
      const results = await invoke<ModpackResult[]>("search_modpacks", {
        query: modpackSearch,
        source: activeSource,
      });
      setModpackResults(results);
    } catch (error) {
      console.error("Failed to search modpacks:", error);
      setModpackResults([]);
    } finally {
      setSearchingModpacks(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setCreating(true);

    try {
      if (selectedModpack) {
        await invoke("create_instance_from_modpack", {
          request: {
            name,
            modpack_id: selectedModpack.id,
            modpack_version: selectedModpackVersion,
            source: activeSource,
            group,
          },
        });
      } else {
        await invoke("create_instance", {
          request: {
            name,
            minecraft_version: version,
            mod_loader_type: modLoader === "None" ? "Vanilla" : modLoader,
            loader_version: modLoader === "None" ? null : loaderVersion || null,
          },
        });
      }
      navigate("/");
    } catch (error) {
      console.error("Failed to create instance:", error);
      alert("Failed to create instance: " + error);
    } finally {
      setCreating(false);
    }
  };

  const getVersionTypeLabel = (type: string) => {
    switch (type) {
      case "release": return "release";
      case "snapshot": return "snapshot";
      case "old_beta": return "beta";
      case "old_alpha": return "alpha";
      default: return type;
    }
  };

  // Parse Minecraft version to compare
  const parseMinecraftVersion = (versionStr: string): { major: number; minor: number; patch: number } | null => {
    // Handle standard versions like "1.20.2" or "1.14"
    const match = versionStr.match(/^(\d+)\.(\d+)(?:\.(\d+))?/);
    if (match) {
      return {
        major: parseInt(match[1], 10),
        minor: parseInt(match[2], 10),
        patch: match[3] ? parseInt(match[3], 10) : 0,
      };
    }
    return null;
  };

  // Compare versions: returns true if v1 >= v2
  const isVersionAtLeast = (versionStr: string, minMajor: number, minMinor: number, minPatch = 0): boolean => {
    const parsed = parseMinecraftVersion(versionStr);
    if (!parsed) return false;
    
    if (parsed.major > minMajor) return true;
    if (parsed.major < minMajor) return false;
    if (parsed.minor > minMinor) return true;
    if (parsed.minor < minMinor) return false;
    return parsed.patch >= minPatch;
  };

  // Check mod loader compatibility with selected Minecraft version
  const isLoaderCompatible = (loader: string): { compatible: boolean; reason?: string } => {
    if (!version || loader === "None") {
      return { compatible: true };
    }

    const parsed = parseMinecraftVersion(version);
    if (!parsed) {
      // For snapshots or unusual version strings, allow all loaders
      return { compatible: true };
    }

    switch (loader) {
      case "Fabric":
      case "Quilt":
        // Fabric and Quilt support 1.14+
        if (!isVersionAtLeast(version, 1, 14)) {
          return { compatible: false, reason: "Requires 1.14+" };
        }
        return { compatible: true };

      case "NeoForge":
        // NeoForge only supports 1.20.2+
        if (!isVersionAtLeast(version, 1, 20, 2)) {
          return { compatible: false, reason: "Requires 1.20.2+" };
        }
        return { compatible: true };

      case "Forge":
        // Forge has wide support, but minimal for very old versions
        return { compatible: true };

      case "LiteLoader":
        // LiteLoader mainly supports 1.5.2 - 1.12.2
        if (isVersionAtLeast(version, 1, 13)) {
          return { compatible: false, reason: "Only supports up to 1.12.2" };
        }
        if (!isVersionAtLeast(version, 1, 5, 2)) {
          return { compatible: false, reason: "Requires 1.5.2+" };
        }
        return { compatible: true };

      default:
        return { compatible: true };
    }
  };

  const renderCustomSource = () => (
    <div className="flex-1 flex flex-col gap-4 overflow-hidden">
      <div className="flex items-center gap-4 pb-4 border-b">
        <div className="flex-1">
          <Label htmlFor="name">Name</Label>
          <Input
            id="name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Instance name"
          />
        </div>
        <div className="w-48">
          <Label htmlFor="group">Group</Label>
          <Input
            id="group"
            value={group}
            onChange={(e) => setGroup(e.target.value)}
            placeholder="Modpacks"
          />
        </div>
      </div>

      <div className="flex flex-1 gap-6 min-h-0">
        <div className="flex-1 flex flex-col min-w-0 min-h-0">
          <div className="flex items-center justify-between mb-2">
            <Label className="text-base font-semibold">Custom</Label>
            <Button variant="ghost" size="sm" onClick={loadMinecraftVersions}>
              <RefreshCw className="h-4 w-4" />
            </Button>
          </div>

          <div className="grid grid-cols-[1fr,100px,80px] gap-2 px-3 py-2 text-xs text-muted-foreground border-b">
            <span>Version</span>
            <span>Released</span>
            <span>Type</span>
          </div>

          <ScrollArea className="h-[350px] border rounded-md">
            {loadingVersions ? (
              <div className="p-4 text-muted-foreground text-sm">Loading versions...</div>
            ) : filteredVersions.length === 0 ? (
              <div className="p-4 text-muted-foreground text-sm">No versions found</div>
            ) : (
              <div className="divide-y">
                {filteredVersions.map((v, index) => (
                  <button
                    key={v.id}
                    type="button"
                    className={cn(
                      "w-full grid grid-cols-[1fr,100px,80px] gap-2 px-3 py-2 text-sm text-left transition-colors",
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
                    <span className="flex items-center gap-2">
                      {index === 0 && <Star className="h-3 w-3 text-yellow-500 fill-yellow-500" />}
                      {v.id}
                    </span>
                    <span className="text-xs opacity-70">
                      {new Date(v.release_time).toLocaleDateString()}
                    </span>
                    <span className="text-xs opacity-70">
                      {getVersionTypeLabel(v.version_type)}
                    </span>
                  </button>
                ))}
              </div>
            )}
          </ScrollArea>

          <div className="mt-2">
            <Input
              value={versionSearch}
              onChange={(e) => setVersionSearch(e.target.value)}
              placeholder="Search"
            />
          </div>
        </div>

        <div className="w-64 flex flex-col gap-4">
          <Card>
            <CardHeader className="py-3">
              <CardTitle className="text-sm">Filter</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="flex items-center space-x-2">
                <Checkbox
                  id="releases"
                  checked={showReleases}
                  onCheckedChange={(checked) => setShowReleases(checked as boolean)}
                />
                <Label htmlFor="releases" className="text-sm font-normal cursor-pointer">
                  Releases
                </Label>
              </div>
              <div className="flex items-center space-x-2">
                <Checkbox
                  id="snapshots"
                  checked={showSnapshots}
                  onCheckedChange={(checked) => setShowSnapshots(checked as boolean)}
                />
                <Label htmlFor="snapshots" className="text-sm font-normal cursor-pointer">
                  Snapshots
                </Label>
              </div>
              <div className="flex items-center space-x-2">
                <Checkbox
                  id="betas"
                  checked={showBetas}
                  onCheckedChange={(checked) => setShowBetas(checked as boolean)}
                />
                <Label htmlFor="betas" className="text-sm font-normal cursor-pointer">
                  Betas
                </Label>
              </div>
              <div className="flex items-center space-x-2">
                <Checkbox
                  id="alphas"
                  checked={showAlphas}
                  onCheckedChange={(checked) => setShowAlphas(checked as boolean)}
                />
                <Label htmlFor="alphas" className="text-sm font-normal cursor-pointer">
                  Alphas
                </Label>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="py-3">
              <CardTitle className="text-sm">Mod Loader</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              {["None", "NeoForge", "Forge", "Fabric", "Quilt", "LiteLoader"].map((loader) => {
                const compat = isLoaderCompatible(loader);
                return (
                  <div key={loader} className="flex items-center space-x-2">
                    <input
                      type="radio"
                      id={loader}
                      name="modLoader"
                      value={loader}
                      checked={modLoader === loader}
                      onChange={(e) => setModLoader(e.target.value)}
                      disabled={!compat.compatible}
                      className="h-4 w-4 disabled:opacity-50"
                    />
                    <Label 
                      htmlFor={loader} 
                      className={cn(
                        "text-sm font-normal cursor-pointer",
                        !compat.compatible && "opacity-50 cursor-not-allowed"
                      )}
                      title={compat.reason}
                    >
                      {loader}
                      {compat.reason && (
                        <span className="ml-2 text-xs text-muted-foreground">
                          ({compat.reason})
                        </span>
                      )}
                    </Label>
                  </div>
                );
              })}
            </CardContent>
          </Card>

          {modLoader !== "None" && (
            <Card>
              <CardHeader className="py-3 flex flex-row items-center justify-between">
                <CardTitle className="text-sm">{modLoader} Version</CardTitle>
                <Button variant="ghost" size="sm" onClick={loadLoaderVersions} disabled={loadingLoaderVersions}>
                  <RefreshCw className={cn("h-4 w-4", loadingLoaderVersions && "animate-spin")} />
                </Button>
              </CardHeader>
              <CardContent>
                {loadingLoaderVersions ? (
                  <p className="text-sm text-muted-foreground">Loading...</p>
                ) : loaderVersions.length === 0 ? (
                  <p className="text-sm text-muted-foreground">No versions available for {version}</p>
                ) : (
                  <ScrollArea className="h-32">
                    <div className="space-y-1">
                      {loaderVersions.map((v) => (
                        <button
                          key={v.version}
                          type="button"
                          className={cn(
                            "w-full text-left px-2 py-1 text-sm rounded transition-colors",
                            loaderVersion === v.version
                              ? "bg-primary text-primary-foreground"
                              : "hover:bg-accent"
                          )}
                          onClick={() => setLoaderVersion(v.version)}
                        >
                          {v.version} {v.recommended && "(Recommended)"}
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

  const renderImportSource = () => (
    <div className="flex-1 flex flex-col items-center justify-center gap-4">
      <Upload className="h-16 w-16 text-muted-foreground" />
      <h2 className="text-xl font-semibold">Import Modpack</h2>
      <p className="text-muted-foreground text-center max-w-md">
        Drag and drop a modpack file here, or click to browse.
        Supports .zip, .mrpack (Modrinth), and CurseForge modpacks.
      </p>
      <Button variant="outline" onClick={() => alert("File picker not implemented yet")}>
        Browse Files
      </Button>
    </div>
  );

  const renderModpackSource = () => (
    <div className="flex-1 flex flex-col gap-4 overflow-hidden">
      <div className="flex items-center gap-4 pb-4 border-b">
        <div className="flex-1">
          <Label htmlFor="name">Name</Label>
          <Input
            id="name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Instance name"
          />
        </div>
        <div className="w-48">
          <Label htmlFor="group">Group</Label>
          <Input
            id="group"
            value={group}
            onChange={(e) => setGroup(e.target.value)}
            placeholder="Modpacks"
          />
        </div>
      </div>

      <div className="flex gap-2">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            value={modpackSearch}
            onChange={(e) => setModpackSearch(e.target.value)}
            placeholder={`Search ${SOURCES.find(s => s.id === activeSource)?.label} modpacks...`}
            className="pl-9"
            onKeyDown={(e) => e.key === "Enter" && searchModpacks()}
          />
        </div>
        <Button onClick={searchModpacks} disabled={searchingModpacks}>
          {searchingModpacks ? "Searching..." : "Search"}
        </Button>
      </div>

      <ScrollArea className="flex-1">
        {selectedModpack ? (
          <Card>
            <CardHeader>
              <div className="flex items-start gap-4">
                {selectedModpack.icon_url && (
                  <img src={selectedModpack.icon_url} alt="" className="w-16 h-16 rounded-md" />
                )}
                <div className="flex-1">
                  <CardTitle>{selectedModpack.name}</CardTitle>
                  <CardDescription>by {selectedModpack.author}</CardDescription>
                </div>
                <Button variant="outline" onClick={() => setSelectedModpack(null)}>
                  Back to Search
                </Button>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-sm">{selectedModpack.description}</p>
              {selectedModpack.versions && selectedModpack.versions.length > 0 && (
                <div className="space-y-2">
                  <Label>Version</Label>
                  <Select value={selectedModpackVersion} onValueChange={setSelectedModpackVersion}>
                    <SelectTrigger>
                      <SelectValue placeholder="Select version" />
                    </SelectTrigger>
                    <SelectContent>
                      {selectedModpack.versions.map((v) => (
                        <SelectItem key={v} value={v}>{v}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              )}
            </CardContent>
          </Card>
        ) : modpackResults.length > 0 ? (
          <div className="grid gap-4">
            {modpackResults.map((pack) => (
              <Card 
                key={pack.id} 
                className="cursor-pointer hover:border-primary transition-colors"
                onClick={() => {
                  setSelectedModpack(pack);
                  setName(pack.name);
                  if (pack.versions && pack.versions.length > 0) {
                    setSelectedModpackVersion(pack.versions[0]);
                  }
                }}
              >
                <CardContent className="flex items-start gap-4 p-4">
                  {pack.icon_url && (
                    <img src={pack.icon_url} alt="" className="w-12 h-12 rounded-md" />
                  )}
                  <div className="flex-1 min-w-0">
                    <h4 className="font-semibold">{pack.name}</h4>
                    <p className="text-sm text-muted-foreground">by {pack.author}</p>
                    <p className="text-sm mt-1 line-clamp-2">{pack.description}</p>
                    <p className="text-xs text-muted-foreground mt-2">
                      {pack.downloads.toLocaleString()} downloads
                    </p>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center h-64 text-muted-foreground">
            <Search className="h-12 w-12 mb-4" />
            <p>Search for modpacks to get started</p>
          </div>
        )}
      </ScrollArea>
    </div>
  );

  return (
    <form onSubmit={handleSubmit} className="flex flex-col h-full">
      <h1 className="text-2xl font-bold mb-4">New Instance</h1>

      <div className="flex flex-1 gap-4 overflow-hidden">
        <div className="w-40 flex-shrink-0 space-y-1">
          {SOURCES.map((source) => (
            <button
              key={source.id}
              type="button"
              className={cn(
                "w-full px-3 py-2 text-left text-sm rounded-md transition-colors flex items-center gap-2",
                activeSource === source.id
                  ? "bg-primary text-primary-foreground"
                  : "hover:bg-muted"
              )}
              onClick={() => {
                setActiveSource(source.id);
                setSelectedModpack(null);
                setModpackResults([]);
              }}
            >
              {source.label}
            </button>
          ))}
        </div>

        <div className="flex-1 flex flex-col overflow-hidden border rounded-lg p-4">
          {activeSource === "custom" && renderCustomSource()}
          {activeSource === "import" && renderImportSource()}
          {["atlauncher", "curseforge", "ftb-legacy", "ftb-app", "modrinth", "technic"].includes(activeSource) && renderModpackSource()}
        </div>
      </div>

      <div className="flex justify-end gap-3 pt-4 mt-4 border-t">
        <Button type="button" variant="outline" onClick={() => navigate("/")}>
          Cancel
        </Button>
        <Button type="button" variant="outline" onClick={() => alert("Help not implemented")}>
          Help
        </Button>
        <Button type="submit" disabled={creating || !name}>
          {creating ? "Creating..." : "OK"}
        </Button>
      </div>
    </form>
  );
}
