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
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
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
  css: {
    preprocessorOptions: {
      scss: {
        additionalData: '@use "mogh_ui/theme.scss" as theme;',
      },
    },
  },
});
