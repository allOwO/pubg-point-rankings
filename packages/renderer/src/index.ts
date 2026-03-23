/**
 * Renderer process entry point
 * 
 * This module provides typed access to the main process API
 * via the preload script exposed electronAPI.
 */

// Export preload types and API wrapper
export * from './preload';

// Export app initialization (for programmatic access)
export { AppState, state, getAPI, navigateTo, showToast } from './app';

// Log that renderer module is loaded
console.log('PUBG Point Rankings - Renderer Module Loaded');
