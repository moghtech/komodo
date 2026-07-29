import path from "path";
import dotenv from "dotenv";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

dotenv.config({ path: ".env.development" });

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    allowedHosts: process.env.ALLOWED_HOSTS?.split(","),
  },
  resolve: {
    alias: [
      { find: "@", replacement: path.resolve(__dirname, "./src") },
      // monaco-editor >= 0.53 has an exports map ("./*.js": "./esm/vs/*.js"),
      // so legacy deep imports like "monaco-editor/esm/vs/..." no longer
      // resolve. monaco-worker-manager (used by monaco-yaml's worker) still
      // imports the legacy path — rewrite it to the exports-map form.
      {
        find: /^monaco-editor\/esm\/vs\/(.*)$/,
        replacement: "monaco-editor/$1",
      },
    ],
    dedupe: [
      "@mantine/core",
      "@mantine/form",
      "@mantine/hooks",
      "@mantine/notifications",
      "@monaco-editor/react",
      "@tanstack/react-table",
      "@tanstack/react-query",
      "lucide-react",
      "mogh_auth_client",
      "monaco-editor",
      "monaco-yaml",
      "react",
      "react-dom",
      "react-router-dom",
    ],
  },
  optimizeDeps: { exclude: ["mogh_ui"] },
  css: {
    preprocessorOptions: {
      scss: {
        additionalData: '@use "mogh_ui/theme.scss" as theme;',
      },
    },
  },
});
