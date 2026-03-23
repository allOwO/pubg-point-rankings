/**
 * Electron Application Entry Point
 * 
 * This is the main entry point for the Electron application.
 * It delegates to the main package for initialization.
 */

const fs = require('node:fs');
const path = require('node:path');

const isDev = process.env.NODE_ENV === 'development';
const bundledEntry = './packages/main/bundle/index.js';
const bundledEntryPath = path.join(__dirname, 'packages', 'main', 'bundle', 'index.js');
const distEntry = './packages/main/dist/index.js';

// Forward to the appropriate main package entry point
module.exports = require(!isDev && fs.existsSync(bundledEntryPath) ? bundledEntry : distEntry);
