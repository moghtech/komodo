import React from "react";
import ReactDOM from "react-dom/client";
import { MantineProvider } from "@mantine/core";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { DEFAULT_COLOR_SCHEME, theme } from "@/theme";
import { WebsocketProvider } from "@/lib/socket";
import { Router } from "@/router";

import "@mantine/core/styles.css";
// ‼️ import notifications styles after core package styles
import "@mantine/notifications/styles.css";
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
        <MantineProvider
          theme={theme}
          defaultColorScheme={DEFAULT_COLOR_SCHEME}
        >
          <Router />
          <Notifications />
        </MantineProvider>
      </WebsocketProvider>
    </QueryClientProvider>
  </React.StrictMode>,
);
