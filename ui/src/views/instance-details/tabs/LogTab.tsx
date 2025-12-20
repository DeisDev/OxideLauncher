// Log output tab component for viewing Minecraft game logs
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

import { useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Copy, Upload, Trash2, Search, X, FolderOpen } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";
import type { LogEntry, LogLevel, LogSource } from "@/types";

interface LogTabProps {
  instanceId?: string;
  logContent: LogEntry[];
  setLogContent: (logs: LogEntry[]) => void;
  searchTerm: string;
  setSearchTerm: (term: string) => void;
  autoScroll: boolean;
  setAutoScroll: (auto: boolean) => void;
  wrapLines: boolean;
  setWrapLines: (wrap: boolean) => void;
}

/**
 * Get the CSS class for a log level
 */
function getLevelClass(level: LogLevel): string {
  switch (level) {
    case "fatal":
      return "text-red-500 font-semibold";
    case "error":
      return "text-red-400";
    case "warning":
      return "text-amber-400";
    case "info":
      return "text-slate-300";
    case "debug":
      return "text-slate-500";
    case "trace":
      return "text-slate-600";
    default:
      return "text-slate-300";
  }
}

/**
 * Get a short prefix for the log source
 */
function getSourcePrefix(source: LogSource): string {
  switch (source) {
    case "launcher":
      return "[LAUNCHER]";
    case "stderr":
      return "[STDERR]";
    case "game":
    default:
      return "";
  }
}

/**
 * Get the CSS class for a log source indicator
 */
function getSourceClass(source: LogSource): string {
  switch (source) {
    case "launcher":
      return "text-cyan-400";
    case "stderr":
      return "text-orange-400";
    case "game":
    default:
      return "";
  }
}

/**
 * Format a log entry for display
 */
function formatLogEntry(entry: LogEntry): string {
  const prefix = getSourcePrefix(entry.source);
  if (prefix) {
    return `${prefix} ${entry.content}`;
  }
  return entry.content;
}

export function LogTab({
  instanceId,
  logContent,
  setLogContent,
  searchTerm,
  setSearchTerm,
  autoScroll,
  setAutoScroll,
  wrapLines,
  setWrapLines,
}: LogTabProps) {
  const logEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (autoScroll && logEndRef.current) {
      logEndRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [logContent, autoScroll]);

  const copyLogs = () => {
    const text = logContent
      .map((entry) => formatLogEntry(entry))
      .join("\n");
    navigator.clipboard.writeText(text);
  };

  const uploadLogs = () => {
    alert("Upload to pastebin/logs service not implemented yet");
  };

  const clearLogs = () => {
    setLogContent([]);
  };

  const openLogsFolder = async () => {
    if (!instanceId) return;
    try {
      await invoke("open_instance_logs_folder", { instanceId });
    } catch (error) {
      console.error("Failed to open logs folder:", error);
    }
  };

  const filteredLogs = searchTerm
    ? logContent.filter((entry) =>
        entry.content.toLowerCase().includes(searchTerm.toLowerCase())
      )
    : logContent;

  return (
    <div className="flex flex-col h-full gap-4">
      <div className="flex items-center gap-2 flex-wrap">
        <Button variant="outline" size="sm" onClick={copyLogs}>
          <Copy className="mr-2 h-4 w-4" /> Copy
        </Button>
        <Button variant="outline" size="sm" onClick={uploadLogs}>
          <Upload className="mr-2 h-4 w-4" /> Upload
        </Button>
        <Button variant="outline" size="sm" onClick={openLogsFolder} disabled={!instanceId}>
          <FolderOpen className="mr-2 h-4 w-4" /> Open Folder
        </Button>
        <Button variant="destructive" size="sm" onClick={clearLogs}>
          <Trash2 className="mr-2 h-4 w-4" /> Clear
        </Button>

        <div className="flex-1" />

        <div className="flex items-center space-x-2">
          <Checkbox
            id="autoScroll"
            checked={autoScroll}
            onCheckedChange={(checked) => setAutoScroll(checked as boolean)}
          />
          <Label htmlFor="autoScroll" className="text-sm">Auto-scroll</Label>
        </div>

        <div className="flex items-center space-x-2">
          <Checkbox
            id="wrapLines"
            checked={wrapLines}
            onCheckedChange={(checked) => setWrapLines(checked as boolean)}
          />
          <Label htmlFor="wrapLines" className="text-sm">Wrap lines</Label>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search logs..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="pl-9"
          />
        </div>
        <Button variant="outline" size="sm" onClick={() => setSearchTerm("")}>
          <X className="h-4 w-4" />
        </Button>
      </div>

      <ScrollArea className="flex-1 rounded-md border bg-black/50">
        <div className={cn("p-4 font-mono text-xs", wrapLines ? "whitespace-pre-wrap" : "whitespace-pre")}>
          {filteredLogs.map((entry, index) => {
            const prefix = getSourcePrefix(entry.source);
            const sourceClass = getSourceClass(entry.source);
            const levelClass = getLevelClass(entry.level);
            
            return (
              <div key={index} className={cn("hover:bg-white/5", levelClass)}>
                {prefix && (
                  <span className={cn("mr-1", sourceClass)}>{prefix}</span>
                )}
                <span>{entry.content}</span>
              </div>
            );
          })}
          <div ref={logEndRef} />
        </div>
      </ScrollArea>
    </div>
  );
}
