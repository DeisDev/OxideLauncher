// Configuration context provider and hook for application settings.
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

import { createContext, useContext, useEffect, useState, ReactNode, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

// Config types matching Rust backend
export interface Config {
  data_dir: string;
  instances_dir: string | null;
  theme: string;
  java: JavaConfig;
  network: NetworkConfig;
  ui: UiConfig;
  minecraft: MinecraftConfig;
  commands: CustomCommands;
  memory: MemoryConfig;
  logging: LoggingConfig;
  api_keys: ApiKeys;
}

export interface JavaConfig {
  custom_path: string | null;
  use_bundled: boolean;
  auto_detect: boolean;
  extra_args: string[];
  skip_compatibility_check: boolean;
  auto_download: boolean;
}

export interface NetworkConfig {
  proxy: ProxyConfig | null;
  max_concurrent_downloads: number;
  download_retries: number;
  timeout_seconds: number;
  user_agent: string;
  downloads_dir: string | null;
  downloads_dir_watch_recursive: boolean;
}

export interface ProxyConfig {
  proxy_type: "Http" | "Socks5";
  host: string;
  port: number;
  username: string | null;
  password: string | null;
}

export interface UiConfig {
  show_news: boolean;
  instance_view: "Grid" | "List";
  instance_sort_by: string;
  instance_sort_asc: boolean;
  instance_grid_size: string;
  color_scheme: string;
  window_width: number;
  window_height: number;
  last_instance: string | null;
  rust_mode: boolean;
  open_instance_after_install: boolean;
}

export interface MinecraftConfig {
  window_width: number;
  window_height: number;
  launch_maximized: boolean;
  close_after_launch: boolean;
  show_console: boolean;
  auto_close_console: boolean;
  show_console_on_error: boolean;
  record_game_time: boolean;
  show_game_time: boolean;
}

export interface CustomCommands {
  pre_launch: string | null;
  post_exit: string | null;
  wrapper_command: string | null;
}

export interface MemoryConfig {
  min_memory: number;
  max_memory: number;
  permgen: number;
}

export interface LoggingConfig {
  debug_to_file: boolean;
  max_file_size_mb: number;
  max_files: number;
}

export interface ApiKeys {
  msa_client_id: string | null;
  curseforge_api_key: string | null;
  modrinth_api_token: string | null;
}

interface ConfigContextType {
  config: Config | null;
  loading: boolean;
  updateConfig: (updates: Partial<Config>) => void;
  updateUiConfig: (updates: Partial<UiConfig>) => void;
  updateMinecraftConfig: (updates: Partial<MinecraftConfig>) => void;
  setConfig: React.Dispatch<React.SetStateAction<Config | null>>;
  reloadConfig: () => Promise<void>;
}

const ConfigContext = createContext<ConfigContextType | null>(null);

export function useConfig() {
  const context = useContext(ConfigContext);
  if (!context) {
    throw new Error("useConfig must be used within a ConfigProvider");
  }
  return context;
}

interface ConfigProviderProps {
  children: ReactNode;
}

export function ConfigProvider({ children }: ConfigProviderProps) {
  const [config, setConfig] = useState<Config | null>(null);
  const [loading, setLoading] = useState(true);
  const saveTimeoutRef = useRef<number | null>(null);

  // Load config on mount
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

  // Debounced save function
  const saveConfig = useCallback(async (newConfig: Config) => {
    if (saveTimeoutRef.current) {
      window.clearTimeout(saveTimeoutRef.current);
    }
    
    saveTimeoutRef.current = window.setTimeout(async () => {
      try {
        await invoke("update_config", { config: newConfig });
      } catch (error) {
        console.error("Failed to save config:", error);
      }
    }, 300); // 300ms debounce
  }, []);

  const updateConfig = useCallback((updates: Partial<Config>) => {
    setConfig((prev) => {
      if (!prev) return prev;
      const newConfig = { ...prev, ...updates };
      saveConfig(newConfig);
      return newConfig;
    });
  }, [saveConfig]);

  const updateUiConfig = useCallback((updates: Partial<UiConfig>) => {
    setConfig((prev) => {
      if (!prev) return prev;
      const newConfig = { ...prev, ui: { ...prev.ui, ...updates } };
      saveConfig(newConfig);
      return newConfig;
    });
  }, [saveConfig]);

  const updateMinecraftConfig = useCallback((updates: Partial<MinecraftConfig>) => {
    setConfig((prev) => {
      if (!prev) return prev;
      const newConfig = { ...prev, minecraft: { ...prev.minecraft, ...updates } };
      saveConfig(newConfig);
      return newConfig;
    });
  }, [saveConfig]);

  const reloadConfig = useCallback(async () => {
    await loadConfig();
  }, []);

  return (
    <ConfigContext.Provider
      value={{
        config,
        loading,
        updateConfig,
        updateUiConfig,
        updateMinecraftConfig,
        setConfig,
        reloadConfig,
      }}
    >
      {children}
    </ConfigContext.Provider>
  );
}
