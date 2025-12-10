import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./SettingsView.css";

interface Config {
  java: {
    custom_path: string | null;
    extra_args: string;
  };
  memory: {
    min_memory: number;
    max_memory: number;
  };
}

export function SettingsView() {
  const [config, setConfig] = useState<Config | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const data = await invoke<Config>("get_config");
      setConfig(data);
    } catch (error) {
      console.error("Failed to load config:", error);
    } finally {
      setLoading(false);
    }
  };

  const saveConfig = async () => {
    if (!config) return;
    try {
      await invoke("update_config", { config });
      alert("Settings saved successfully!");
    } catch (error) {
      console.error("Failed to save config:", error);
      alert("Failed to save settings");
    }
  };

  if (loading) {
    return <div className="loading">Loading settings...</div>;
  }

  if (!config) {
    return <div className="error">Failed to load settings</div>;
  }

  return (
    <div className="settings-view">
      <h1>Settings</h1>

      <div className="card">
        <h2>Java Settings</h2>
        <div className="form-group">
          <label htmlFor="javaPath">Custom Java Path (optional)</label>
          <input
            id="javaPath"
            type="text"
            value={config.java.custom_path || ""}
            onChange={(e) =>
              setConfig({
                ...config,
                java: { ...config.java, custom_path: e.target.value || null },
              })
            }
            placeholder="/path/to/java"
          />
        </div>
        <div className="form-group">
          <label htmlFor="javaArgs">Extra Java Arguments</label>
          <input
            id="javaArgs"
            type="text"
            value={config.java.extra_args}
            onChange={(e) =>
              setConfig({
                ...config,
                java: { ...config.java, extra_args: e.target.value },
              })
            }
            placeholder="-XX:+UseG1GC"
          />
        </div>
      </div>

      <div className="card">
        <h2>Memory Settings</h2>
        <div className="form-group">
          <label htmlFor="minMemory">Minimum Memory (MB)</label>
          <input
            id="minMemory"
            type="number"
            value={config.memory.min_memory}
            onChange={(e) =>
              setConfig({
                ...config,
                memory: { ...config.memory, min_memory: parseInt(e.target.value) },
              })
            }
            min="512"
            max="32768"
          />
        </div>
        <div className="form-group">
          <label htmlFor="maxMemory">Maximum Memory (MB)</label>
          <input
            id="maxMemory"
            type="number"
            value={config.memory.max_memory}
            onChange={(e) =>
              setConfig({
                ...config,
                memory: { ...config.memory, max_memory: parseInt(e.target.value) },
              })
            }
            min="1024"
            max="32768"
          />
        </div>
      </div>

      <div className="save-section">
        <button onClick={saveConfig} className="btn-success">
          Save Settings
        </button>
      </div>
    </div>
  );
}
