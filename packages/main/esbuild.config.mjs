import { build } from 'esbuild';

await build({
  entryPoints: ['src/index.ts'],
  outfile: 'bundle/index.js',
  bundle: true,
  format: 'cjs',
  platform: 'node',
  target: ['node20'],
  tsconfig: './tsconfig.json',
  external: ['electron', 'better-sqlite3'],
  logLevel: 'info',
  minify: true,
  sourcemap: false,
});
