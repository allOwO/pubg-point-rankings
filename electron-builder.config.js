/**
 * Electron Builder Configuration
 * Separated from package.json for better maintainability
 */

module.exports = {
  appId: 'com.yourcompany.pubg-point-rankings',
  productName: 'PUBG Point Rankings',
  copyright: 'Copyright © 2024',
  compression: 'maximum',
  electronLanguages: ['en-US', 'zh-CN'],
  removePackageScripts: true,
  removePackageKeywords: true,
  
  directories: {
    output: 'dist-electron',
    app: '.',
    buildResources: 'build-resources',
  },

  files: [
    'index.js',
    'packages/main/bundle/**/*',
    'packages/renderer/bundle/**/*',
    'package.json',
    '!**/*.map',
    '!**/*.tsbuildinfo',
    '!**/*.d.ts',
    '!**/*.test.*',
    '!**/src/**',
    '!**/docs/**',
    '!**/build-resources/**',
    '!packages/*/dist/**',
    '!node_modules/better-sqlite3/src/**',
    '!node_modules/better-sqlite3/deps/**',
    '!node_modules/better-sqlite3/build/**/obj.target/**',
    '!node_modules/better-sqlite3/build/config.gypi',
    '!node_modules/better-sqlite3/build/node_gyp_bins/**',
    '!node_modules/better-sqlite3/binding.gyp',
    '!node_modules/better-sqlite3/README.md',
    '!node_modules/bindings/LICENSE.md',
    '!node_modules/file-uri-to-path/History.md',
  ],

  // macOS configuration
  mac: {
    category: 'public.app-category.games',
    target: [
      {
        target: 'dmg',
        arch: ['x64', 'arm64'],
      },
      {
        target: 'zip',
        arch: ['x64', 'arm64'],
      },
    ],
    hardenedRuntime: false,
    gatekeeperAssess: false,
  },

  dmg: {
    contents: [
      {
        x: 130,
        y: 220,
      },
      {
        x: 410,
        y: 220,
        type: 'link',
        path: '/Applications',
      },
    ],
    window: {
      width: 540,
      height: 380,
    },
  },

  // Windows configuration
  win: {
    target: [
      {
        target: 'nsis',
        arch: ['x64'],
      },
    ],
    verifyUpdateCodeSignature: false,
  },

  nsis: {
    buildUniversalInstaller: false,
    oneClick: false,
    allowToChangeInstallationDirectory: true,
    createDesktopShortcut: true,
    createStartMenuShortcut: true,
    shortcutName: 'PUBG Point Rankings',
  },

  // Linux configuration
  linux: {
    target: [
      {
        target: 'AppImage',
        arch: ['x64'],
      },
      {
        target: 'deb',
        arch: ['x64'],
      },
      {
        target: 'rpm',
        arch: ['x64'],
      },
    ],
    category: 'Game',
    maintainer: 'PUBG Point Rankings',
    vendor: 'PUBG Point Rankings',
    synopsis: 'PUBG Point Rankings - Track matches and calculate teammate points',
    description: 'An Electron-based app for tracking PUBG matches and calculating teammate points.',
  },

  // ASAR configuration
  asar: true,
  asarUnpack: [
    'node_modules/better-sqlite3/build/Release/*.node',
  ],

  // Overwolf Electron specific (when using @overwolf/ow-electron)
  // Note: Uncomment and adjust when migrating to Overwolf Electron
  // electronDist: 'node_modules/@overwolf/ow-electron/dist',
  // electronVersion: '28.0.0',
};
