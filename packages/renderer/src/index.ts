/**
 * Renderer process entry point
 * 
 * This module provides typed access to the current desktop host API.
 */

export * from './tauri-api';

// Export app initialization (for programmatic access)
export { AppState, state, navigateTo, showToast } from './app';

// Log that renderer module is loaded
console.log('PUBG Point Rankings - Renderer Module Loaded');
