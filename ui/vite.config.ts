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
      { find: "@", replacement: path.resolve(import.meta.dirname, "./src") },
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
  optimizeDeps: {
    exclude: ["mogh_ui"],
    // path-browserify is a CJS dep of monaco-yaml's yaml.worker. Vite's dep
    // scanner doesn't traverse `?worker` graphs, so without this it gets
    // served raw ("module is not defined" inside the worker). Force it
    // through prebundling to get CJS -> ESM interop.
    // @mantine/form and @tanstack/react-table are runtime-imported only via
    // the excluded (linked) mogh_ui — app src imports of react-table are
    // type-only, which the scanner erases — so they get served raw and their
    // nested CJS deps (fast-deep-equal, use-sync-external-store) break with
    // "does not provide an export named ...". Prebundling the packages
    // inlines those CJS deps with proper interop.
    include: ["path-browserify", "@mantine/form", "@tanstack/react-table"],
  },
  build: {
    // The only chunks above the default 500 kB warning limit are
    // monaco-editor's core (~2.7 MB) and prettier's typescript parser
    // (~900 kB). Both are already async — mogh_ui lazy-loads the editor
    // implementation, so they only download when an editor mounts — and
    // neither can be split further (each is one static import graph).
    // Raise the limit above them so the warning still catches an eager
    // chunk regression instead of firing on every build.
    chunkSizeWarningLimit: 2800,
    rolldownOptions: {
      output: {
        // Split stable vendor code out of the main chunk so app-code
        // changes don't invalidate the browser cache for react/mantine.
        codeSplitting: {
          groups: [
            {
              name: "react",
              test: /node_modules[\\/](react|react-dom|scheduler|react-router)[\\/]/,
            },
            {
              name: "mantine",
              test: /node_modules[\\/](@mantine|@floating-ui|react-transition-group|react-remove-scroll)[\\/]/,
            },
            {
              name: "tanstack",
              test: /node_modules[\\/]@tanstack[\\/]/,
            },
          ],
        },
      },
    },
  },
  css: {
    preprocessorOptions: {
      scss: {
        additionalData: '@use "mogh_ui/theme.scss" as theme;',
      },
    },
  },
});
