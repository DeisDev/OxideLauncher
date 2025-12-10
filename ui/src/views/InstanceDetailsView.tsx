import { useEffect, useState, useRef } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import {
  faArrowLeft,
  faCopy,
  faUpload,
  faTrash,
  faSearch,
  faCheckCircle,
  faTimesCircle,
  faDownload,
  faToggleOn,
  faToggleOff,
} from "@fortawesome/free-solid-svg-icons";
import "./InstanceDetailsView.css";

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
  const logContainerRef = useRef<HTMLDivElement>(null);

  // Mods state
  const [modSearchQuery, setModSearchQuery] = useState("");
  const [modSearchResults, setModSearchResults] = useState<ModSearchResult[]>([]);
  const [installedMods, setInstalledMods] = useState<InstalledMod[]>([]);
  const [searchingMods, setSearchingMods] = useState(false);
  const [downloadingMod, setDownloadingMod] = useState<string | null>(null);

  useEffect(() => {
    loadInstance();
    
    // Poll for logs every second
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
    if (!confirm(`Are you sure you want to delete ${filename}?`)) return;
    
    try {
      await invoke("delete_mod", {
        instanceId: id,
        filename: filename,
      });
      await loadInstalledMods();
    } catch (error) {
      console.error("Failed to delete mod:", error);
      alert("Failed to delete mod: " + error);
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
    return <div className="loading">Loading instance...</div>;
  }

  if (!instance) {
    return <div className="error">Instance not found</div>;
  }

  return (
    <div className="instance-details-view">
      <div className="details-header">
        <button onClick={() => navigate("/")} className="btn-secondary">
          <FontAwesomeIcon icon={faArrowLeft} /> Back
        </button>
        <h1>{instance.name}</h1>
        <div className="instance-info-badges">
          <span className="badge">{instance.minecraft_version}</span>
          <span className="badge">{instance.mod_loader}</span>
        </div>
      </div>

      <div className="details-body">
        <div className="details-sidebar">
          <button
            className={`sidebar-tab ${activeTab === "log" ? "active" : ""}`}
            onClick={() => setActiveTab("log")}
          >
            Minecraft Log
          </button>
          <button
            className={`sidebar-tab ${activeTab === "version" ? "active" : ""}`}
            onClick={() => setActiveTab("version")}
          >
            Version
          </button>
          <button
            className={`sidebar-tab ${activeTab === "mods" ? "active" : ""}`}
            onClick={() => setActiveTab("mods")}
          >
            Mods
          </button>
          <button
            className={`sidebar-tab ${
              activeTab === "resourcepacks" ? "active" : ""
            }`}
            onClick={() => setActiveTab("resourcepacks")}
          >
            Resource Packs
          </button>
          <button
            className={`sidebar-tab ${
              activeTab === "shaderpacks" ? "active" : ""
            }`}
            onClick={() => setActiveTab("shaderpacks")}
          >
            Shader Packs
          </button>
          <button
            className={`sidebar-tab ${activeTab === "notes" ? "active" : ""}`}
            onClick={() => setActiveTab("notes")}
          >
            Notes
          </button>
          <button
            className={`sidebar-tab ${activeTab === "worlds" ? "active" : ""}`}
            onClick={() => setActiveTab("worlds")}
          >
            Worlds
          </button>
          <button
            className={`sidebar-tab ${
              activeTab === "screenshots" ? "active" : ""
            }`}
            onClick={() => setActiveTab("screenshots")}
          >
            Screenshots
          </button>
          <button
            className={`sidebar-tab ${activeTab === "settings" ? "active" : ""}`}
            onClick={() => setActiveTab("settings")}
          >
            Settings
          </button>
        </div>

        <div className="details-content">
          {activeTab === "log" && (
            <div className="log-viewer">
              <div className="log-toolbar">
                <button onClick={copyLogs} className="btn-secondary btn-sm">
                  <FontAwesomeIcon icon={faCopy} /> Copy
                </button>
                <button onClick={uploadLogs} className="btn-secondary btn-sm">
                  <FontAwesomeIcon icon={faUpload} /> Upload
                </button>
                <button onClick={clearLogs} className="btn-danger btn-sm">
                  <FontAwesomeIcon icon={faTrash} /> Clear
                </button>

                <div className="spacer"></div>

                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={autoScroll}
                    onChange={(e) => setAutoScroll(e.target.checked)}
                  />
                  <span>Auto-scroll</span>
                </label>

                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={wrapLines}
                    onChange={(e) => setWrapLines(e.target.checked)}
                  />
                  <span>Wrap lines</span>
                </label>
              </div>

              <div className="log-search">
                <FontAwesomeIcon icon={faSearch} className="search-icon" />
                <input
                  type="text"
                  placeholder="Search logs..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                />
                <button className="btn-secondary btn-sm">Find</button>
                <button className="btn-secondary btn-sm">
                  <FontAwesomeIcon icon={faTimesCircle} />
                </button>
              </div>

              <div
                ref={logContainerRef}
                className={`log-content ${wrapLines ? "wrap" : ""}`}
              >
                {filteredLogs.map((line, index) => (
                  <div key={index} className="log-line">
                    {line}
                  </div>
                ))}
                <div ref={logEndRef} />
              </div>
            </div>
          )}

          {activeTab === "version" && (
            <div className="tab-content">
              <h2>Version Components</h2>
              <p>This tab would show the components/libraries for this instance, similar to the image you provided.</p>
            </div>
          )}

          {activeTab === "mods" && (
            <div className="mods-manager">
              <div className="mods-header">
                <h2>Mods</h2>
                <div className="mod-search-bar">
                  <input
                    type="text"
                    placeholder="Search mods on Modrinth..."
                    value={modSearchQuery}
                    onChange={(e) => setModSearchQuery(e.target.value)}
                    onKeyPress={(e) => e.key === "Enter" && searchMods()}
                  />
                  <button 
                    onClick={searchMods} 
                    className="btn-success"
                    disabled={searchingMods}
                  >
                    <FontAwesomeIcon icon={faSearch} /> 
                    {searchingMods ? "Searching..." : "Search"}
                  </button>
                </div>
              </div>

              {modSearchResults.length > 0 && (
                <div className="mod-search-results">
                  <h3>Search Results</h3>
                  <div className="mod-grid">
                    {modSearchResults.map((mod) => (
                      <div key={mod.id} className="mod-card">
                        {mod.icon_url && (
                          <img 
                            src={mod.icon_url} 
                            alt={mod.name} 
                            className="mod-icon"
                          />
                        )}
                        <div className="mod-info">
                          <h4>{mod.name}</h4>
                          <p className="mod-author">by {mod.author}</p>
                          <p className="mod-description">{mod.description}</p>
                          <div className="mod-stats">
                            <span>{mod.downloads.toLocaleString()} downloads</span>
                          </div>
                        </div>
                        <button
                          onClick={() => downloadMod(mod.id)}
                          className="btn-success btn-sm"
                          disabled={downloadingMod === mod.id}
                        >
                          <FontAwesomeIcon icon={faDownload} />
                          {downloadingMod === mod.id ? "Installing..." : "Install"}
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              <div className="installed-mods-section">
                <h3>Installed Mods ({installedMods.length})</h3>
                {installedMods.length === 0 ? (
                  <p className="empty-message">No mods installed yet. Search and install mods above.</p>
                ) : (
                  <div className="installed-mods-list">
                    {installedMods.map((mod) => (
                      <div key={mod.filename} className="installed-mod-item">
                        <div className="mod-name-status">
                          <button
                            onClick={() => toggleMod(mod.filename)}
                            className={`btn-toggle ${mod.enabled ? "enabled" : "disabled"}`}
                            title={mod.enabled ? "Click to disable" : "Click to enable"}
                          >
                            <FontAwesomeIcon 
                              icon={mod.enabled ? faToggleOn : faToggleOff} 
                            />
                          </button>
                          <span className={mod.enabled ? "enabled-text" : "disabled-text"}>
                            {mod.filename}
                          </span>
                        </div>
                        <button
                          onClick={() => deleteMod(mod.filename)}
                          className="btn-danger btn-sm"
                        >
                          <FontAwesomeIcon icon={faTrash} /> Delete
                        </button>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          )}

          {activeTab === "resourcepacks" && (
            <div className="tab-content">
              <h2>Resource Packs</h2>
              <p>Resource pack management coming soon...</p>
            </div>
          )}

          {activeTab === "shaderpacks" && (
            <div className="tab-content">
              <h2>Shader Packs</h2>
              <p>Shader pack management coming soon...</p>
            </div>
          )}

          {activeTab === "notes" && (
            <div className="tab-content">
              <h2>Notes</h2>
              <textarea
                placeholder="Add notes about this instance..."
                rows={10}
                style={{ width: "100%", padding: "12px" }}
              />
            </div>
          )}

          {activeTab === "worlds" && (
            <div className="tab-content">
              <h2>Worlds</h2>
              <p>World management coming soon...</p>
            </div>
          )}

          {activeTab === "screenshots" && (
            <div className="tab-content">
              <h2>Screenshots</h2>
              <p>Screenshot viewer coming soon...</p>
            </div>
          )}

          {activeTab === "settings" && (
            <div className="tab-content">
              <h2>Settings</h2>
              <button
                onClick={() => navigate(`/instance/${id}/settings`)}
                className="btn-success"
              >
                <FontAwesomeIcon icon={faCheckCircle} /> Open Settings Editor
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
