import { build } from 'esbuild';
import { cp, mkdir, rm } from 'node:fs/promises';

await rm('bundle', { recursive: true, force: true });
await mkdir('bundle/preload', { recursive: true });

await build({
  entryPoints: ['src/app.ts'],
  outfile: 'bundle/app.js',
  bundle: true,
  format: 'esm',
  platform: 'browser',
  target: ['chrome114'],
  tsconfig: './tsconfig.json',
  logLevel: 'info',
  minify: true,
  sourcemap: false,
});

await build({
  entryPoints: ['src/preload/preload.ts'],
  outfile: 'bundle/preload/preload.js',
  bundle: true,
  format: 'cjs',
  platform: 'node',
  target: ['node20'],
  tsconfig: './tsconfig.json',
  external: ['electron'],
  logLevel: 'info',
  minify: true,
  sourcemap: false,
});

await Promise.all([
  cp('src/index.html', 'bundle/index.html'),
  cp('src/styles.css', 'bundle/styles.css'),
]);
