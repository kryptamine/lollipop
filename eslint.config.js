import comments from "@eslint-community/eslint-plugin-eslint-comments";
import js from "@eslint/js";
import { defineConfig, globalIgnores } from "eslint/config";
import tseslint from "typescript-eslint";

export default defineConfig([
  globalIgnores(["dist/", "target/", "crates/", "scripts/", "eslint.config.js"]),
  js.configs.recommended,
  tseslint.configs.strictTypeChecked,
  tseslint.configs.stylisticTypeChecked,
  {
    linterOptions: { reportUnusedDisableDirectives: "error" },
    languageOptions: { parserOptions: { projectService: true } },
    plugins: { "eslint-comments": comments },
    rules: {
      complexity: ["error", 10],
      "max-depth": ["error", 3],
      "max-lines": ["error", { max: 200, skipBlankLines: true, skipComments: true }],
      "max-lines-per-function": ["error", 60],
      "max-params": ["error", 4],
      // An exception must state its reason (ADR 0001).
      "eslint-comments/require-description": "error",
      "no-restricted-imports": [
        "error",
        { patterns: [{ group: ["@tauri-apps/*"], message: "Reach Tauri through ui/src/bridge.ts." }] },
      ],
    },
  },
  {
    files: ["ui/src/bridge.ts"],
    rules: { "no-restricted-imports": "off" },
  },
]);
