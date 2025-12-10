import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { 
  faPlus, faPlay, faTrash, faInfoCircle, faCube, faPencil,
  faFolder, faCopy, faFileExport, faImage, faExclamationTriangle, faCog,
  faStop, faObjectGroup, faLink
} from "@fortawesome/free-solid-svg-icons";
import { ContextMenu, ContextMenuItem } from "../components/ContextMenu";
import "./InstancesView.css";

interface InstanceInfo {
  id: string;
  name: string;
  minecraft_version: string;
  mod_loader: string;
  icon: string | null;
  last_played: string | null;
  total_played_seconds: number;
  group?: string | null;
}

export function InstancesView() {
  const navigate = useNavigate();
  const [instances, setInstances] = useState<InstanceInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    instanceId: string;
  } | null>(null);
  const [showRenameDialog, setShowRenameDialog] = useState<string | null>(null);
  const [renameName, setRenameName] = useState("");
  const [showGroupDialog, setShowGroupDialog] = useState<string | null>(null);
  const [groupName, setGroupName] = useState("");

  useEffect(() => {
    loadInstances();
  }, []);

  const loadInstances = async () => {
    try {
      const data = await invoke<InstanceInfo[]>("get_instances");
      setInstances(data);
    } catch (error) {
      console.error("Failed to load instances:", error);
    } finally {
      setLoading(false);
    }
  };

  const launchInstance = async (id: string) => {
    try {
      await invoke("launch_instance", { instanceId: id });
    } catch (error) {
      console.error("Failed to launch instance:", error);
    }
  };

  const deleteInstance = async (id: string) => {
    if (confirm("Are you sure you want to delete this instance?")) {
      try {
        await invoke("delete_instance", { instanceId: id });
        loadInstances();
      } catch (error) {
        console.error("Failed to delete instance:", error);
      }
    }
  };

  const handleContextMenu = (e: React.MouseEvent, instanceId: string) => {
    e.preventDefault();
    setContextMenu({
      x: e.clientX,
      y: e.clientY,
      instanceId,
    });
  };

  const handleRename = async (instanceId: string) => {
    if (!renameName.trim()) return;
    
    try {
      await invoke("rename_instance", {
        instanceId,
        newName: renameName,
      });
      setShowRenameDialog(null);
      setRenameName("");
      loadInstances();
    } catch (error) {
      console.error("Failed to rename instance:", error);
      alert("Failed to rename instance");
    }
  };

  const handleChangeIcon = async (instanceId: string) => {
    // TODO: Implement icon picker
    alert("Icon picker not implemented yet. This would show a grid of default icons and allow custom uploads.");
  };

  const handleCopy = async (instanceId: string) => {
    try {
      await invoke("copy_instance", { instanceId });
      loadInstances();
    } catch (error) {
      console.error("Failed to copy instance:", error);
      alert("Failed to copy instance");
    }
  };

  const handleChangeGroup = async (instanceId: string) => {
    if (!groupName.trim()) return;
    
    try {
      await invoke("change_instance_group", {
        instanceId,
        group: groupName || null,
      });
      setShowGroupDialog(null);
      setGroupName("");
      loadInstances();
    } catch (error) {
      console.error("Failed to change group:", error);
      alert("Failed to change group");
    }
  };

  const handleOpenFolder = async (instanceId: string) => {
    try {
      await invoke("open_instance_folder", { instanceId });
    } catch (error) {
      console.error("Failed to open folder:", error);
      alert("Failed to open folder");
    }
  };

  const handleExport = async (instanceId: string) => {
    // TODO: Implement file picker for export location
    alert("Export functionality requires file picker integration. This would let you choose where to save the .zip");
  };

  const handleCreateShortcut = async (instanceId: string) => {
    try {
      await invoke("create_instance_shortcut", { instanceId });
    } catch (error) {
      console.error("Failed to create shortcut:", error);
      alert(String(error));
    }
  };

  const handleKill = async (instanceId: string) => {
    try {
      await invoke("kill_instance", { instanceId });
    } catch (error) {
      console.error("Failed to kill instance:", error);
      alert("Failed to kill instance");
    }
  };

  const getContextMenuItems = (instanceId: string): ContextMenuItem[] => {
    return [
      {
        icon: faPlay,
        label: "Launch",
        action: () => launchInstance(instanceId),
      },
      {
        icon: faStop,
        label: "Kill",
        action: () => handleKill(instanceId),
      },
      { divider: true } as ContextMenuItem,
      {
        icon: faPencil,
        label: "Rename",
        action: () => {
          const instance = instances.find(i => i.id === instanceId);
          setRenameName(instance?.name || "");
          setShowRenameDialog(instanceId);
        },
      },
      {
        icon: faImage,
        label: "Change Icon",
        action: () => handleChangeIcon(instanceId),
      },
      {
        icon: faCog,
        label: "Edit...",
        action: () => navigate(`/instance/${instanceId}`),
      },
      {
        icon: faObjectGroup,
        label: "Change Group...",
        action: () => {
          const instance = instances.find(i => i.id === instanceId);
          setGroupName(instance?.group || "");
          setShowGroupDialog(instanceId);
        },
      },
      { divider: true } as ContextMenuItem,
      {
        icon: faFolder,
        label: "Folder",
        action: () => handleOpenFolder(instanceId),
      },
      {
        icon: faFileExport,
        label: "Export",
        action: () => handleExport(instanceId),
      },
      {
        icon: faCopy,
        label: "Copy",
        action: () => handleCopy(instanceId),
      },
      {
        icon: faLink,
        label: "Create Shortcut",
        action: () => handleCreateShortcut(instanceId),
      },
      { divider: true } as ContextMenuItem,
      {
        icon: faTrash,
        label: "Delete",
        action: () => deleteInstance(instanceId),
        danger: true,
      },
    ];
  };

  if (loading) {
    return <div className="loading">Loading instances...</div>;
  }

  return (
    <div className="instances-view">
      <div className="header">
        <h1>Minecraft Instances</h1>
        <Link to="/create-instance">
          <button className="btn-success">
            <FontAwesomeIcon icon={faPlus} /> Create Instance
          </button>
        </Link>
      </div>

      {instances.length === 0 ? (
        <div className="empty-state">
          <p>No instances found. Create your first instance to get started!</p>
          <Link to="/create-instance">
            <button className="btn-success">
              <FontAwesomeIcon icon={faPlus} /> Create Instance
            </button>
          </Link>
        </div>
      ) : (
        <div className="instances-grid">
          {instances.map((instance) => (
            <div 
              key={instance.id} 
              className="instance-card card"
              onContextMenu={(e) => handleContextMenu(e, instance.id)}
            >
              <div className="instance-icon">
                {instance.icon ? (
                  <img src={instance.icon} alt={instance.name} />
                ) : (
                  <div className="default-icon">
                    <FontAwesomeIcon icon={faCube} size="3x" />
                  </div>
                )}
              </div>
              <div className="instance-info">
                <h3>{instance.name}</h3>
                <p className="version">Minecraft {instance.minecraft_version}</p>
                <p className="mod-loader">{instance.mod_loader}</p>
                {instance.last_played && (
                  <p className="last-played">
                    Last played: {new Date(instance.last_played).toLocaleDateString()}
                  </p>
                )}
              </div>
              <div className="instance-actions">
                <button onClick={() => launchInstance(instance.id)} className="btn-success">
                  <FontAwesomeIcon icon={faPlay} /> Launch
                </button>
                <Link to={`/instance/${instance.id}`}>
                  <button className="btn-secondary">
                    <FontAwesomeIcon icon={faInfoCircle} />
                  </button>
                </Link>
                <button onClick={() => deleteInstance(instance.id)} className="btn-danger">
                  <FontAwesomeIcon icon={faTrash} />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          items={getContextMenuItems(contextMenu.instanceId)}
          onClose={() => setContextMenu(null)}
        />
      )}

      {/* Rename Dialog */}
      {showRenameDialog && (
        <div className="dialog-overlay" onClick={() => setShowRenameDialog(null)}>
          <div className="dialog card" onClick={(e) => e.stopPropagation()}>
            <h2>Rename Instance</h2>
            <input
              type="text"
              value={renameName}
              onChange={(e) => setRenameName(e.target.value)}
              placeholder="Enter new name"
              autoFocus
              onKeyDown={(e) => {
                if (e.key === "Enter") handleRename(showRenameDialog);
                if (e.key === "Escape") setShowRenameDialog(null);
              }}
            />
            <div className="dialog-actions">
              <button onClick={() => setShowRenameDialog(null)} className="btn-secondary">
                Cancel
              </button>
              <button onClick={() => handleRename(showRenameDialog)} className="btn-success">
                Rename
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Group Dialog */}
      {showGroupDialog && (
        <div className="dialog-overlay" onClick={() => setShowGroupDialog(null)}>
          <div className="dialog card" onClick={(e) => e.stopPropagation()}>
            <h2>Change Group</h2>
            <input
              type="text"
              value={groupName}
              onChange={(e) => setGroupName(e.target.value)}
              placeholder="Enter group name (or leave empty)"
              autoFocus
              onKeyDown={(e) => {
                if (e.key === "Enter") handleChangeGroup(showGroupDialog);
                if (e.key === "Escape") setShowGroupDialog(null);
              }}
            />
            <div className="dialog-actions">
              <button onClick={() => setShowGroupDialog(null)} className="btn-secondary">
                Cancel
              </button>
              <button onClick={() => handleChangeGroup(showGroupDialog)} className="btn-success">
                Change Group
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
