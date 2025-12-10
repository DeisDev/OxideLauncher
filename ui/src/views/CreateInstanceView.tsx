import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faSave, faTimes, faSearch } from "@fortawesome/free-solid-svg-icons";
import "./CreateInstanceView.css";

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
      alert("Failed to create instance");
    } finally {
      setCreating(false);
    }
  };

  return (
    <div className="create-instance-view">
      <h1>Create New Instance</h1>
      <form onSubmit={handleSubmit} className="card">
        <div className="form-group">
          <label htmlFor="name">Instance Name</label>
          <input
            id="name"
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="My Minecraft Instance"
            required
          />
        </div>

        <div className="form-group">
          <label>Minecraft Version</label>
          
          <div className="version-filters">
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={showReleases}
                onChange={(e) => setShowReleases(e.target.checked)}
              />
              <span>Releases</span>
            </label>
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={showSnapshots}
                onChange={(e) => setShowSnapshots(e.target.checked)}
              />
              <span>Snapshots</span>
            </label>
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={showOld}
                onChange={(e) => setShowOld(e.target.checked)}
              />
              <span>Old Versions (Alpha/Beta)</span>
            </label>
          </div>

          <div className="version-search">
            <FontAwesomeIcon icon={faSearch} className="search-icon" />
            <input
              type="text"
              value={versionSearch}
              onChange={(e) => setVersionSearch(e.target.value)}
              placeholder="Search versions..."
            />
          </div>

          <select
            id="version"
            value={version}
            onChange={(e) => setVersion(e.target.value)}
            size={10}
            required
          >
            {loadingVersions ? (
              <option disabled>Loading versions...</option>
            ) : filteredVersions.length === 0 ? (
              <option disabled>No versions found</option>
            ) : (
              filteredVersions.map((v) => (
                <option key={v.id} value={v.id}>
                  {v.id} ({v.version_type})
                </option>
              ))
            )}
          </select>
        </div>

        <div className="form-group">
          <label htmlFor="modLoader">Mod Loader</label>
          <select
            id="modLoader"
            value={modLoader}
            onChange={(e) => setModLoader(e.target.value)}
          >
            <option value="Vanilla">Vanilla (No Mods)</option>
            <option value="Forge">Forge</option>
            <option value="NeoForge">NeoForge (Modern Forge)</option>
            <option value="Fabric">Fabric</option>
            <option value="Quilt">Quilt</option>
            <option value="LiteLoader">LiteLoader (Legacy)</option>
          </select>
        </div>

        {modLoader !== "Vanilla" && (
          <div className="form-group">
            <label htmlFor="loaderVersion">
              {modLoader} Version
              {loadingLoaderVersions && " (Loading...)"}
            </label>
            <select
              id="loaderVersion"
              value={loaderVersion}
              onChange={(e) => setLoaderVersion(e.target.value)}
              disabled={loadingLoaderVersions || loaderVersions.length === 0}
              required
            >
              {loaderVersions.length === 0 ? (
                <option value="">
                  {loadingLoaderVersions ? "Loading..." : "No versions available"}
                </option>
              ) : (
                loaderVersions.map((v) => (
                  <option key={v.version} value={v.version}>
                    {v.version} {v.recommended ? "(Recommended)" : ""}
                  </option>
                ))
              )}
            </select>
          </div>
        )}

        <div className="form-actions">
          <button type="button" onClick={() => navigate("/")} className="btn-secondary">
            <FontAwesomeIcon icon={faTimes} /> Cancel
          </button>
          <button type="submit" disabled={creating} className="btn-success">
            <FontAwesomeIcon icon={faSave} /> {creating ? "Creating..." : "Create Instance"}
          </button>
        </div>
      </form>
    </div>
  );
}
