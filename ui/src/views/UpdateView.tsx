// View component for checking and applying launcher updates
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

import { useState } from "react";
import { Download, RefreshCw, CheckCircle2, AlertCircle, Info, ExternalLink } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";

interface UpdateInfo {
  currentVersion: string;
  latestVersion: string | null;
  updateAvailable: boolean;
  releaseNotes: string | null;
  downloadUrl: string | null;
}

export function UpdateView() {
  const [checking, setChecking] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo>({
    currentVersion: "0.1.0", // TODO: Get from app
    latestVersion: null,
    updateAvailable: false,
    releaseNotes: null,
    downloadUrl: null,
  });
  const [lastChecked, setLastChecked] = useState<Date | null>(null);
  const [error, setError] = useState<string | null>(null);

  const checkForUpdates = async () => {
    setChecking(true);
    setError(null);
    
    try {
      // TODO: Implement actual update check via Tauri command
      // For now, simulate a check
      await new Promise(resolve => setTimeout(resolve, 1500));
      
      setUpdateInfo({
        ...updateInfo,
        latestVersion: "0.1.0",
        updateAvailable: false,
        releaseNotes: null,
      });
      setLastChecked(new Date());
    } catch (err) {
      setError("Failed to check for updates. Please try again later.");
      console.error("Update check failed:", err);
    } finally {
      setChecking(false);
    }
  };

  const downloadUpdate = async () => {
    if (!updateInfo.downloadUrl) return;
    
    setDownloading(true);
    setDownloadProgress(0);
    
    try {
      // TODO: Implement actual download via Tauri
      // Simulate download progress
      for (let i = 0; i <= 100; i += 10) {
        await new Promise(resolve => setTimeout(resolve, 200));
        setDownloadProgress(i);
      }
      
      // TODO: Install update
    } catch (err) {
      setError("Failed to download update. Please try again later.");
      console.error("Update download failed:", err);
    } finally {
      setDownloading(false);
    }
  };

  return (
    <div className="w-full max-w-2xl mx-auto">
      <div className="mb-8 pb-5 border-b border-border">
        <h1 className="text-3xl font-bold bg-gradient-to-r from-foreground to-muted-foreground bg-clip-text text-transparent">
          Launcher Updates
        </h1>
        <p className="text-muted-foreground mt-2">
          Keep Oxide Launcher up to date with the latest features and improvements.
        </p>
      </div>

      <div className="space-y-6">
        {/* Current Version Card */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Info className="h-5 w-5" />
              Current Version
            </CardTitle>
            <CardDescription>
              Information about your installed version
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-center justify-between">
              <div>
                <p className="text-2xl font-bold">v{updateInfo.currentVersion}</p>
                <p className="text-sm text-muted-foreground">
                  Oxide Launcher
                </p>
              </div>
              <Button
                onClick={checkForUpdates}
                disabled={checking}
                variant="outline"
              >
                <RefreshCw className={`mr-2 h-4 w-4 ${checking ? "animate-spin" : ""}`} />
                {checking ? "Checking..." : "Check for Updates"}
              </Button>
            </div>
            {lastChecked && (
              <p className="text-xs text-muted-foreground mt-4">
                Last checked: {lastChecked.toLocaleString()}
              </p>
            )}
          </CardContent>
        </Card>

        {/* Update Status Card */}
        {(updateInfo.latestVersion || error) && (
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                {error ? (
                  <AlertCircle className="h-5 w-5 text-destructive" />
                ) : updateInfo.updateAvailable ? (
                  <Download className="h-5 w-5 text-primary" />
                ) : (
                  <CheckCircle2 className="h-5 w-5 text-green-500" />
                )}
                {error ? "Error" : updateInfo.updateAvailable ? "Update Available" : "Up to Date"}
              </CardTitle>
            </CardHeader>
            <CardContent>
              {error ? (
                <p className="text-destructive">{error}</p>
              ) : updateInfo.updateAvailable ? (
                <div className="space-y-4">
                  <div>
                    <p className="font-medium">
                      Version {updateInfo.latestVersion} is available!
                    </p>
                    <p className="text-sm text-muted-foreground">
                      You're currently on version {updateInfo.currentVersion}
                    </p>
                  </div>
                  
                  {updateInfo.releaseNotes && (
                    <div className="bg-muted p-4 rounded-lg">
                      <p className="text-sm font-medium mb-2">What's New:</p>
                      <p className="text-sm text-muted-foreground whitespace-pre-line">
                        {updateInfo.releaseNotes}
                      </p>
                    </div>
                  )}
                  
                  {downloading ? (
                    <div className="space-y-2">
                      <Progress value={downloadProgress} />
                      <p className="text-sm text-muted-foreground text-center">
                        Downloading... {downloadProgress}%
                      </p>
                    </div>
                  ) : (
                    <Button onClick={downloadUpdate} className="w-full">
                      <Download className="mr-2 h-4 w-4" />
                      Download Update
                    </Button>
                  )}
                </div>
              ) : (
                <p className="text-muted-foreground">
                  You're running the latest version of Oxide Launcher.
                </p>
              )}
            </CardContent>
          </Card>
        )}

        {/* Update Settings Card */}
        <Card>
          <CardHeader>
            <CardTitle>Update Settings</CardTitle>
            <CardDescription>
              Configure how updates are handled
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">Automatic Updates</p>
                <p className="text-sm text-muted-foreground">
                  Coming soon - Automatically check for and install updates
                </p>
              </div>
              <Button variant="outline" disabled>
                Coming Soon
              </Button>
            </div>
            
            <div className="pt-4 border-t">
              <Button variant="link" className="h-auto p-0" asChild>
                <a 
                  href="https://github.com/OxideLauncher/OxideLauncher/releases" 
                  target="_blank" 
                  rel="noopener noreferrer"
                >
                  <ExternalLink className="mr-2 h-4 w-4" />
                  View all releases on GitHub
                </a>
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
