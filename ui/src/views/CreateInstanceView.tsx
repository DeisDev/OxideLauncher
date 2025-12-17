import { useState, useEffect } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { Package } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { 
  ImportTab, 
  CustomTab,
  SourceType, 
  SOURCES 
} from "./create-instance";
import { ModpackDownloadDialog } from "@/components/dialogs/ModpackDownloadDialog";

export function CreateInstanceView() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const initialSource = (searchParams.get("source") as SourceType) || "custom";
  const [activeSource, setActiveSource] = useState<SourceType>(initialSource);
  
  // Modpack dialog state
  const [modpackDialogOpen, setModpackDialogOpen] = useState(false);

  // Update active source when URL params change
  useEffect(() => {
    const source = searchParams.get("source") as SourceType;
    if (source && SOURCES.some(s => s.id === source)) {
      setActiveSource(source);
    }
  }, [searchParams]);
  
  // Instance creation state for custom tab
  const [name, setName] = useState("");
  const [version, setVersion] = useState("");
  const [modLoader, setModLoader] = useState("None");
  const [loaderVersion, setLoaderVersion] = useState("");
  const [creating, setCreating] = useState(false);
  const [group, setGroup] = useState("");

  const handleCreateInstance = async () => {
    setCreating(true);

    try {
      await invoke("create_instance", {
        request: {
          name,
          minecraft_version: version,
          mod_loader_type: modLoader === "None" ? "Vanilla" : modLoader,
          loader_version: modLoader === "None" ? null : loaderVersion || null,
          group: group || null,
        },
      });
      navigate("/");
    } catch (error) {
      console.error("Failed to create instance:", error);
      alert("Failed to create instance: " + error);
    } finally {
      setCreating(false);
    }
  };

  const handleSourceChange = (source: SourceType) => {
    if (source === "modpacks") {
      setModpackDialogOpen(true);
    } else {
      setActiveSource(source);
    }
  };

  // Determine if we can create an instance
  const canCreate = () => {
    if (activeSource !== "custom") return false;
    if (!name) return false;
    if (creating) return false;
    return true;
  };

  // Show bottom actions only for custom tab
  const showBottomActions = activeSource === "custom";

  return (
    <div className="flex flex-col h-full">
      <h1 className="text-xl md:text-2xl font-bold mb-3 md:mb-4">New Instance</h1>

      <div className="flex flex-1 flex-col md:flex-row gap-3 md:gap-4 overflow-hidden">
        {/* Source Sidebar - horizontal on mobile */}
        <div className="flex md:flex-col md:w-40 flex-shrink-0 gap-1 overflow-x-auto pb-1 md:pb-0">
          {SOURCES.map((source) => (
            <button
              key={source.id}
              type="button"
              className={cn(
                "px-3 py-2 text-sm rounded-md transition-colors flex items-center gap-2 whitespace-nowrap md:w-full md:text-left",
                activeSource === source.id && source.id !== "modpacks"
                  ? "bg-primary text-primary-foreground"
                  : "hover:bg-muted"
              )}
              onClick={() => handleSourceChange(source.id)}
            >
              {source.id === "modpacks" && <Package className="h-4 w-4" />}
              {source.label}
            </button>
          ))}
        </div>

        {/* Main Content Area */}
        <div className="flex-1 flex flex-col overflow-hidden border rounded-lg p-3 md:p-4 min-h-0">
          {activeSource === "custom" && (
            <CustomTab
              name={name}
              setName={setName}
              group={group}
              setGroup={setGroup}
              version={version}
              setVersion={setVersion}
              modLoader={modLoader}
              setModLoader={setModLoader}
              loaderVersion={loaderVersion}
              setLoaderVersion={setLoaderVersion}
            />
          )}
          
          {activeSource === "import" && (
            <ImportTab />
          )}
        </div>
      </div>

      {/* Bottom Actions - Only shown for custom tab */}
      {showBottomActions && (
        <div className="flex flex-wrap justify-end gap-2 md:gap-3 pt-3 md:pt-4 mt-3 md:mt-4 border-t">
          <Button type="button" variant="outline" size="sm" className="md:size-default" onClick={() => navigate("/")}>
            Cancel
          </Button>
          <Button 
            type="button" 
            size="sm"
            className="md:size-default"
            disabled={!canCreate()}
            onClick={handleCreateInstance}
          >
            {creating ? "Creating..." : "Create"}
          </Button>
        </div>
      )}

      {/* Modpack Download Dialog */}
      <ModpackDownloadDialog
        open={modpackDialogOpen}
        onOpenChange={setModpackDialogOpen}
        onModpackInstalled={() => navigate("/")}
      />
    </div>
  );
}
