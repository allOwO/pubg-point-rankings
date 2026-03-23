/**
 * Main process entry point
 */

import { initializeMain } from './main/bootstrap';

console.log('PUBG Point Rankings - Main Process Starting...');

// Initialize the main process
initializeMain().catch((error) => {
  console.error('Failed to initialize main process:', error);
  process.exit(1);
});

// Export public API for external use
export * from './main/bootstrap';
export * from './main/window';
