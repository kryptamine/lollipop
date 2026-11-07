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
    // One page per Tauri window: the overlay, the settings window and the Recap.
    rolldownOptions: {
      input: {
        overlay: decodeURIComponent(new URL('ui/index.html', import.meta.url).pathname),
        settings: decodeURIComponent(new URL('ui/settings.html', import.meta.url).pathname),
        recap: decodeURIComponent(new URL('ui/recap.html', import.meta.url).pathname),
      },
    },
  },
});
