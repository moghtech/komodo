import { defineConfig } from "vite";
import tspaths from "vite-tsconfig-paths";
import react from "@vitejs/plugin-react";
import dotenv from "dotenv";

dotenv.config({ path: ".env.development" });

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react(), tspaths()],
  server: { allowedHosts: [process.env.ALLOWED_HOST] },
});
