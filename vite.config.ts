import preact from "@preact/preset-vite";
import { defineConfig } from "vite";

export default defineConfig({
  root: "ui",
  plugins: [preact()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { outDir: "../dist", emptyOutDir: true },
});
