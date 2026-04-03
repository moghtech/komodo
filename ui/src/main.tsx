import React from "react";
import ReactDOM from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { WebsocketProvider } from "@/lib/socket";
import { Router } from "@/router";
import { ThemeProvider } from "mogh_ui";
import { themeAdditionalColors } from "@/lib/color";

import "@mantine/core/styles.css";
// ‼️ import extra package styles after core package styles
import "@mantine/notifications/styles.css";
import "@mantine/spotlight/styles.css";
// Import local css after to avoid mantine default body color flash.
import "./index.scss";

// Run monaco setup
import "@/monaco";
import { Notifications } from "@mantine/notifications";
import initMonaco from "@/monaco/init";

initMonaco();

export const KOMODO_BASE_URL =
  import.meta.env.VITE_KOMODO_HOST ?? location.origin;
export const UPDATE_WS_URL =
  KOMODO_BASE_URL.replace("http", "ws") + "/ws/update";
const client = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <QueryClientProvider client={client}>
      <WebsocketProvider>
        <ThemeProvider additionalColors={themeAdditionalColors()}>
          <Router />
          <Notifications />
        </ThemeProvider>
      </WebsocketProvider>
    </QueryClientProvider>
  </React.StrictMode>,
);
