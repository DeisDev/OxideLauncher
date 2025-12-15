import { createContext, useContext } from "react";
import type { SettingsContextType } from "./types";

export const SettingsContext = createContext<SettingsContextType | null>(null);

export function useSettings() {
  const context = useContext(SettingsContext);
  if (!context) {
    throw new Error("useSettings must be used within a SettingsProvider");
  }
  return context;
}

// Helper to convert extra_args array to string for display
export function extraArgsToString(args: string[]): string {
  return args.join(' ');
}

// Helper to convert extra_args string to array for storage
export function stringToExtraArgs(str: string): string[] {
  if (!str.trim()) return [];
  return str.split(/\s+/).filter(arg => arg.length > 0);
}
