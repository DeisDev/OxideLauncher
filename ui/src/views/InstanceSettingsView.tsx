import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faSave, faTimes } from "@fortawesome/free-solid-svg-icons";
import "./InstanceSettingsView.css";

type TabType = "general" | "java" | "memory" | "game";

interface InstanceInfo {
  id: string;
  name: string;
  minecraft_version: string;
  mod_loader: string;
}

export function InstanceSettingsView() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = useState<TabType>("general");
  const [instance, setInstance] = useState<InstanceInfo | null>(null);
  const [loading, setLoading] = useState(true);

  // Settings state
  const [name, setName] = useState("");
  const [javaPath, setJavaPath] = useState("");
  const [javaArgs, setJavaArgs] = useState("");
  const [minMemory, setMinMemory] = useState("512");
  const [maxMemory, setMaxMemory] = useState("4096");
  const [width, setWidth] = useState("854");
  const [height, setHeight] = useState("480");

  useEffect(() => {
    loadInstance();
  }, [id]);

  const loadInstance = async () => {
    try {
      const data = await invoke<InstanceInfo>("get_instance_details", {
        instanceId: id,
      });
      setInstance(data);
      setName(data.name);
    } catch (error) {
      console.error("Failed to load instance:", error);
    } finally {
      setLoading(false);
    }
  };

  const saveSettings = async () => {
    // TODO: Implement save settings command
    alert("Save settings not fully implemented yet");
    navigate("/");
  };

  if (loading) {
    return <div className="loading">Loading settings...</div>;
  }

  if (!instance) {
    return <div className="error">Instance not found</div>;
  }

  return (
    <div className="instance-settings-view">
      <h1>Settings for {instance.name}</h1>

      <div className="settings-tabs">
        <button
          className={`settings-tab ${activeTab === "general" ? "active" : ""}`}
          onClick={() => setActiveTab("general")}
        >
          General
        </button>
        <button
          className={`settings-tab ${activeTab === "java" ? "active" : ""}`}
          onClick={() => setActiveTab("java")}
        >
          Java
        </button>
        <button
          className={`settings-tab ${activeTab === "memory" ? "active" : ""}`}
          onClick={() => setActiveTab("memory")}
        >
          Memory
        </button>
        <button
          className={`settings-tab ${activeTab === "game" ? "active" : ""}`}
          onClick={() => setActiveTab("game")}
        >
          Game Window
        </button>
      </div>

      <div className="settings-content card" style={{ padding: "32px" }}>
        {activeTab === "general" && (
          <div className="settings-section">
            <h2>General Settings</h2>
            <div className="form-group">
              <label htmlFor="name">Instance Name</label>
              <input
                id="name"
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
              />
            </div>
            <div className="form-group">
              <label>Minecraft Version</label>
              <input type="text" value={instance.minecraft_version} disabled />
            </div>
            <div className="form-group">
              <label>Mod Loader</label>
              <input type="text" value={instance.mod_loader} disabled />
            </div>
          </div>
        )}

        {activeTab === "java" && (
          <div className="settings-section">
            <h2>Java Settings</h2>
            <p>Configure Java runtime settings for this instance.</p>
            <div className="form-group">
              <label htmlFor="javaPath">Custom Java Path (optional)</label>
              <input
                id="javaPath"
                type="text"
                value={javaPath}
                onChange={(e) => setJavaPath(e.target.value)}
                placeholder="Leave empty to use default"
              />
            </div>
            <div className="form-group">
              <label htmlFor="javaArgs">Extra Java Arguments</label>
              <input
                id="javaArgs"
                type="text"
                value={javaArgs}
                onChange={(e) => setJavaArgs(e.target.value)}
                placeholder="-XX:+UseG1GC -Dsun.rmi.dgc.server.gcInterval=2147483646"
              />
            </div>
          </div>
        )}

        {activeTab === "memory" && (
          <div className="settings-section">
            <h2>Memory Settings</h2>
            <p>Configure how much RAM this instance can use.</p>
            <div className="form-row">
              <div className="form-group">
                <label htmlFor="minMemory">Minimum Memory (MB)</label>
                <input
                  id="minMemory"
                  type="number"
                  value={minMemory}
                  onChange={(e) => setMinMemory(e.target.value)}
                  min="512"
                  max="32768"
                />
              </div>
              <div className="form-group">
                <label htmlFor="maxMemory">Maximum Memory (MB)</label>
                <input
                  id="maxMemory"
                  type="number"
                  value={maxMemory}
                  onChange={(e) => setMaxMemory(e.target.value)}
                  min="1024"
                  max="32768"
                />
              </div>
            </div>
          </div>
        )}

        {activeTab === "game" && (
          <div className="settings-section">
            <h2>Game Window</h2>
            <p>Configure the Minecraft window size on launch.</p>
            <div className="form-row">
              <div className="form-group">
                <label htmlFor="width">Window Width</label>
                <input
                  id="width"
                  type="number"
                  value={width}
                  onChange={(e) => setWidth(e.target.value)}
                  min="640"
                />
              </div>
              <div className="form-group">
                <label htmlFor="height">Window Height</label>
                <input
                  id="height"
                  type="number"
                  value={height}
                  onChange={(e) => setHeight(e.target.value)}
                  min="480"
                />
              </div>
            </div>
          </div>
        )}
      </div>

      <div className="settings-actions">
        <button onClick={() => navigate("/")} className="btn-secondary">
          <FontAwesomeIcon icon={faTimes} /> Cancel
        </button>
        <button onClick={saveSettings} className="btn-success">
          <FontAwesomeIcon icon={faSave} /> Save Settings
        </button>
      </div>
    </div>
  );
}
