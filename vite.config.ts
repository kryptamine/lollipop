import { defineConfig } from 'vite';
import preactPreset from '@preact/preset-vite';

export default defineConfig({
  root: 'ui',
  plugins: [preactPreset()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    outDir: '../dist',
    emptyOutDir: true,
  },
});
