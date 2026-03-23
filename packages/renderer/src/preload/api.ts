/**
 * Renderer API
 * Convenient wrapper around the exposed electronAPI for use in renderer code
 */

import type { ElectronAPI } from './types';

/**
 * Get the electron API from the window object
 * Throws an error if the API is not available (preload script not loaded)
 */
export function getElectronAPI(): ElectronAPI {
  if (typeof window === 'undefined') {
    throw new Error('getElectronAPI can only be called in the renderer process');
  }

  if (!window.electronAPI) {
    throw new Error(
      'electronAPI is not available. Make sure the preload script is loaded correctly.'
    );
  }

  return window.electronAPI;
}

/**
 * Check if the Electron API is available
 */
export function isElectronAvailable(): boolean {
  return typeof window !== 'undefined' && !!window.electronAPI;
}

// Re-export types
export * from './types';
