# Build Resources

This directory contains resources used by electron-builder during the packaging process.

## Icons

Place application icons here:

- `icon.ico` - Windows icon (256x256 or multi-size ICO)
- `icon.icns` - macOS icon (1024x1024 ICNS)
- `icons/` - Linux icons (multiple PNG sizes: 16x16, 32x32, 48x48, 64x64, 128x128, 256x256, 512x512, 1024x1024)

## Entitlements (macOS)

- `entitlements.mac.plist` - macOS entitlements for hardened runtime

## Generating Icons

You can generate icons from a single source image using:

```bash
# macOS: Use iconutil or sips
# Windows: Use icotools or online converters
# Linux: Place PNG files of various sizes in icons/ directory
```

## Note

These are placeholder files. Replace with your actual application icons before distribution.
