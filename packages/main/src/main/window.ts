/**
 * Main Window Management
 * Creates and manages the Electron BrowserWindow
 */

import { BrowserWindow, screen, shell } from 'electron';
import * as path from 'node:path';

// Keep a global reference to prevent garbage collection
let mainWindow: BrowserWindow | null = null;

// Window configuration constants
const WINDOW_WIDTH = 1200;
const WINDOW_HEIGHT = 800;
const WINDOW_MIN_WIDTH = 800;
const WINDOW_MIN_HEIGHT = 600;

/**
 * Get the preload script path
 */
function getPreloadPath(): string {
  const isDev = process.env.NODE_ENV === 'development';
  
  if (isDev) {
    return path.resolve(__dirname, '../../../renderer/bundle/preload/preload.js');
  }
  
  // In production, electron-builder places files in resources/app
  return path.join(process.resourcesPath, 'app', 'packages', 'renderer', 'bundle', 'preload', 'preload.js');
}

/**
 * Get the renderer HTML file path
 */
function getRendererPath(): string {
  const isDev = process.env.NODE_ENV === 'development';
  
  if (isDev) {
    return path.resolve(__dirname, '../../../renderer/bundle/index.html');
  }
  
  // In production
  return path.join(process.resourcesPath, 'app', 'packages', 'renderer', 'bundle', 'index.html');
}

/**
 * Create the main application window
 */
export function createMainWindow(): BrowserWindow {
  if (mainWindow) {
    return mainWindow;
  }

  // Get primary display dimensions
  const primaryDisplay = screen.getPrimaryDisplay();
  const { width: screenWidth, height: screenHeight } = primaryDisplay.workAreaSize;

  // Calculate centered position
  const x = Math.round((screenWidth - WINDOW_WIDTH) / 2);
  const y = Math.round((screenHeight - WINDOW_HEIGHT) / 2);

  // Create the browser window
  mainWindow = new BrowserWindow({
    width: WINDOW_WIDTH,
    height: WINDOW_HEIGHT,
    x,
    y,
    minWidth: WINDOW_MIN_WIDTH,
    minHeight: WINDOW_MIN_HEIGHT,
    show: false, // Show when ready to prevent visual flash
    frame: true,
    titleBarStyle: 'default',
    webPreferences: {
      preload: getPreloadPath(),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false, // Required for preload script access
      allowRunningInsecureContent: false,
      experimentalFeatures: false,
    },
    title: 'PUBG Point Rankings',
    icon: getIconPath(),
  });

  // Load the renderer
  const rendererPath = getRendererPath();
  console.log('Loading renderer from:', rendererPath);
  
  mainWindow.loadFile(rendererPath).catch((err) => {
    console.error('Failed to load renderer:', err);
  });

  // Show window when ready to prevent visual flash
  mainWindow.once('ready-to-show', () => {
    mainWindow?.show();
    
    // Open DevTools in development
    if (process.env.NODE_ENV === 'development') {
      mainWindow?.webContents.openDevTools();
    }
  });

  // Handle window closed
  mainWindow.on('closed', () => {
    mainWindow = null;
  });

  // Handle external links - open in system browser
  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    if (url.startsWith('http:') || url.startsWith('https:')) {
      shell.openExternal(url);
    }
    return { action: 'deny' };
  });

  return mainWindow;
}

/**
 * Get the appropriate icon path for the platform
 */
function getIconPath(): string | undefined {
  const isDev = process.env.NODE_ENV === 'development';
  const iconDir = isDev
    ? path.resolve(__dirname, '../../../renderer/assets')
    : path.join(process.resourcesPath, 'app', 'packages', 'renderer', 'assets');
  
  switch (process.platform) {
    case 'win32':
      return path.join(iconDir, 'icon.ico');
    case 'darwin':
      return path.join(iconDir, 'icon.icns');
    default:
      return path.join(iconDir, 'icon.png');
  }
}

/**
 * Get the main window instance
 */
export function getMainWindow(): BrowserWindow | null {
  return mainWindow;
}

/**
 * Show the main window (create if needed)
 */
export function showMainWindow(): BrowserWindow {
  if (!mainWindow || mainWindow.isDestroyed()) {
    return createMainWindow();
  }
  
  if (mainWindow.isMinimized()) {
    mainWindow.restore();
  }
  
  mainWindow.focus();
  return mainWindow;
}

/**
 * Hide the main window
 */
export function hideMainWindow(): void {
  mainWindow?.hide();
}

/**
 * Close the main window
 */
export function closeMainWindow(): void {
  mainWindow?.close();
  mainWindow = null;
}

/**
 * Check if the main window is visible
 */
export function isMainWindowVisible(): boolean {
  return mainWindow?.isVisible() ?? false;
}

/**
 * Send a message to the renderer process
 */
export function sendToRenderer(channel: string, ...args: unknown[]): void {
  if (mainWindow && !mainWindow.isDestroyed()) {
    mainWindow.webContents.send(channel, ...args);
  }
}
