import { createContext, useContext, useEffect, useState, ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ThemeContextType {
  theme: string;
  colorScheme: string;
  rustMode: boolean;
  setTheme: (theme: string) => void;
  setColorScheme: (scheme: string) => void;
  setRustMode: (enabled: boolean) => void;
}

const ThemeContext = createContext<ThemeContextType | null>(null);

export function useTheme() {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error("useTheme must be used within a ThemeProvider");
  }
  return context;
}

interface ThemeProviderProps {
  children: ReactNode;
}

export function ThemeProvider({ children }: ThemeProviderProps) {
  const [theme, setThemeState] = useState("system");
  const [colorScheme, setColorSchemeState] = useState("ocean");
  const [rustMode, setRustModeState] = useState(false);
  const [loaded, setLoaded] = useState(false);

  // Load initial theme from config
  useEffect(() => {
    const loadTheme = async () => {
      try {
        const config = await invoke<{ ui: { color_scheme: string; rust_mode: boolean }; theme: string }>("get_config");
        setThemeState(config.theme || "dark");
        setColorSchemeState(config.ui.color_scheme || "ocean");
        setRustModeState(config.ui.rust_mode || false);
      } catch (error) {
        console.error("Failed to load theme config:", error);
      } finally {
        setLoaded(true);
      }
    };
    loadTheme();
  }, []);

  // Apply theme to document
  useEffect(() => {
    if (!loaded) return;
    
    const root = document.documentElement;
    
    // Determine actual theme (handle system preference)
    let effectiveTheme = theme;
    if (theme === "system") {
      effectiveTheme = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
    }
    
    // Apply theme mode
    root.setAttribute("data-theme", effectiveTheme);
    
    // Apply color scheme
    if (colorScheme && colorScheme !== "default") {
      root.setAttribute("data-color-scheme", colorScheme);
    } else {
      root.removeAttribute("data-color-scheme");
    }
    
    // Apply rust mode
    root.setAttribute("data-rust-mode", rustMode ? "true" : "false");
  }, [theme, colorScheme, rustMode, loaded]);

  // Listen for system theme changes
  useEffect(() => {
    if (theme !== "system") return;
    
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const handleChange = () => {
      const root = document.documentElement;
      root.setAttribute("data-theme", mediaQuery.matches ? "dark" : "light");
    };
    
    mediaQuery.addEventListener("change", handleChange);
    return () => mediaQuery.removeEventListener("change", handleChange);
  }, [theme]);

  const setTheme = (newTheme: string) => {
    setThemeState(newTheme);
  };

  const setColorScheme = (scheme: string) => {
    setColorSchemeState(scheme);
  };

  const setRustMode = (enabled: boolean) => {
    setRustModeState(enabled);
  };

  return (
    <ThemeContext.Provider value={{ theme, colorScheme, rustMode, setTheme, setColorScheme, setRustMode }}>
      {children}
    </ThemeContext.Provider>
  );
}
