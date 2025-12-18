import { useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Copy, Upload, Trash2, Search, X, FolderOpen } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";

interface LogTabProps {
  instanceId?: string;
  logContent: string[];
  setLogContent: (logs: string[]) => void;
  searchTerm: string;
  setSearchTerm: (term: string) => void;
  autoScroll: boolean;
  setAutoScroll: (auto: boolean) => void;
  wrapLines: boolean;
  setWrapLines: (wrap: boolean) => void;
}

/**
 * Determines the log level/color class for a log line
 * Matches common Minecraft/Java log patterns
 */
function getLogLineClass(line: string): string {
  const lowerLine = line.toLowerCase();
  
  // Error patterns - red
  if (
    lowerLine.includes('/error]') ||
    lowerLine.includes('[error]') ||
    lowerLine.includes(' error:') ||
    lowerLine.includes('exception') ||
    lowerLine.includes('fatal') ||
    lowerLine.includes('failed') ||
    lowerLine.includes('crash') ||
    /\berror\b/.test(lowerLine)
  ) {
    return 'text-red-400';
  }
  
  // Warning patterns - amber/orange
  if (
    lowerLine.includes('/warn]') ||
    lowerLine.includes('[warn]') ||
    lowerLine.includes('[warning]') ||
    lowerLine.includes(' warn:') ||
    lowerLine.includes(' warning:') ||
    /\bwarn(ing)?\b/.test(lowerLine)
  ) {
    return 'text-amber-400';
  }
  
  // Info patterns - default (could make blue/cyan)
  if (
    lowerLine.includes('/info]') ||
    lowerLine.includes('[info]')
  ) {
    return 'text-slate-300';
  }
  
  // Debug patterns - dim gray
  if (
    lowerLine.includes('/debug]') ||
    lowerLine.includes('[debug]') ||
    lowerLine.includes('[trace]')
  ) {
    return 'text-slate-500';
  }
  
  // Default color
  return 'text-slate-300';
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
    navigator.clipboard.writeText(logContent.join("\n"));
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
    ? logContent.filter((line) =>
        line.toLowerCase().includes(searchTerm.toLowerCase())
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
          {filteredLogs.map((line, index) => (
            <div key={index} className={cn("hover:bg-white/5", getLogLineClass(line))}>
              {line}
            </div>
          ))}
          <div ref={logEndRef} />
        </div>
      </ScrollArea>
    </div>
  );
}
