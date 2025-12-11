import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { Save, X, Search } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
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

interface MinecraftVersion {
  id: string;
  version_type: string;
  release_time: string;
}

interface LoaderVersion {
  version: string;
  recommended: boolean;
}

export function CreateInstanceView() {
  const navigate = useNavigate();
  const [name, setName] = useState("");
  const [version, setVersion] = useState("");
  const [modLoader, setModLoader] = useState("Vanilla");
  const [loaderVersion, setLoaderVersion] = useState("");
  const [creating, setCreating] = useState(false);

  // Version list state
  const [versions, setVersions] = useState<MinecraftVersion[]>([]);
  const [filteredVersions, setFilteredVersions] = useState<MinecraftVersion[]>([]);
  const [loadingVersions, setLoadingVersions] = useState(true);
  const [versionSearch, setVersionSearch] = useState("");
  const [showReleases, setShowReleases] = useState(true);
  const [showSnapshots, setShowSnapshots] = useState(false);
  const [showOld, setShowOld] = useState(false);

  // Loader version state
  const [loaderVersions, setLoaderVersions] = useState<LoaderVersion[]>([]);
  const [loadingLoaderVersions, setLoadingLoaderVersions] = useState(false);

  // Load Minecraft versions on mount
  useEffect(() => {
    loadMinecraftVersions();
  }, [showReleases, showSnapshots, showOld]);

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
    if (version && modLoader !== "Vanilla") {
      loadLoaderVersions();
    } else {
      setLoaderVersions([]);
      setLoaderVersion("");
    }
  }, [version, modLoader]);

  const loadMinecraftVersions = async () => {
    setLoadingVersions(true);
    try {
      const data = await invoke<MinecraftVersion[]>("get_minecraft_versions", {
        showReleases,
        showSnapshots,
        showOld,
      });
      setVersions(data);
      setFilteredVersions(data);
      if (data.length > 0 && !version) {
        setVersion(data[0].id);
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

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setCreating(true);

    try {
      await invoke("create_instance", {
        request: {
          name,
          minecraft_version: version,
          mod_loader_type: modLoader,
          loader_version: modLoader === "Vanilla" ? null : loaderVersion || null,
        },
      });
      navigate("/");
    } catch (error) {
      console.error("Failed to create instance:", error);
    } finally {
      setCreating(false);
    }
  };

  return (
    <div className="max-w-2xl mx-auto">
      <h1 className="text-3xl font-bold mb-8">Create New Instance</h1>

      <Card>
        <CardHeader>
          <CardTitle>Instance Configuration</CardTitle>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-6">
            <div className="space-y-2">
              <Label htmlFor="name">Instance Name</Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="My Minecraft Instance"
                required
              />
            </div>

            <div className="space-y-4">
              <Label>Minecraft Version</Label>

              <div className="flex flex-wrap gap-4">
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
                    id="old"
                    checked={showOld}
                    onCheckedChange={(checked) => setShowOld(checked as boolean)}
                  />
                  <Label htmlFor="old" className="text-sm font-normal cursor-pointer">
                    Old Versions (Alpha/Beta)
                  </Label>
                </div>
              </div>

              <div className="relative">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  value={versionSearch}
                  onChange={(e) => setVersionSearch(e.target.value)}
                  placeholder="Search versions..."
                  className="pl-9"
                />
              </div>

              <ScrollArea className="h-48 rounded-md border">
                <div className="p-2">
                  {loadingVersions ? (
                    <p className="text-muted-foreground text-sm p-2">Loading versions...</p>
                  ) : filteredVersions.length === 0 ? (
                    <p className="text-muted-foreground text-sm p-2">No versions found</p>
                  ) : (
                    filteredVersions.map((v) => (
                      <button
                        key={v.id}
                        type="button"
                        className={`w-full text-left px-3 py-2 rounded-md text-sm transition-colors ${
                          version === v.id
                            ? "bg-primary text-primary-foreground"
                            : "hover:bg-accent"
                        }`}
                        onClick={() => setVersion(v.id)}
                      >
                        {v.id}{" "}
                        <span className="text-xs opacity-70">({v.version_type})</span>
                      </button>
                    ))
                  )}
                </div>
              </ScrollArea>
            </div>

            <div className="space-y-2">
              <Label htmlFor="modLoader">Mod Loader</Label>
              <Select value={modLoader} onValueChange={setModLoader}>
                <SelectTrigger>
                  <SelectValue placeholder="Select mod loader" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Vanilla">Vanilla (No Mods)</SelectItem>
                  <SelectItem value="Forge">Forge</SelectItem>
                  <SelectItem value="NeoForge">NeoForge (Modern Forge)</SelectItem>
                  <SelectItem value="Fabric">Fabric</SelectItem>
                  <SelectItem value="Quilt">Quilt</SelectItem>
                  <SelectItem value="LiteLoader">LiteLoader (Legacy)</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {modLoader !== "Vanilla" && (
              <div className="space-y-2">
                <Label htmlFor="loaderVersion">
                  {modLoader} Version
                  {loadingLoaderVersions && (
                    <span className="text-muted-foreground ml-2">(Loading...)</span>
                  )}
                </Label>
                <Select
                  value={loaderVersion}
                  onValueChange={setLoaderVersion}
                  disabled={loadingLoaderVersions || loaderVersions.length === 0}
                >
                  <SelectTrigger>
                    <SelectValue
                      placeholder={
                        loadingLoaderVersions
                          ? "Loading..."
                          : loaderVersions.length === 0
                          ? "No versions available"
                          : "Select version"
                      }
                    />
                  </SelectTrigger>
                  <SelectContent>
                    {loaderVersions.map((v) => (
                      <SelectItem key={v.version} value={v.version}>
                        {v.version} {v.recommended && "(Recommended)"}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            )}

            <div className="flex justify-end gap-3 pt-4">
              <Button type="button" variant="secondary" onClick={() => navigate("/")}>
                <X className="mr-2 h-4 w-4" /> Cancel
              </Button>
              <Button type="submit" disabled={creating}>
                <Save className="mr-2 h-4 w-4" />
                {creating ? "Creating..." : "Create Instance"}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
