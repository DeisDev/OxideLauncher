import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Settings, Download, Coffee, Terminal, Gamepad2 } from "lucide-react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";
import { SettingsContext } from "./context";
import { OxideLauncherSettings } from "./OxideLauncherSettings";
import { JavaSettings } from "./JavaSettings";
import { MinecraftSettings } from "./MinecraftSettings";
import { DownloadSettings } from "./DownloadSettings";
import { AdvancedSettings } from "./AdvancedSettings";
import { useConfig } from "@/hooks/useConfig";
import type { Config } from "./types";

type SettingsTab = "launcher" | "java" | "minecraft" | "downloads" | "advanced";

interface SettingsNavItem {
  id: SettingsTab;
  label: string;
  icon: React.ReactNode;
  description: string;
}

const NAV_ITEMS: SettingsNavItem[] = [
  {
    id: "launcher",
    label: "Oxide Launcher",
    icon: <Settings className="h-4 w-4" />,
    description: "General launcher settings",
  },
  {
    id: "java",
    label: "Java",
    icon: <Coffee className="h-4 w-4" />,
    description: "Java runtime & memory",
  },
  {
    id: "minecraft",
    label: "Minecraft",
    icon: <Gamepad2 className="h-4 w-4" />,
    description: "Game window & console",
  },
  {
    id: "downloads",
    label: "Downloads",
    icon: <Download className="h-4 w-4" />,
    description: "Network & downloads",
  },
  {
    id: "advanced",
    label: "Advanced",
    icon: <Terminal className="h-4 w-4" />,
    description: "Logging, API keys & commands",
  },
];

// Skeleton loading component
function LoadingSkeleton() {
  return (
    <div className="flex h-full">
      <div className="w-56 border-r p-4 space-y-2">
        {[1, 2, 3, 4, 5].map((i) => (
          <div key={i} className="skeleton h-14 rounded-lg" />
        ))}
      </div>
      <div className="flex-1 p-6">
        <div className="skeleton h-9 w-48 mb-6" />
        <div className="space-y-4">
          <div className="skeleton h-40" />
          <div className="skeleton h-40" />
        </div>
      </div>
    </div>
  );
}

export function SettingsLayout() {
  const [config, setConfig] = useState<Config | null>(null);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<SettingsTab>("launcher");
  const saveTimeoutRef = useRef<number | null>(null);
  const { setConfig: setGlobalConfig } = useConfig();

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

  // Auto-save with debounce and sync to global config
  const saveConfig = useCallback(async (newConfig: Config) => {
    if (saveTimeoutRef.current) {
      window.clearTimeout(saveTimeoutRef.current);
    }
    
    saveTimeoutRef.current = window.setTimeout(async () => {
      try {
        await invoke("update_config", { config: newConfig });
        // Sync to global config provider so Layout and other components update
        setGlobalConfig(newConfig as import("@/hooks/useConfig").Config);
      } catch (error) {
        console.error("Failed to save config:", error);
      }
    }, 300);
  }, [setGlobalConfig]);

  // Wrapper for setConfig that auto-saves
  const setConfigWithSave: React.Dispatch<React.SetStateAction<Config | null>> = useCallback((action) => {
    setConfig((prev) => {
      const updated = typeof action === 'function' ? action(prev) : action;
      if (updated) {
        saveConfig(updated);
      }
      return updated;
    });
  }, [saveConfig]);

  if (loading) {
    return <LoadingSkeleton />;
  }

  if (!config) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-destructive">Failed to load settings</p>
      </div>
    );
  }

  return (
    <SettingsContext.Provider value={{ config, setConfig: setConfigWithSave, saveConfig: async () => {}, loading }}>
      <div className="flex flex-col md:flex-row h-full">
        {/* Sidebar Navigation */}
        <div className="w-full md:w-56 lg:w-64 border-b md:border-b-0 md:border-r flex flex-col flex-shrink-0">
          <div className="p-4 pb-2">
            <h1 className="text-xl font-bold">Settings</h1>
          </div>
          <ScrollArea className="flex-1">
            <div className="p-3 pt-2 flex md:flex-col gap-2 overflow-x-auto md:overflow-x-visible">
              {NAV_ITEMS.map((item) => (
                <button
                  key={item.id}
                  onClick={() => setActiveTab(item.id)}
                  className={cn(
                    "flex items-start gap-2 md:gap-3 p-2 md:p-3 rounded-lg text-left transition-colors flex-shrink-0 md:flex-shrink md:w-full",
                    activeTab === item.id
                      ? "bg-primary text-primary-foreground"
                      : "hover:bg-muted"
                  )}
                >
                  <div className="mt-0.5 flex-shrink-0">{item.icon}</div>
                  <div className="min-w-0 flex-1 hidden md:block">
                    <p className="font-medium text-sm leading-tight">{item.label}</p>
                    <p
                      className={cn(
                        "text-xs mt-0.5",
                        activeTab === item.id
                          ? "text-primary-foreground/70"
                          : "text-muted-foreground"
                      )}
                    >
                      {item.description}
                    </p>
                  </div>
                  <span className="md:hidden text-xs font-medium">{item.label}</span>
                </button>
              ))}
            </div>
          </ScrollArea>
        </div>

        {/* Main Content */}
        <div className="flex-1 overflow-hidden flex flex-col">
          {/* Sticky Header with Title */}
          <div className="flex-shrink-0 p-6 pb-4 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
            <div className="max-w-4xl">
              <h2 className="text-2xl font-bold">
                {NAV_ITEMS.find((item) => item.id === activeTab)?.label}
              </h2>
              <p className="text-muted-foreground">
                {NAV_ITEMS.find((item) => item.id === activeTab)?.description}
              </p>
            </div>
          </div>

          <ScrollArea className="flex-1">
            <div className="p-6 pt-4 max-w-4xl">

              {activeTab === "launcher" && <OxideLauncherSettings />}
              {activeTab === "java" && <JavaSettings />}
              {activeTab === "minecraft" && <MinecraftSettings />}
              {activeTab === "downloads" && <DownloadSettings />}
              {activeTab === "advanced" && <AdvancedSettings />}
            </div>
          </ScrollArea>
        </div>
      </div>
    </SettingsContext.Provider>
  );
}
