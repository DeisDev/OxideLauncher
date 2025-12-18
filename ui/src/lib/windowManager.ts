import { WebviewWindow, getAllWebviewWindows, getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { emit, listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

// Window state type matching Rust backend
export interface WindowState {
  x: number | null;
  y: number | null;
  width: number | null;
  height: number | null;
  maximized: boolean;
}

// Window labels for our dialog windows
export const WINDOW_LABELS = {
  MODPACK_BROWSER: "modpack-browser",
  MOD_BROWSER: "mod-browser",
  RESOURCE_BROWSER: "resource-browser",
  SHADER_BROWSER: "shader-browser",
} as const;

export type DialogWindowLabel = typeof WINDOW_LABELS[keyof typeof WINDOW_LABELS];

interface DialogWindowConfig {
  label: DialogWindowLabel;
  url: string;
  title: string;
  width?: number;
  height?: number;
  minWidth?: number;
  minHeight?: number;
}

// Route paths (used with hash router - will be prefixed with index.html#)
const DIALOG_ROUTES: Record<DialogWindowLabel, string> = {
  [WINDOW_LABELS.MODPACK_BROWSER]: "/dialog/modpack-browser",
  [WINDOW_LABELS.MOD_BROWSER]: "/dialog/mod-browser",
  [WINDOW_LABELS.RESOURCE_BROWSER]: "/dialog/resource-browser",
  [WINDOW_LABELS.SHADER_BROWSER]: "/dialog/shader-browser",
};

const DIALOG_CONFIGS: Record<DialogWindowLabel, Omit<DialogWindowConfig, "label">> = {
  [WINDOW_LABELS.MODPACK_BROWSER]: {
    url: DIALOG_ROUTES[WINDOW_LABELS.MODPACK_BROWSER],
    title: "Browse Modpacks",
    width: 1200,
    height: 800,
    minWidth: 900,
    minHeight: 600,
  },
  [WINDOW_LABELS.MOD_BROWSER]: {
    url: DIALOG_ROUTES[WINDOW_LABELS.MOD_BROWSER],
    title: "Browse Mods",
    width: 1200,
    height: 800,
    minWidth: 900,
    minHeight: 600,
  },
  [WINDOW_LABELS.RESOURCE_BROWSER]: {
    url: DIALOG_ROUTES[WINDOW_LABELS.RESOURCE_BROWSER],
    title: "Browse Resource Packs",
    width: 1200,
    height: 800,
    minWidth: 900,
    minHeight: 600,
  },
  [WINDOW_LABELS.SHADER_BROWSER]: {
    url: DIALOG_ROUTES[WINDOW_LABELS.SHADER_BROWSER],
    title: "Browse Shaders",
    width: 1200,
    height: 800,
    minWidth: 900,
    minHeight: 600,
  },
};

// Currently open dialog window (only one allowed at a time)
// Note: This state is only valid within the main window context
let currentDialogWindow: WebviewWindow | null = null;
let currentDialogLabel: DialogWindowLabel | null = null;

/**
 * Close any currently open dialog window
 * Works from either main window or dialog window context
 */
export async function closeCurrentDialog(): Promise<void> {
  try {
    // Get all open windows and close any dialog windows
    const windows = await getAllWebviewWindows();
    const dialogLabels = Object.values(WINDOW_LABELS);
    
    for (const window of windows) {
      if (dialogLabels.includes(window.label as DialogWindowLabel)) {
        try {
          await window.close();
        } catch {
          // Window might already be closed
        }
      }
    }
    
    // Clear state
    currentDialogWindow = null;
    currentDialogLabel = null;
    
    // Emit event so main window can hide overlay
    await emitDialogClosed();
  } catch (error) {
    console.error("Failed to close dialog:", error);
  }
}

/**
 * Check if a specific dialog is currently open
 */
export async function isDialogOpen(label: DialogWindowLabel): Promise<boolean> {
  try {
    const windows = await getAllWebviewWindows();
    return windows.some(w => w.label === label);
  } catch {
    return false;
  }
}

/**
 * Check if any dialog is currently open
 */
export async function isAnyDialogOpen(): Promise<boolean> {
  try {
    const windows = await getAllWebviewWindows();
    const dialogLabels = Object.values(WINDOW_LABELS);
    return windows.some(w => dialogLabels.includes(w.label as DialogWindowLabel));
  } catch {
    return false;
  }
}

/**
 * Get the currently open dialog label
 */
export function getCurrentDialogLabel(): DialogWindowLabel | null {
  return currentDialogLabel;
}

/**
 * Open a dialog window. Closes any existing dialog first.
 * @param label The dialog window label to open
 * @param urlParams Optional URL parameters to pass to the dialog
 * @returns The created WebviewWindow, or null if creation failed
 */
export async function openDialogWindow(
  label: DialogWindowLabel,
  urlParams?: Record<string, string>
): Promise<WebviewWindow | null> {
  // Close any existing dialog first
  await closeCurrentDialog();

  const config = DIALOG_CONFIGS[label];
  if (!config) {
    console.error(`Unknown dialog label: ${label}`);
    return null;
  }

  // Build URL with hash router format: index.html#/dialog/path?params
  // This is required because Tauri loads files directly, not through a web server
  let route = config.url;
  if (urlParams && Object.keys(urlParams).length > 0) {
    const params = new URLSearchParams(urlParams);
    route = `${route}?${params.toString()}`;
  }
  
  // Use index.html with hash for proper routing in production builds
  const url = `index.html#${route}`;

  // Check for saved window state
  const savedState = await getWindowState(label);
  const usePosition = savedState && savedState.x !== null && savedState.y !== null;
  const useSize = savedState && savedState.width !== null && savedState.height !== null;

  try {
    // Check if window already exists (e.g., from a previous session)
    const existingWindows = await getAllWebviewWindows();
    const existing = existingWindows.find(w => w.label === label);
    if (existing) {
      // Focus the existing window
      await existing.setFocus();
      currentDialogWindow = existing;
      currentDialogLabel = label;
      return existing;
    }

    // Create new window with saved or default dimensions
    const webview = new WebviewWindow(label, {
      url,
      title: config.title,
      width: useSize ? savedState!.width! : config.width,
      height: useSize ? savedState!.height! : config.height,
      minWidth: config.minWidth,
      minHeight: config.minHeight,
      resizable: true,
      center: !usePosition, // Only center if we don't have a saved position
      x: usePosition ? savedState!.x! : undefined,
      y: usePosition ? savedState!.y! : undefined,
      decorations: false, // Disable native title bar
      transparent: false,
      parent: "main", // Set parent to main window for modal-like behavior
    });

    // Wait for window creation
    await new Promise<void>((resolve, reject) => {
      webview.once("tauri://created", () => {
        currentDialogWindow = webview;
        currentDialogLabel = label;
        resolve();
      });

      webview.once("tauri://error", (e) => {
        console.error("Failed to create dialog window:", e);
        reject(new Error(`Failed to create dialog window: ${e}`));
      });
    });

    // Emit event so main window can show overlay
    await emitDialogOpened(label);

    // Listen for window close to clean up state and save position
    webview.onCloseRequested(async () => {
      if (currentDialogLabel === label) {
        // Save position before closing
        await saveDialogWindowPosition(label, webview);
        
        currentDialogWindow = null;
        currentDialogLabel = null;
        // Emit event so main window can hide overlay
        await emitDialogClosed();
      }
    });

    return webview;
  } catch (error) {
    console.error("Failed to open dialog window:", error);
    currentDialogWindow = null;
    currentDialogLabel = null;
    return null;
  }
}

/**
 * Focus the currently open dialog window, if any
 */
export async function focusCurrentDialog(): Promise<void> {
  if (currentDialogWindow) {
    try {
      await currentDialogWindow.setFocus();
    } catch {
      // Window might have been closed
      currentDialogWindow = null;
      currentDialogLabel = null;
    }
  }
}

/**
 * Get the window by label from currently open windows
 */
export async function getWindowByLabel(label: string): Promise<WebviewWindow | null> {
  try {
    const windows = await getAllWebviewWindows();
    return windows.find(w => w.label === label) || null;
  } catch {
    return null;
  }
}

/**
 * Check if the current window is a dialog window (not the main window)
 */
export function isDialogWindowContext(): boolean {
  // Dialog windows use hash router, so we check the URL hash
  const hash = window.location.hash;
  return hash.includes("/dialog/");
}

// Track listener cleanup
let mainWindowCloseListener: UnlistenFn | null = null;

/**
 * Setup listener on main window to close all dialog windows when main closes.
 * Call this once when the main app initializes.
 */
export async function setupMainWindowCloseHandler(): Promise<void> {
  // Only setup on main window
  if (isDialogWindowContext()) return;
  
  try {
    const mainWindow = getCurrentWebviewWindow();
    
    // Clean up existing listener if any
    if (mainWindowCloseListener) {
      mainWindowCloseListener();
      mainWindowCloseListener = null;
    }
    
    // Listen for close request on main window
    mainWindowCloseListener = await mainWindow.onCloseRequested(async () => {
      // Close any open dialog windows first
      await closeAllDialogWindows();
    });
  } catch (error) {
    console.error("Failed to setup main window close handler:", error);
  }
}

/**
 * Close all dialog windows
 */
export async function closeAllDialogWindows(): Promise<void> {
  try {
    const windows = await getAllWebviewWindows();
    for (const window of windows) {
      // Close all windows except main
      if (window.label !== "main") {
        try {
          await window.close();
        } catch {
          // Window might already be closed
        }
      }
    }
    currentDialogWindow = null;
    currentDialogLabel = null;
  } catch (error) {
    console.error("Failed to close dialog windows:", error);
  }
}

/**
 * Emit event when dialog opens (for main window to show overlay)
 */
export async function emitDialogOpened(label: DialogWindowLabel): Promise<void> {
  await emit("dialog-opened", { label });
}

/**
 * Emit event when dialog closes (for main window to hide overlay)
 */
export async function emitDialogClosed(): Promise<void> {
  await emit("dialog-closed", {});
}

/**
 * Setup listeners for dialog open/close events
 * @returns cleanup function
 */
export async function setupDialogEventListeners(
  onOpen: (label: DialogWindowLabel) => void,
  onClose: () => void
): Promise<() => void> {
  const unlistenOpen = await listen<{ label: DialogWindowLabel }>("dialog-opened", (event) => {
    onOpen(event.payload.label);
  });
  
  const unlistenClose = await listen("dialog-closed", () => {
    onClose();
  });
  
  return () => {
    unlistenOpen();
    unlistenClose();
  };
}

// Window position memory functions

/**
 * Get saved window state from backend
 */
export async function getWindowState(windowType: string): Promise<WindowState | null> {
  try {
    return await invoke<WindowState | null>("get_window_state", { windowType });
  } catch (error) {
    console.error("Failed to get window state:", error);
    return null;
  }
}

/**
 * Save window state to backend
 */
export async function saveWindowState(windowType: string, state: WindowState): Promise<void> {
  try {
    await invoke("save_window_state", { windowType, windowState: state });
  } catch (error) {
    console.error("Failed to save window state:", error);
  }
}

/**
 * Check if window position memory is enabled for a window type
 */
export async function isWindowPositionMemoryEnabled(windowType: string): Promise<boolean> {
  try {
    return await invoke<boolean>("is_window_position_memory_enabled", { windowType });
  } catch (error) {
    console.error("Failed to check window position memory:", error);
    return false;
  }
}

/**
 * Get current window position and size
 */
export async function getCurrentWindowState(window: WebviewWindow): Promise<WindowState> {
  try {
    const position = await window.outerPosition();
    const size = await window.outerSize();
    const maximized = await window.isMaximized();
    
    return {
      x: position.x,
      y: position.y,
      width: size.width,
      height: size.height,
      maximized,
    };
  } catch (error) {
    console.error("Failed to get current window state:", error);
    return {
      x: null,
      y: null,
      width: null,
      height: null,
      maximized: false,
    };
  }
}

/**
 * Apply saved window state to a window
 */
export async function applyWindowState(window: WebviewWindow, state: WindowState): Promise<void> {
  try {
    // Only apply if we have valid position data
    if (state.x !== null && state.y !== null) {
      await window.setPosition({ type: "Physical", x: state.x, y: state.y });
    }
    
    if (state.width !== null && state.height !== null) {
      await window.setSize({ type: "Physical", width: state.width, height: state.height });
    }
    
    if (state.maximized) {
      await window.maximize();
    }
  } catch (error) {
    console.error("Failed to apply window state:", error);
  }
}

/**
 * Save and restore main window position
 * Call this once on app startup in the main window
 */
export async function setupMainWindowPositionMemory(): Promise<() => void> {
  // Only setup on main window
  if (isDialogWindowContext()) return () => {};
  
  const mainWindow = getCurrentWebviewWindow();
  
  // Try to restore saved position
  const savedState = await getWindowState("main");
  if (savedState && (savedState.x !== null || savedState.width !== null)) {
    await applyWindowState(mainWindow, savedState);
  }
  
  // Save position when window is moved or resized (debounced)
  let saveTimeout: number | null = null;
  
  const savePosition = async () => {
    const isEnabled = await isWindowPositionMemoryEnabled("main");
    if (!isEnabled) return;
    
    const state = await getCurrentWindowState(mainWindow);
    await saveWindowState("main", state);
  };
  
  const debouncedSave = () => {
    if (saveTimeout) {
      clearTimeout(saveTimeout);
    }
    saveTimeout = setTimeout(savePosition, 500) as unknown as number;
  };
  
  // Listen for move and resize events
  const unlistenMove = mainWindow.onMoved(debouncedSave);
  const unlistenResize = mainWindow.onResized(debouncedSave);
  
  // Save final position when closing
  const unlistenClose = mainWindow.onCloseRequested(async () => {
    const isEnabled = await isWindowPositionMemoryEnabled("main");
    if (isEnabled) {
      const state = await getCurrentWindowState(mainWindow);
      await saveWindowState("main", state);
    }
  });
  
  // Return cleanup function
  return async () => {
    if (saveTimeout) {
      clearTimeout(saveTimeout);
    }
    (await unlistenMove)();
    (await unlistenResize)();
    (await unlistenClose)();
  };
}

/**
 * Save dialog window position when closing
 */
export async function saveDialogWindowPosition(label: DialogWindowLabel, window: WebviewWindow): Promise<void> {
  const isEnabled = await isWindowPositionMemoryEnabled(label);
  if (!isEnabled) return;
  
  const state = await getCurrentWindowState(window);
  await saveWindowState(label, state);
}
