// Main application layout with sidebar navigation
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

import { ReactNode, useState, useEffect } from "react";
import { Link, useLocation, useNavigate } from "react-router-dom";
import { 
  Gamepad2, User, Settings, Newspaper, Download, FolderOpen, 
  HelpCircle, ChevronDown, ChevronRight, Bug, MessageSquare, 
  Info, Globe, BookOpen, Folder
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { cn } from "@/lib/utils";
import { useConfig } from "@/hooks/useConfig";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { 
  setupMainWindowCloseHandler, 
  setupDialogEventListeners,
  focusCurrentDialog,
  setupMainWindowPositionMemory,
} from "@/lib/windowManager";

interface LayoutProps {
  children: ReactNode;
}

// About Dialog Component
function AboutDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void }) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="text-2xl">About Oxide Launcher</DialogTitle>
          <DialogDescription>
            A modern, open-source Minecraft launcher
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <div className="flex items-center gap-4">
            <div className="h-16 w-16 rounded-lg bg-gradient-to-br from-primary/20 to-primary/5 flex items-center justify-center">
              <Gamepad2 className="h-8 w-8 text-primary" />
            </div>
            <div>
              <h3 className="font-semibold text-lg">Oxide Launcher</h3>
              <p className="text-sm text-muted-foreground">Version 0.1.0</p>
            </div>
          </div>
          
          <div className="space-y-2 text-sm">
            <p>
              Built with Tauri, React, and Rust for a fast, native experience.
            </p>
            <p className="text-muted-foreground">
              Oxide Launcher is free and open source software, licensed under the GPL-3.0 license.
            </p>
          </div>
          
          <div className="pt-4 border-t space-y-2">
            <h4 className="font-medium text-sm">Credits</h4>
            <p className="text-sm text-muted-foreground">
              Developed by the Oxide Launcher team. Special thanks to all contributors and the open source community.
            </p>
          </div>
          
          <div className="pt-4 border-t">
            <p className="text-xs text-muted-foreground">
              © 2024-2025 Oxide Launcher Contributors
            </p>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

export function Layout({ children }: LayoutProps) {
  const location = useLocation();
  const navigate = useNavigate();
  const { config } = useConfig();
  const [foldersOpen, setFoldersOpen] = useState(false);
  const [helpOpen, setHelpOpen] = useState(false);
  const [aboutDialogOpen, setAboutDialogOpen] = useState(false);
  const [dialogOverlayVisible, setDialogOverlayVisible] = useState(false);

  const showNews = config?.ui.show_news ?? false;

  // Setup main window close handler and dialog event listeners
  useEffect(() => {
    // Setup handler to close dialogs when main window closes
    setupMainWindowCloseHandler();
    
    // Setup window position memory for main window
    let cleanupPositionMemory: (() => void) | null = null;
    setupMainWindowPositionMemory().then(cleanup => {
      cleanupPositionMemory = cleanup;
    });
    
    // Setup listeners for dialog open/close events
    const setupListeners = async () => {
      const cleanup = await setupDialogEventListeners(
        () => {
          setDialogOverlayVisible(true);
        },
        () => {
          setDialogOverlayVisible(false);
        }
      );
      return cleanup;
    };
    
    const cleanupPromise = setupListeners();
    
    return () => {
      cleanupPromise.then(cleanup => cleanup());
      cleanupPositionMemory?.();
    };
  }, []);

  // Listen for navigation events from dialog windows
  useEffect(() => {
    let unlistenInstances: UnlistenFn | undefined;
    let unlistenInstance: UnlistenFn | undefined;
    
    (async () => {
      unlistenInstances = await listen("navigate-to-instances", () => {
        // Navigate to instances view when modpack install completes
        navigate("/");
      });
      
      unlistenInstance = await listen<{ instanceId: string }>("navigate-to-instance", (event) => {
        // Navigate to specific instance details when modpack install completes
        if (event.payload.instanceId) {
          navigate(`/instance/${event.payload.instanceId}`);
        }
      });
    })();
    
    return () => {
      unlistenInstances?.();
      unlistenInstance?.();
    };
  }, [navigate]);

  // Main navigation items
  const navItems = [
    { path: "/", icon: Gamepad2, label: "Instances" },
    ...(showNews ? [{ path: "/news", icon: Newspaper, label: "News" }] : []),
    { path: "/accounts", icon: User, label: "Accounts" },
    { path: "/settings", icon: Settings, label: "Settings" },
    { path: "/update", icon: Download, label: "Updates" },
  ];

  // Folder shortcuts
  const folderItems = [
    { id: "instances", label: "Instances", action: () => openFolder("instances") },
    { id: "logs", label: "Logs", action: () => openFolder("logs") },
    { id: "java", label: "Java", action: () => openFolder("java") },
    { id: "assets", label: "Assets", action: () => openFolder("assets") },
    { id: "libraries", label: "Libraries", action: () => openFolder("libraries") },
    { id: "icons", label: "Icons", action: () => openFolder("icons") },
    { id: "skins", label: "Skins", action: () => openFolder("skins") },
  ];

  // Help menu items - About moved to bottom for consistency
  const helpItems = [
    { 
      id: "bug", 
      label: "Report a Bug", 
      icon: Bug,
      action: () => openExternalLink("https://github.com/OxideLauncher/OxideLauncher/issues/new?template=bug_report.md")
    },
    { 
      id: "feature", 
      label: "Suggest a Feature", 
      icon: MessageSquare,
      action: () => openExternalLink("https://github.com/OxideLauncher/OxideLauncher/issues/new?template=feature_request.md")
    },
    { id: "divider1", divider: true },
    { 
      id: "discord", 
      label: "Discord", 
      icon: MessageSquare,
      action: () => openExternalLink("https://discord.gg/oxide-launcher") // Placeholder
    },
    { 
      id: "reddit", 
      label: "Reddit", 
      icon: Globe,
      action: () => openExternalLink("https://reddit.com/r/OxideLauncher") // Placeholder
    },
    { 
      id: "website", 
      label: "Website", 
      icon: Globe,
      action: () => openExternalLink("https://oxidelauncher.org")
    },
    { 
      id: "docs", 
      label: "Documentation", 
      icon: BookOpen,
      action: () => openExternalLink("https://github.com/OxideLauncher/OxideLauncher/wiki")
    },
    { id: "divider2", divider: true },
    { 
      id: "about", 
      label: "About", 
      icon: Info,
      action: () => setAboutDialogOpen(true)
    },
  ];

  const openFolder = async (folderType: string) => {
    try {
      await invoke("open_launcher_folder", { folderType });
    } catch (error) {
      console.error(`Failed to open ${folderType} folder:`, error);
    }
  };

  const openExternalLink = async (url: string) => {
    try {
      await invoke("open_external_url", { url });
    } catch (error) {
      // Fallback to window.open if command doesn't exist
      window.open(url, "_blank");
    }
  };

  return (
    <div className="flex h-screen w-screen overflow-hidden">
      <nav className="w-56 bg-card border-r border-border flex flex-col flex-shrink-0">
        <div className="p-6 border-b border-border">
          <h2 className="text-xl font-bold bg-gradient-to-r from-primary to-primary/60 bg-clip-text text-transparent">
            OxideLauncher
          </h2>
        </div>
        
        {/* Main Navigation */}
        <ul className="flex-1 p-3 space-y-1 overflow-y-auto">
          {navItems.map((item) => (
            <li key={item.path}>
              <Link
                to={item.path}
                className={cn(
                  "flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-all",
                  location.pathname === item.path
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                )}
              >
                <item.icon className="h-5 w-5" />
                {item.label}
              </Link>
            </li>
          ))}

          {/* Folders Collapsible */}
          <li>
            <Collapsible open={foldersOpen} onOpenChange={setFoldersOpen}>
              <CollapsibleTrigger className="flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-all w-full text-muted-foreground hover:bg-accent hover:text-accent-foreground">
                <FolderOpen className="h-5 w-5" />
                <span className="flex-1 text-left">Folders</span>
                {foldersOpen ? (
                  <ChevronDown className="h-4 w-4" />
                ) : (
                  <ChevronRight className="h-4 w-4" />
                )}
              </CollapsibleTrigger>
              <CollapsibleContent className="pl-4 space-y-1 mt-1">
                {folderItems.map((item) => (
                  <button
                    key={item.id}
                    onClick={item.action}
                    className="flex items-center gap-3 px-4 py-2 rounded-lg text-sm transition-all w-full text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                  >
                    <Folder className="h-4 w-4" />
                    {item.label}
                  </button>
                ))}
              </CollapsibleContent>
            </Collapsible>
          </li>

          {/* Help Collapsible */}
          <li>
            <Collapsible open={helpOpen} onOpenChange={setHelpOpen}>
              <CollapsibleTrigger className="flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-all w-full text-muted-foreground hover:bg-accent hover:text-accent-foreground">
                <HelpCircle className="h-5 w-5" />
                <span className="flex-1 text-left">Help</span>
                {helpOpen ? (
                  <ChevronDown className="h-4 w-4" />
                ) : (
                  <ChevronRight className="h-4 w-4" />
                )}
              </CollapsibleTrigger>
              <CollapsibleContent className="pl-4 space-y-1 mt-1">
                {helpItems.map((item) => 
                  item.divider ? (
                    <div key={item.id} className="my-1 mx-4 border-t border-border" />
                  ) : (
                    <button
                      key={item.id}
                      onClick={item.action}
                      className="flex items-center gap-3 px-4 py-2 rounded-lg text-sm transition-all w-full text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                    >
                      {item.icon && <item.icon className="h-4 w-4" />}
                      {item.label}
                    </button>
                  )
                )}
              </CollapsibleContent>
            </Collapsible>
          </li>
        </ul>
      </nav>
      
      <main className="flex-1 overflow-auto">
        <div key={location.pathname} className="page-transition view-container h-full p-8">
          {children}
        </div>
      </main>

      {/* About Dialog */}
      <AboutDialog open={aboutDialogOpen} onOpenChange={setAboutDialogOpen} />
      
      {/* Dialog Window Overlay - blocks interaction with main window when dialog is open */}
      {dialogOverlayVisible && (
        <div 
          className="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center cursor-pointer"
          onClick={() => focusCurrentDialog()}
        >
          <div className="text-center space-y-2 pointer-events-none">
            <p className="text-muted-foreground text-sm">
              A dialog window is open
            </p>
            <p className="text-xs text-muted-foreground/70">
              Click anywhere to focus the dialog
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
