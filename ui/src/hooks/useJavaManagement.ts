// Custom hook for Java installation management.
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

import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { JavaInstallation, AvailableJavaVersion } from "@/types";

export interface UseJavaManagementReturn {
  // State
  detectedJava: JavaInstallation[];
  availableVersions: AvailableJavaVersion[];
  loadingDetection: boolean;
  loadingVersions: boolean;
  downloadingJava: boolean;
  downloadProgress: string;
  
  // Actions
  detectJavaInstallations: () => Promise<void>;
  fetchAvailableVersions: () => Promise<void>;
  downloadJava: (majorVersion: number) => Promise<void>;
  deleteJavaInstallation: (javaPath: string) => Promise<boolean>;
  validateJava: (javaPath: string) => Promise<JavaInstallation | null>;
}

export function useJavaManagement(): UseJavaManagementReturn {
  const [detectedJava, setDetectedJava] = useState<JavaInstallation[]>([]);
  const [availableVersions, setAvailableVersions] = useState<AvailableJavaVersion[]>([]);
  const [loadingDetection, setLoadingDetection] = useState(false);
  const [loadingVersions, setLoadingVersions] = useState(false);
  const [downloadingJava, setDownloadingJava] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState("");

  const detectJavaInstallations = useCallback(async () => {
    setLoadingDetection(true);
    try {
      const installations = await invoke<JavaInstallation[]>("detect_java");
      setDetectedJava(installations);
    } catch (error) {
      console.error("Failed to detect Java:", error);
      setDetectedJava([]);
    } finally {
      setLoadingDetection(false);
    }
  }, []);

  const fetchAvailableVersions = useCallback(async () => {
    setLoadingVersions(true);
    try {
      const versions = await invoke<AvailableJavaVersion[]>("fetch_available_java_versions");
      setAvailableVersions(versions);
    } catch (error) {
      console.error("Failed to fetch available Java versions:", error);
      setAvailableVersions([]);
    } finally {
      setLoadingVersions(false);
    }
  }, []);

  const downloadJava = useCallback(async (majorVersion: number) => {
    setDownloadingJava(true);
    setDownloadProgress("Starting download...");
    try {
      await invoke("download_java", { majorVersion });
      setDownloadProgress("Download complete!");
      // Refresh detected Java after download
      await detectJavaInstallations();
    } catch (error) {
      console.error("Failed to download Java:", error);
      setDownloadProgress(`Download failed: ${error}`);
      throw error;
    } finally {
      setDownloadingJava(false);
    }
  }, [detectJavaInstallations]);

  const deleteJavaInstallation = useCallback(async (javaPath: string): Promise<boolean> => {
    try {
      await invoke("delete_java", { javaPath });
      // Refresh detected Java after deletion
      await detectJavaInstallations();
      return true;
    } catch (error) {
      console.error("Failed to delete Java:", error);
      return false;
    }
  }, [detectJavaInstallations]);

  const validateJava = useCallback(async (javaPath: string): Promise<JavaInstallation | null> => {
    try {
      const installation = await invoke<JavaInstallation>("validate_java", { javaPath });
      return installation;
    } catch (error) {
      console.error("Failed to validate Java:", error);
      return null;
    }
  }, []);

  return {
    detectedJava,
    availableVersions,
    loadingDetection,
    loadingVersions,
    downloadingJava,
    downloadProgress,
    detectJavaInstallations,
    fetchAvailableVersions,
    downloadJava,
    deleteJavaInstallation,
    validateJava,
  };
}

export default useJavaManagement;
